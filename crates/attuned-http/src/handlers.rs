//! HTTP request handlers.

use attuned_core::{
    HealthCheck, HealthState, HealthStatus, PromptContext, RuleTranslator, Source, StateSnapshot,
    Translator,
};
use attuned_store::StateStore;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "inference")]
use attuned_infer::{Baseline, InferenceConfig, InferenceEngine, InferenceSource};
#[cfg(feature = "inference")]
use dashmap::DashMap;
#[cfg(feature = "inference")]
use std::collections::HashMap;

/// Application state shared across handlers.
pub struct AppState<S: StateStore> {
    /// The state store backend.
    pub store: Arc<S>,
    /// The translator for converting state to context.
    pub translator: Arc<dyn Translator>,
    /// Server start time for uptime calculation.
    pub start_time: Instant,
    /// Inference engine (optional, requires "inference" feature).
    #[cfg(feature = "inference")]
    pub inference_engine: Option<InferenceEngine>,
    /// Per-user baselines for delta analysis.
    #[cfg(feature = "inference")]
    pub baselines: Arc<DashMap<String, Baseline>>,
}

impl<S: StateStore> AppState<S> {
    /// Create new application state.
    pub fn new(store: S) -> Self {
        Self {
            store: Arc::new(store),
            translator: Arc::new(RuleTranslator::default()),
            start_time: Instant::now(),
            #[cfg(feature = "inference")]
            inference_engine: None,
            #[cfg(feature = "inference")]
            baselines: Arc::new(DashMap::new()),
        }
    }

    /// Create application state with inference enabled.
    #[cfg(feature = "inference")]
    pub fn with_inference(store: S, config: Option<InferenceConfig>) -> Self {
        let engine = match config {
            Some(c) => InferenceEngine::with_config(c),
            None => InferenceEngine::default(),
        };
        Self {
            store: Arc::new(store),
            translator: Arc::new(RuleTranslator::default()),
            start_time: Instant::now(),
            inference_engine: Some(engine),
            baselines: Arc::new(DashMap::new()),
        }
    }
}

/// Request body for upserting state.
#[derive(Debug, Deserialize)]
pub struct UpsertStateRequest {
    /// User ID to update state for.
    pub user_id: String,
    /// Source of the state data.
    #[serde(default)]
    pub source: SourceInput,
    /// Confidence level of the state data.
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    /// Axis values to set.
    pub axes: std::collections::BTreeMap<String, f32>,
    /// Optional message text for inference (requires "inference" feature).
    /// When provided, axes are inferred from the message and merged with explicit axes.
    /// Explicit axes always override inferred values.
    #[serde(default)]
    pub message: Option<String>,
}

fn default_confidence() -> f32 {
    1.0
}

/// Source of state data in API requests.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceInput {
    /// User explicitly provided this state.
    #[default]
    SelfReport,
    /// State was inferred from behavior.
    Inferred,
    /// Combination of self-report and inference.
    Mixed,
}

impl From<SourceInput> for Source {
    fn from(s: SourceInput) -> Self {
        match s {
            SourceInput::SelfReport => Source::SelfReport,
            SourceInput::Inferred => Source::Inferred,
            SourceInput::Mixed => Source::Mixed,
        }
    }
}

/// Response for state operations.
#[derive(Debug, Serialize)]
pub struct StateResponse {
    /// User ID.
    pub user_id: String,
    /// Timestamp of last update (Unix ms).
    pub updated_at_unix_ms: i64,
    /// Source of the state data.
    pub source: String,
    /// Confidence level.
    pub confidence: f32,
    /// Axis values.
    pub axes: std::collections::BTreeMap<String, f32>,
}

impl From<StateSnapshot> for StateResponse {
    fn from(s: StateSnapshot) -> Self {
        Self {
            user_id: s.user_id,
            updated_at_unix_ms: s.updated_at_unix_ms,
            source: s.source.to_string(),
            confidence: s.confidence,
            axes: s.axes,
        }
    }
}

/// Error response format.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error details.
    pub error: ErrorDetail,
}

/// Detailed error information.
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    /// Error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Request ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response.
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                request_id: None,
            },
        }
    }
}

/// POST /v1/state - Upsert state
#[tracing::instrument(skip(state, body))]
#[allow(unused_mut)] // mut needed when inference feature is enabled
pub async fn upsert_state<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Json(body): Json<UpsertStateRequest>,
) -> impl IntoResponse {
    let mut axes = body.axes;
    let mut source: Source = body.source.into();

    // Run inference if enabled and message provided
    #[cfg(feature = "inference")]
    if let (Some(engine), Some(message)) = (&state.inference_engine, &body.message) {
        // Get or create baseline for user
        let mut baseline_ref = state
            .baselines
            .entry(body.user_id.clone())
            .or_insert_with(|| engine.new_baseline());

        // Run inference with baseline
        let inferred = engine.infer_with_baseline(message, &mut baseline_ref, None);

        // Merge: explicit axes override inferred
        for estimate in inferred.all() {
            if !axes.contains_key(&estimate.axis) {
                // Only use inferred if not explicitly provided
                axes.insert(estimate.axis.clone(), estimate.value);
            }
        }

        // Mark source as mixed if we used inference
        if !inferred.is_empty() && source == Source::SelfReport {
            source = Source::Mixed;
        }
    }

    let snapshot = match StateSnapshot::builder()
        .user_id(&body.user_id)
        .source(source)
        .confidence(body.confidence)
        .axes(axes.into_iter())
        .build()
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("VALIDATION_ERROR", &e.to_string())),
            )
                .into_response();
        }
    };

    match state.store.upsert_latest(snapshot).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("STORE_ERROR", &e.to_string())),
        )
            .into_response(),
    }
}

/// GET /v1/state/:user_id - Get state
#[tracing::instrument(skip(state))]
pub async fn get_state<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_latest(&user_id).await {
        Ok(Some(snapshot)) => Json(StateResponse::from(snapshot)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "USER_NOT_FOUND",
                &format!("No state found for user {}", user_id),
            )),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("STORE_ERROR", &e.to_string())),
        )
            .into_response(),
    }
}

/// DELETE /v1/state/:user_id - Delete state
#[tracing::instrument(skip(state))]
pub async fn delete_state<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.store.delete(&user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("STORE_ERROR", &e.to_string())),
        )
            .into_response(),
    }
}

/// GET /v1/context/:user_id - Get translated context
#[tracing::instrument(skip(state))]
pub async fn get_context<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_latest(&user_id).await {
        Ok(Some(snapshot)) => {
            let context = state.translator.to_prompt_context(&snapshot);
            Json(context).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "USER_NOT_FOUND",
                &format!("No state found for user {}", user_id),
            )),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("STORE_ERROR", &e.to_string())),
        )
            .into_response(),
    }
}

/// Request body for inline translation.
#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    /// Axis values to translate.
    pub axes: std::collections::BTreeMap<String, f32>,
    /// Source of the state data.
    #[serde(default)]
    pub source: SourceInput,
    /// Confidence level.
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

/// POST /v1/translate - Translate arbitrary state
#[tracing::instrument(skip(state, body))]
pub async fn translate<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Json(body): Json<TranslateRequest>,
) -> impl IntoResponse {
    let snapshot = match StateSnapshot::builder()
        .user_id("_anonymous")
        .source(body.source.into())
        .confidence(body.confidence)
        .axes(body.axes.into_iter())
        .build()
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("VALIDATION_ERROR", &e.to_string())),
            )
                .into_response();
        }
    };

    let context = state.translator.to_prompt_context(&snapshot);
    Json(context).into_response()
}

/// GET /health - Health check
#[tracing::instrument(skip(state))]
pub async fn health<S: StateStore + HealthCheck + 'static>(
    State(state): State<Arc<AppState<S>>>,
) -> impl IntoResponse {
    let store_health = state.store.check().await;
    let uptime = state.start_time.elapsed().as_secs();

    let status = HealthStatus::from_checks(vec![store_health], uptime);

    let status_code = match status.status {
        HealthState::Healthy => StatusCode::OK,
        HealthState::Degraded => StatusCode::OK,
        HealthState::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(status))
}

/// GET /ready - Readiness check
#[tracing::instrument(skip(state))]
pub async fn ready<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
) -> impl IntoResponse {
    match state.store.health_check().await {
        Ok(true) => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Response for prompt context.
#[derive(Debug, Serialize)]
pub struct ContextResponse {
    /// Behavioral guidelines for the LLM.
    pub guidelines: Vec<String>,
    /// Suggested tone.
    pub tone: String,
    /// Desired response verbosity.
    pub verbosity: String,
    /// Active flags for special conditions.
    pub flags: Vec<String>,
}

impl From<PromptContext> for ContextResponse {
    fn from(c: PromptContext) -> Self {
        Self {
            guidelines: c.guidelines,
            tone: c.tone,
            verbosity: format!("{:?}", c.verbosity).to_lowercase(),
            flags: c.flags,
        }
    }
}

// ============================================================================
// Inference endpoint (requires "inference" feature)
// ============================================================================

/// Request body for inference endpoint.
#[cfg(feature = "inference")]
#[derive(Debug, Deserialize)]
pub struct InferRequest {
    /// The message text to analyze.
    pub message: String,
    /// Optional user ID for baseline comparison.
    /// If provided, the user's baseline will be updated.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Include debug feature information in response.
    #[serde(default)]
    pub include_features: bool,
}

/// A single axis estimate in the inference response.
#[cfg(feature = "inference")]
#[derive(Debug, Serialize)]
pub struct InferEstimate {
    /// The axis name.
    pub axis: String,
    /// Estimated value in [0.0, 1.0].
    pub value: f32,
    /// Confidence in this estimate.
    pub confidence: f32,
    /// Source of this inference.
    pub source: InferSourceResponse,
}

/// Inference source for API response.
#[cfg(feature = "inference")]
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InferSourceResponse {
    /// Inferred from linguistic features.
    Linguistic {
        /// Features that contributed to this inference.
        features_used: Vec<String>,
    },
    /// Inferred from deviation from baseline.
    Delta {
        /// Z-score deviation from baseline.
        z_score: f32,
        /// Which metric showed deviation.
        metric: String,
    },
    /// Combined from multiple sources.
    Combined {
        /// Number of sources combined.
        source_count: usize,
    },
    /// Prior/default value.
    Prior {
        /// Reason for this prior.
        reason: String,
    },
}

#[cfg(feature = "inference")]
impl From<&InferenceSource> for InferSourceResponse {
    fn from(source: &InferenceSource) -> Self {
        match source {
            InferenceSource::Linguistic { features_used, .. } => InferSourceResponse::Linguistic {
                features_used: features_used.clone(),
            },
            InferenceSource::Delta {
                z_score, metric, ..
            } => InferSourceResponse::Delta {
                z_score: *z_score,
                metric: metric.clone(),
            },
            InferenceSource::Combined { sources, .. } => InferSourceResponse::Combined {
                source_count: sources.len(),
            },
            InferenceSource::Prior { reason } => InferSourceResponse::Prior {
                reason: reason.clone(),
            },
            InferenceSource::Decayed { original, .. } => {
                // Unwrap to original source
                InferSourceResponse::from(original.as_ref())
            }
            InferenceSource::SelfReport => {
                // Shouldn't happen in inference, but handle gracefully
                InferSourceResponse::Prior {
                    reason: "self_report".into(),
                }
            }
        }
    }
}

/// Response for inference endpoint.
#[cfg(feature = "inference")]
#[derive(Debug, Serialize)]
pub struct InferResponse {
    /// Estimated axes.
    pub estimates: Vec<InferEstimate>,
    /// Debug feature information (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, serde_json::Value>>,
}

/// POST /v1/infer - Infer axes from message text without storage
#[cfg(feature = "inference")]
#[tracing::instrument(skip(state, body))]
pub async fn infer<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Json(body): Json<InferRequest>,
) -> impl IntoResponse {
    let Some(engine) = &state.inference_engine else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "INFERENCE_DISABLED",
                "Inference is not enabled on this server",
            )),
        )
            .into_response();
    };

    // Run inference with optional baseline
    let inferred = if let Some(user_id) = &body.user_id {
        let mut baseline_ref = state
            .baselines
            .entry(user_id.clone())
            .or_insert_with(|| engine.new_baseline());
        engine.infer_with_baseline(&body.message, &mut baseline_ref, None)
    } else {
        engine.infer(&body.message)
    };

    // Convert to response format
    let estimates: Vec<InferEstimate> = inferred
        .all()
        .map(|est| InferEstimate {
            axis: est.axis.clone(),
            value: est.value,
            confidence: est.confidence,
            source: InferSourceResponse::from(&est.source),
        })
        .collect();

    // Include features if requested
    let features = if body.include_features {
        let extractor = attuned_infer::LinguisticExtractor::new();
        let f = extractor.extract(&body.message);
        let mut map = HashMap::new();
        map.insert("word_count".into(), serde_json::json!(f.word_count));
        map.insert("sentence_count".into(), serde_json::json!(f.sentence_count));
        map.insert("hedge_count".into(), serde_json::json!(f.hedge_count));
        map.insert(
            "urgency_word_count".into(),
            serde_json::json!(f.urgency_word_count),
        );
        map.insert(
            "negative_emotion_count".into(),
            serde_json::json!(f.negative_emotion_count),
        );
        map.insert(
            "exclamation_ratio".into(),
            serde_json::json!(f.exclamation_ratio),
        );
        map.insert("question_ratio".into(), serde_json::json!(f.question_ratio));
        map.insert("caps_ratio".into(), serde_json::json!(f.caps_ratio));
        map.insert(
            "first_person_ratio".into(),
            serde_json::json!(f.first_person_ratio),
        );
        Some(map)
    } else {
        None
    };

    Json(InferResponse {
        estimates,
        features,
    })
    .into_response()
}

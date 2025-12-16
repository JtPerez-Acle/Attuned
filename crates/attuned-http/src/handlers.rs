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

/// Application state shared across handlers.
pub struct AppState<S: StateStore> {
    /// The state store backend.
    pub store: Arc<S>,
    /// The translator for converting state to context.
    pub translator: Arc<dyn Translator>,
    /// Server start time for uptime calculation.
    pub start_time: Instant,
}

impl<S: StateStore> AppState<S> {
    /// Create new application state.
    pub fn new(store: S) -> Self {
        Self {
            store: Arc::new(store),
            translator: Arc::new(RuleTranslator::default()),
            start_time: Instant::now(),
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
pub async fn upsert_state<S: StateStore + 'static>(
    State(state): State<Arc<AppState<S>>>,
    Json(body): Json<UpsertStateRequest>,
) -> impl IntoResponse {
    let snapshot = match StateSnapshot::builder()
        .user_id(&body.user_id)
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

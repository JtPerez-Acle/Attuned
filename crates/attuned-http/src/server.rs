//! HTTP server implementation.

use crate::config::ServerConfig;
use crate::error::HttpError;
use crate::handlers::{
    delete_state, get_context, get_state, health, ready, translate, upsert_state, AppState,
};
use crate::middleware::security_headers;
use attuned_core::HealthCheck;
use attuned_store::StateStore;
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[cfg(feature = "inference")]
use crate::handlers::infer;

/// HTTP server for the Attuned API.
pub struct Server<S: StateStore + HealthCheck> {
    state: Arc<AppState<S>>,
    config: ServerConfig,
}

impl<S: StateStore + HealthCheck + 'static> Server<S> {
    /// Create a new server with the given store and configuration.
    pub fn new(store: S, config: ServerConfig) -> Self {
        #[cfg(feature = "inference")]
        let state = if config.enable_inference {
            Arc::new(AppState::with_inference(
                store,
                config.inference_config.clone(),
            ))
        } else {
            Arc::new(AppState::new(store))
        };
        #[cfg(not(feature = "inference"))]
        let state = Arc::new(AppState::new(store));

        Self { state, config }
    }

    /// Build the router with all routes.
    pub fn router(&self) -> Router {
        // Build routes with typed state
        let typed_router = Router::new()
            // State management
            .route("/v1/state", post(upsert_state::<S>))
            .route("/v1/state/{user_id}", get(get_state::<S>))
            .route("/v1/state/{user_id}", delete(delete_state::<S>))
            // Context/translation
            .route("/v1/context/{user_id}", get(get_context::<S>))
            .route("/v1/translate", post(translate::<S>))
            // Operations
            .route("/health", get(health::<S>))
            .route("/ready", get(ready::<S>));

        // Add inference endpoint if feature enabled
        #[cfg(feature = "inference")]
        let typed_router = typed_router.route("/v1/infer", post(infer::<S>));

        // Apply state and convert to Router<()>
        let mut router = typed_router.with_state(self.state.clone());

        // Add security headers middleware (outermost layer, runs last on request, first on response)
        if self.config.security_headers {
            router = router.layer(middleware::from_fn(security_headers));
        }

        // Add tracing
        router = router.layer(TraceLayer::new_for_http());

        router
    }

    /// Run the server until shutdown.
    pub async fn run(self) -> Result<(), HttpError> {
        let app = self.router();

        tracing::info!(
            addr = %self.config.bind_addr,
            security_headers = %self.config.security_headers,
            auth_enabled = %self.config.auth.is_enabled(),
            rate_limit = %self.config.rate_limit.max_requests,
            "starting HTTP server"
        );

        let listener = tokio::net::TcpListener::bind(&self.config.bind_addr)
            .await
            .map_err(|e| HttpError::Bind {
                addr: self.config.bind_addr.to_string(),
                message: e.to_string(),
            })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| HttpError::Request(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use attuned_store::MemoryStore;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_server() -> Server<MemoryStore> {
        let store = MemoryStore::default();
        let config = ServerConfig::default();
        Server::new(store, config)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = test_server();
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let server = test_server();
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let server = test_server();
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/state/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_upsert_and_get_state() {
        let server = test_server();
        let app = server.router();

        // Upsert state
        let body = r#"{"user_id": "test_user", "axes": {"warmth": 0.7}}"#;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/state")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Get state
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/state/test_user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let server = test_server();
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify security headers
        let headers = response.headers();
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
        assert!(headers.get("content-security-policy").is_some());
        assert_eq!(headers.get("cache-control").unwrap(), "no-store, max-age=0");
    }
}

//! Server configuration.

use crate::middleware::{AuthConfig, RateLimitConfig};
use std::net::SocketAddr;
use std::time::Duration;

#[cfg(feature = "inference")]
use attuned_infer::InferenceConfig;

/// Configuration for the HTTP server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Address to bind the server to.
    /// Default: 127.0.0.1:8080 (localhost only for security)
    pub bind_addr: SocketAddr,

    /// Request timeout.
    /// Default: 30 seconds
    pub request_timeout: Duration,

    /// Maximum request body size in bytes.
    /// Default: 1MB
    pub body_limit: usize,

    /// CORS allowed origins.
    /// Default: empty (CORS disabled)
    pub cors_origins: Vec<String>,

    /// API key authentication configuration.
    /// Default: disabled (no keys configured)
    pub auth: AuthConfig,

    /// Rate limiting configuration.
    /// Default: 100 requests per minute per IP
    pub rate_limit: RateLimitConfig,

    /// Enable security headers.
    /// Default: true
    pub security_headers: bool,

    /// Enable automatic inference from message text.
    /// Default: false (requires "inference" feature)
    #[cfg(feature = "inference")]
    pub enable_inference: bool,

    /// Inference engine configuration.
    /// Default: None (uses InferenceConfig::default())
    #[cfg(feature = "inference")]
    pub inference_config: Option<InferenceConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            // Secure default: localhost only
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            request_timeout: Duration::from_secs(30),
            body_limit: 1024 * 1024, // 1MB
            cors_origins: vec![],
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            security_headers: true,
            #[cfg(feature = "inference")]
            enable_inference: false,
            #[cfg(feature = "inference")]
            inference_config: None,
        }
    }
}

impl ServerConfig {
    /// Create a config with API key authentication enabled.
    pub fn with_api_keys(mut self, keys: impl IntoIterator<Item = String>) -> Self {
        self.auth = AuthConfig::with_keys(keys);
        self
    }

    /// Disable rate limiting.
    pub fn without_rate_limit(mut self) -> Self {
        self.rate_limit.max_requests = u32::MAX;
        self
    }

    /// Set custom rate limit.
    pub fn with_rate_limit(mut self, max_requests: u32, window_secs: u64) -> Self {
        self.rate_limit.max_requests = max_requests;
        self.rate_limit.window = Duration::from_secs(window_secs);
        self
    }

    /// Enable inference from message text.
    #[cfg(feature = "inference")]
    pub fn with_inference(mut self) -> Self {
        self.enable_inference = true;
        self
    }

    /// Enable inference with custom configuration.
    #[cfg(feature = "inference")]
    pub fn with_inference_config(mut self, config: InferenceConfig) -> Self {
        self.enable_inference = true;
        self.inference_config = Some(config);
        self
    }
}

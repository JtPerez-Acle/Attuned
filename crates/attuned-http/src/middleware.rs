//! HTTP middleware for security, rate limiting, and authentication.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// Security Headers Middleware
// ============================================================================

/// Add security headers to all responses.
///
/// Headers added:
/// - `X-Content-Type-Options: nosniff` - Prevent MIME sniffing
/// - `X-Frame-Options: DENY` - Prevent clickjacking
/// - `X-XSS-Protection: 1; mode=block` - Legacy XSS protection
/// - `Content-Security-Policy: default-src 'none'` - Strict CSP
/// - `Cache-Control: no-store` - Prevent caching of sensitive data
/// - `Referrer-Policy: strict-origin-when-cross-origin` - Control referrer info
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // Legacy XSS protection (still useful for older browsers)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Strict Content Security Policy (API-only, no inline content)
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );

    // Prevent caching of potentially sensitive responses
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );

    // Control referrer information
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions policy (disable all browser features)
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), camera=(), microphone=()"),
    );

    response
}

// ============================================================================
// API Key Authentication Middleware
// ============================================================================

/// Configuration for API key authentication.
#[derive(Clone, Debug)]
pub struct AuthConfig {
    /// Set of valid API keys.
    pub api_keys: HashSet<String>,
    /// Header name for API key (default: "Authorization").
    pub header_name: String,
    /// Prefix expected before the key (default: "Bearer ").
    pub prefix: String,
    /// Paths that don't require authentication.
    pub public_paths: HashSet<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_keys: HashSet::new(),
            header_name: "Authorization".to_string(),
            prefix: "Bearer ".to_string(),
            public_paths: ["/health", "/ready"].iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl AuthConfig {
    /// Create a new auth config with the given API keys.
    pub fn with_keys(keys: impl IntoIterator<Item = String>) -> Self {
        Self {
            api_keys: keys.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Add a public path that doesn't require authentication.
    pub fn add_public_path(mut self, path: impl Into<String>) -> Self {
        self.public_paths.insert(path.into());
        self
    }

    /// Check if authentication is required for a path.
    pub fn requires_auth(&self, path: &str) -> bool {
        !self.public_paths.contains(path)
    }

    /// Validate an API key.
    pub fn validate_key(&self, key: &str) -> bool {
        self.api_keys.contains(key)
    }

    /// Check if authentication is enabled (has any API keys configured).
    pub fn is_enabled(&self) -> bool {
        !self.api_keys.is_empty()
    }
}

/// State for authentication middleware.
#[derive(Clone)]
pub struct AuthState {
    /// The authentication configuration.
    pub config: Arc<AuthConfig>,
}

/// API key authentication middleware.
pub async fn api_key_auth(
    State(auth): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // Skip auth for public paths
    if !auth.config.requires_auth(path) {
        return Ok(next.run(request).await);
    }

    // Skip auth if not enabled (no keys configured)
    if !auth.config.is_enabled() {
        return Ok(next.run(request).await);
    }

    // Extract API key from header
    let auth_header = request
        .headers()
        .get(&auth.config.header_name)
        .and_then(|v| v.to_str().ok());

    let api_key = match auth_header {
        Some(value) if value.starts_with(&auth.config.prefix) => {
            &value[auth.config.prefix.len()..]
        }
        Some(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Invalid authorization header format",
            )
                .into_response());
        }
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Missing authorization header",
            )
                .into_response());
        }
    };

    // Validate the key
    if !auth.config.validate_key(api_key) {
        tracing::warn!(
            path = %path,
            "Invalid API key attempt"
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "Invalid API key",
        )
            .into_response());
    }

    Ok(next.run(request).await)
}

// ============================================================================
// Rate Limiting Middleware
// ============================================================================

/// Rate limiting strategy.
#[derive(Clone, Debug, Default)]
pub enum RateLimitKey {
    /// Rate limit by client IP address.
    #[default]
    ByIp,
    /// Rate limit by API key.
    ByApiKey,
}

/// Configuration for rate limiting.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per window.
    pub max_requests: u32,
    /// Time window duration.
    pub window: Duration,
    /// How to identify clients for rate limiting.
    pub key_strategy: RateLimitKey,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            key_strategy: RateLimitKey::ByIp,
        }
    }
}

/// Entry in the rate limit store.
#[derive(Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// State for rate limiting middleware.
#[derive(Clone)]
pub struct RateLimitState {
    /// The rate limiting configuration.
    pub config: Arc<RateLimitConfig>,
    entries: Arc<RwLock<std::collections::HashMap<String, RateLimitEntry>>>,
}

impl RateLimitState {
    /// Create a new rate limit state.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config: Arc::new(config),
            entries: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Clean up expired entries.
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window = self.config.window;
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| now.duration_since(entry.window_start) < window);
    }

    /// Check and increment rate limit for a key.
    async fn check_and_increment(&self, key: String) -> Result<(u32, u32), (u32, Duration)> {
        let now = Instant::now();
        let mut entries = self.entries.write().await;

        let entry = entries.entry(key).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Reset if window has passed
        if now.duration_since(entry.window_start) >= self.config.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        if entry.count > self.config.max_requests {
            let retry_after = self.config.window - now.duration_since(entry.window_start);
            Err((entry.count, retry_after))
        } else {
            Ok((self.config.max_requests - entry.count, self.config.max_requests))
        }
    }
}

/// Rate limiting middleware.
pub async fn rate_limit(
    State(state): State<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let key = match state.config.key_strategy {
        RateLimitKey::ByIp => addr.ip().to_string(),
        RateLimitKey::ByApiKey => {
            // Extract API key from Authorization header
            request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_start_matches("Bearer ").to_string())
                .unwrap_or_else(|| addr.ip().to_string())
        }
    };

    match state.check_and_increment(key).await {
        Ok((remaining, limit)) => {
            let mut response = next.run(request).await;
            let headers = response.headers_mut();

            // Add rate limit headers
            headers.insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string()).unwrap(),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&remaining.to_string()).unwrap(),
            );

            Ok(response)
        }
        Err((_, retry_after)) => {
            let retry_secs = retry_after.as_secs().max(1);
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                [
                    ("Retry-After", retry_secs.to_string()),
                    ("X-RateLimit-Limit", state.config.max_requests.to_string()),
                    ("X-RateLimit-Remaining", "0".to_string()),
                ],
                "Rate limit exceeded",
            )
                .into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(!config.is_enabled());
        assert!(config.public_paths.contains("/health"));
        assert!(config.public_paths.contains("/ready"));
    }

    #[test]
    fn test_auth_config_with_keys() {
        let config = AuthConfig::with_keys(["key1".to_string(), "key2".to_string()]);
        assert!(config.is_enabled());
        assert!(config.validate_key("key1"));
        assert!(config.validate_key("key2"));
        assert!(!config.validate_key("key3"));
    }

    #[test]
    fn test_auth_config_public_paths() {
        let config = AuthConfig::default().add_public_path("/metrics");
        assert!(!config.requires_auth("/health"));
        assert!(!config.requires_auth("/ready"));
        assert!(!config.requires_auth("/metrics"));
        assert!(config.requires_auth("/v1/state"));
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_rate_limit_state() {
        let state = RateLimitState::new(RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
            key_strategy: RateLimitKey::ByIp,
        });

        // First 3 requests should succeed
        assert!(state.check_and_increment("test".to_string()).await.is_ok());
        assert!(state.check_and_increment("test".to_string()).await.is_ok());
        assert!(state.check_and_increment("test".to_string()).await.is_ok());

        // 4th should fail
        assert!(state.check_and_increment("test".to_string()).await.is_err());

        // Different key should succeed
        assert!(state.check_and_increment("other".to_string()).await.is_ok());
    }
}

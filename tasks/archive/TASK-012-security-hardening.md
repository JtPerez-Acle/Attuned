# TASK-012: Security Hardening

## Status
- [ ] Not Started
- [ ] In Progress
- [x] Completed
- [ ] Blocked

**Priority**: High
**Created**: 2025-12-16
**Last Updated**: 2025-12-16
**Phase**: 4 - Quality & Testing
**Depends On**: TASK-002, TASK-003, TASK-006
**Blocks**: TASK-011 (Release)

## Task Description
Implement security best practices across all Attuned components. Security is critical given that Attuned handles user state data.

## Requirements
1. Input validation on all boundaries
2. Secure defaults
3. Authentication mechanisms
4. Rate limiting
5. Audit logging
6. Security documentation

## Security Measures

### 1. Input Validation

#### Axis Values
```rust
impl StateSnapshot {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for (name, value) in &self.axes {
            if *value < 0.0 || *value > 1.0 {
                return Err(ValidationError::AxisOutOfRange {
                    axis: name.clone(),
                    value: *value,
                });
            }
            if !is_valid_axis_name(name) {
                return Err(ValidationError::InvalidAxisName {
                    axis: name.clone(),
                });
            }
        }
        Ok(())
    }
}
```

#### User IDs
```rust
fn validate_user_id(id: &str) -> Result<(), ValidationError> {
    // Max length
    if id.len() > 256 {
        return Err(ValidationError::UserIdTooLong);
    }
    // Allowed characters (alphanumeric + underscore + hyphen)
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::InvalidUserIdChars);
    }
    Ok(())
}
```

### 2. HTTP Security

#### Headers
```rust
async fn security_headers(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'none'".parse().unwrap());

    response
}
```

#### Rate Limiting
```rust
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub by: RateLimitKey,  // ByIp | ByApiKey | ByUserId
}
```

#### Authentication
```rust
pub enum AuthMethod {
    /// API key in header: Authorization: Bearer <key>
    ApiKey {
        keys: HashSet<String>,
        header: String,
    },
    /// JWT validation
    Jwt {
        issuer: String,
        audience: String,
        jwks_url: String,
    },
}
```

### 3. Data Protection

#### No PII in Logs
```rust
// Redact user_id in production logs
impl fmt::Debug for StateSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateSnapshot")
            .field("user_id", &redact(&self.user_id))
            .field("updated_at_unix_ms", &self.updated_at_unix_ms)
            // ...
            .finish()
    }
}
```

#### Encryption at Rest
- Document Qdrant encryption options
- Document database-level encryption
- Recommend for production deployments

### 4. Dependency Security

#### Audit Configuration
```toml
# .cargo/audit.toml
[advisories]
ignore = []
informational_warnings = ["unmaintained"]

[licenses]
unlicensed = "deny"
deny = ["GPL-3.0"]
```

#### Supply Chain
- Pin dependency versions in Cargo.lock
- Review new dependencies before adding
- Use `cargo-deny` for license compliance

### 5. Secure Defaults

```rust
impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),  // Localhost only
            request_timeout: Duration::from_secs(30),
            body_limit: 1024 * 1024,  // 1MB
            cors_origins: vec![],  // No CORS by default
            auth: AuthConfig::ApiKey { .. },  // Auth required
            tls: None,  // Document TLS recommendation
        }
    }
}
```

### 6. Security Documentation

Create `SECURITY.md`:
- Supported versions
- How to report vulnerabilities
- Security design principles
- Deployment recommendations

## Threat Model

| Threat | Mitigation |
|--------|------------|
| Axis value injection | Strict validation [0.0, 1.0] |
| User ID enumeration | Rate limiting, auth required |
| Unauthorized access | API key / JWT auth |
| Data exposure in logs | PII redaction |
| Dependency vulnerabilities | Automated auditing |
| DoS via large payloads | Body size limits |

## Acceptance Criteria
- [x] All inputs validated
- [x] Rate limiting implemented
- [x] Authentication working
- [x] Security headers on all responses
- [x] PII redacted from logs
- [x] cargo-audit passing
- [x] SECURITY.md created
- [x] Security review documented

## Progress Log
- 2025-12-16: Task created
- 2025-12-16: Task completed
  - Input validation already existed (axis values, user IDs, axis names, confidence)
  - Added security headers middleware (`crates/attuned-http/src/middleware.rs`)
  - Implemented rate limiting infrastructure (`RateLimitConfig`, `RateLimitState`)
  - Implemented API key authentication infrastructure (`AuthConfig`, `AuthState`)
  - Added PII redaction to `StateSnapshot::Debug` implementation
  - Configured cargo-audit (`.cargo/audit.toml`)
  - Created SECURITY.md with full documentation
  - All 47 tests passing

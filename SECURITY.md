# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Attuned, please report it responsibly:

1. **Do NOT open a public GitHub issue** for security vulnerabilities
2. Email security concerns to: [security@attuned-dev.org](mailto:security@attuned-dev.org)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to understand and address the issue.

## Security Design Principles

Attuned is designed with security as a core consideration:

### 1. No Action Execution

Attuned produces **context**, not actions. It never:
- Sends messages on behalf of users
- Schedules events or triggers automations
- Makes API calls to external services
- Moves money or executes transactions

### 2. Data Minimization

- **No content storage**: Attuned stores state descriptors, not personal content or message history
- **PII redaction**: User IDs are automatically redacted in log output
- **Bounded data**: All axis values are normalized to [0.0, 1.0]

### 3. User Agency

- Self-report always overrides inference
- No hidden optimizers or persuasion algorithms
- All state is human-readable and overrideable

## Security Measures

### Input Validation

All inputs are validated at API boundaries:

| Input | Validation |
|-------|------------|
| User ID | Max 256 chars, alphanumeric + underscore + hyphen only |
| Axis values | Must be in range [0.0, 1.0] |
| Axis names | Lowercase alphanumeric + underscore, no trailing underscore |
| Confidence | Must be in range [0.0, 1.0] |
| Request body | Limited to 1MB by default |

### HTTP Security

When using `attuned-http`, the following protections are enabled by default:

**Security Headers:**
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`
- `Cache-Control: no-store, max-age=0`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: geolocation=(), camera=(), microphone=()`

**Rate Limiting:**
- Default: 100 requests per minute per IP
- Configurable via `RateLimitConfig`

**Authentication:**
- API key authentication available (disabled by default)
- Configure with `ServerConfig::with_api_keys()`

### Secure Defaults

```rust
ServerConfig::default()
// - Binds to 127.0.0.1:8080 (localhost only)
// - 30 second request timeout
// - 1MB body limit
// - Security headers enabled
// - Rate limiting: 100 req/min per IP
```

### Dependency Security

We use `cargo-audit` to monitor for vulnerabilities:

```bash
cargo audit
```

Configuration in `.cargo/audit.toml`:
- Checks for known vulnerabilities
- Flags unmaintained dependencies
- Detects yanked crates

## Deployment Recommendations

### Production Checklist

1. **TLS Termination**: Always use HTTPS in production (via reverse proxy)
2. **Authentication**: Enable API key authentication for all non-health endpoints
3. **Network Isolation**: Bind to localhost and use a reverse proxy
4. **Rate Limiting**: Adjust limits based on expected traffic
5. **Monitoring**: Enable structured logging and metrics collection
6. **Updates**: Run `cargo audit` regularly in CI/CD

### Example Secure Deployment

```rust
let config = ServerConfig::default()
    // Enable API key authentication
    .with_api_keys(["your-api-key-here".to_string()])
    // Adjust rate limits for production
    .with_rate_limit(1000, 60); // 1000 req/min

let server = Server::new(store, config);
```

### Reverse Proxy Configuration

Example nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Misuse Warnings

Attuned cannot prevent downstream misuse, but we can make expectations explicit.

### Partial Compliance Risk

**The Problem**: Systems may selectively apply Attuned context—respecting "verbosity: low" but ignoring "suggestion_tolerance: low" when it hurts conversion.

**Our Expectation**: Consume `PromptContext` atomically. If you use Attuned, apply all guidelines, not just the convenient ones.

**Recommendation**: Log which context fields were applied for audit purposes.

### Vulnerability Targeting Risk

**The Problem**: Axis combinations like high `cognitive_load` + high `anxiety_level` could identify vulnerable users for targeting.

**Our Expectation**: These axes exist to *protect* users (simplify responses, add reassurance), not to *exploit* them.

**What We Do**:
- Every axis has explicit `forbidden_uses` in its definition
- Axes describe state, not exploitability
- The manifesto explicitly prohibits persuasion optimization

**What You Must Do**:
- Never use state signals to identify "easy marks"
- Never time offers or actions to moments of user vulnerability
- Implement your own vulnerability protections if needed

### Axis Misinterpretation Risk

**The Problem**: Axis meanings could drift or be misunderstood across implementations.

**Our Expectation**: Read the `AxisDefinition` for each axis you use. Pay attention to:
- `intent`: What the axis is *for*
- `forbidden_uses`: What it must *never* be used for
- `low_anchor` / `high_anchor`: What values actually mean

**Example**:
```rust
// WRONG: Assuming low tolerance_for_complexity = low intelligence
// RIGHT: Low tolerance = user currently prefers simplicity (temporary state)

// WRONG: Using high urgency_sensitivity to pressure decisions
// RIGHT: Using it to skip non-essential content and respect user's time
```

### Inference Override Risk

**The Problem**: Systems might "correct" user self-report with their own inference.

**Our Expectation**: Self-report is sovereign. If a user explicitly sets an axis value, that value is truth. Your inference is a guess.

**Implementation**:
```rust
// Attuned supports Source::SelfReport vs Source::Inferred
// Self-report has implicit priority
let snapshot = StateSnapshot::builder()
    .source(Source::SelfReport)  // This is authoritative
    .confidence(1.0)
    .axis("warmth", 0.3)  // User says they want cool/professional
    .build()?;

// Don't override this with "but they seem friendly in chat"
```

## Threat Model

| Threat | Mitigation |
|--------|------------|
| Axis value injection | Strict validation [0.0, 1.0] |
| User ID enumeration | Rate limiting, authentication |
| Unauthorized access | API key authentication |
| Data exposure in logs | PII redaction in Debug impl |
| Dependency vulnerabilities | Automated cargo-audit in CI |
| DoS via large payloads | Body size limits (1MB default) |
| XSS/Clickjacking | Strict security headers |
| Cache poisoning | No-store cache control |

## Security Updates

Security updates are released as patch versions. We recommend:

1. Subscribe to GitHub releases
2. Run `cargo update` regularly
3. Monitor `cargo audit` output in CI

## Acknowledgments

We appreciate responsible disclosure of security issues. Contributors who report valid security vulnerabilities will be acknowledged here (with permission).

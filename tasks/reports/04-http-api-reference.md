# HTTP API Reference

## Base URL

```
http://127.0.0.1:8080
```

## Authentication

Optional API key authentication via Bearer token:

```bash
curl -H "Authorization: Bearer your-api-key" http://localhost:8080/v1/state/user123
```

Configure in `ServerConfig`:
```rust
let config = ServerConfig::default()
    .with_api_keys(vec!["key1".into(), "key2".into()]);
```

## Endpoints

### POST /v1/state

Upsert state for a user (patch semantics - only provided axes are updated).

**Request Body**
```json
{
  "user_id": "user123",
  "source": "self_report",
  "confidence": 1.0,
  "axes": {
    "warmth": 0.7,
    "anxiety_level": 0.3,
    "cognitive_load": 0.5
  },
  "message": "Optional message text for inference"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `user_id` | string | Yes | Unique user identifier |
| `source` | enum | No | `self_report` (default), `inferred`, `mixed` |
| `confidence` | float | No | 0.0-1.0, default 1.0 |
| `axes` | object | Yes | Axis name → value (0.0-1.0) |
| `message` | string | No | Message text for inference (requires `inference` feature) |

**Response**
- `204 No Content` - Success
- `400 Bad Request` - Validation error

**Example**
```bash
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user123",
    "axes": {"warmth": 0.7, "formality": 0.3}
  }'
```

---

### GET /v1/state/{user_id}

Retrieve the latest state for a user.

**Response**
```json
{
  "user_id": "user123",
  "updated_at_unix_ms": 1702756800000,
  "source": "self_report",
  "confidence": 1.0,
  "axes": {
    "warmth": 0.7,
    "anxiety_level": 0.3,
    "cognitive_load": 0.5
  }
}
```

**Status Codes**
- `200 OK` - State found
- `404 Not Found` - No state for user

**Example**
```bash
curl http://localhost:8080/v1/state/user123
```

---

### DELETE /v1/state/{user_id}

Delete all state for a user (GDPR compliance).

**Response**
- `204 No Content` - Deleted (or already absent)
- `500 Internal Server Error` - Storage error

**Example**
```bash
curl -X DELETE http://localhost:8080/v1/state/user123
```

---

### GET /v1/context/{user_id}

Get translated PromptContext for conditioning an LLM.

**Response**
```json
{
  "guidelines": [
    "Use warm, supportive language",
    "Keep responses concise and clear",
    "Acknowledge concerns before providing solutions"
  ],
  "tone": "warm and supportive",
  "verbosity": "concise",
  "flags": [
    "needs_reassurance",
    "prefers_warmth"
  ]
}
```

**Status Codes**
- `200 OK` - Context generated
- `404 Not Found` - No state for user

**Example**
```bash
curl http://localhost:8080/v1/context/user123
```

**Using with LLMs**
```python
# Fetch context
context = requests.get(f"{BASE_URL}/v1/context/{user_id}").json()

# Build system prompt
system_prompt = f"""
You are a helpful assistant. Follow these guidelines:
{chr(10).join(f'- {g}' for g in context['guidelines'])}

Tone: {context['tone']}
Verbosity: {context['verbosity']}
"""
```

---

### POST /v1/translate

Translate arbitrary state to context without storage.

**Request Body**
```json
{
  "axes": {
    "cognitive_load": 0.8,
    "warmth": 0.6
  },
  "source": "inferred",
  "confidence": 0.7
}
```

**Response**
Same as `/v1/context/{user_id}`

**Example**
```bash
curl -X POST http://localhost:8080/v1/translate \
  -H "Content-Type: application/json" \
  -d '{"axes": {"cognitive_load": 0.8, "warmth": 0.6}}'
```

---

### POST /v1/infer

*Requires `inference` feature*

Infer axis values from message text without storing state.

**Request Body**
```json
{
  "message": "I'm really worried about this deadline!!!",
  "user_id": "user123",
  "include_features": true
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message` | string | Yes | Text to analyze |
| `user_id` | string | No | For baseline comparison |
| `include_features` | bool | No | Include debug feature info |

**Response**
```json
{
  "estimates": [
    {
      "axis": "anxiety_level",
      "value": 0.72,
      "confidence": 0.58,
      "source": {
        "type": "linguistic",
        "features_used": ["anxiety_score", "negative_emotion_density"]
      }
    },
    {
      "axis": "urgency_sensitivity",
      "value": 0.81,
      "confidence": 0.45,
      "source": {
        "type": "linguistic",
        "features_used": ["urgency_score"]
      }
    }
  ],
  "features": {
    "word_count": 7,
    "negative_emotion_count": 1,
    "urgency_word_count": 1,
    "exclamation_ratio": 0.43,
    "first_person_ratio": 0.14
  }
}
```

**Source Types**
```json
// Linguistic inference
{"type": "linguistic", "features_used": ["hedge_density", "formality_score"]}

// Baseline deviation
{"type": "delta", "z_score": 2.1, "metric": "urgency"}

// Combined sources
{"type": "combined", "source_count": 2}

// Prior/default
{"type": "prior", "reason": "no observation"}
```

**Example**
```bash
curl -X POST http://localhost:8080/v1/infer \
  -H "Content-Type: application/json" \
  -d '{"message": "I need help ASAP!!!", "include_features": true}'
```

---

### GET /health

Health check endpoint.

**Response**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "checks": [
    {
      "name": "store",
      "status": "healthy",
      "latency_ms": 1
    }
  ]
}
```

**Status Codes**
- `200 OK` - Healthy or degraded
- `503 Service Unavailable` - Unhealthy

---

### GET /ready

Readiness check for load balancers.

**Response**
- `200 OK` - Ready to serve
- `503 Service Unavailable` - Not ready

---

## Error Response Format

All errors return a consistent format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "axis value out of range: warmth=1.5",
    "request_id": "abc-123"
  }
}
```

**Error Codes**
| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid request body |
| `USER_NOT_FOUND` | 404 | No state for user_id |
| `STORE_ERROR` | 500 | Storage backend failure |
| `INFERENCE_DISABLED` | 503 | Inference not enabled |

---

## Rate Limiting

Default: 100 requests per minute per IP.

Configure in `ServerConfig`:
```rust
let config = ServerConfig::default()
    .with_rate_limit(200, 60);  // 200 requests per 60 seconds
```

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1702756860
```

---

## Security Headers

Default security headers (can be disabled):
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'none'
Cache-Control: no-store, max-age=0
```

---

## Server Configuration

```rust
use attuned_http::{Server, ServerConfig};
use attuned_store::MemoryStore;

let config = ServerConfig {
    bind_addr: "0.0.0.0:8080".parse().unwrap(),
    request_timeout: Duration::from_secs(30),
    body_limit: 1024 * 1024,  // 1MB
    cors_origins: vec!["https://app.example.com".into()],
    auth: AuthConfig::with_keys(vec!["secret-key".into()]),
    rate_limit: RateLimitConfig { max_requests: 100, window: Duration::from_secs(60) },
    security_headers: true,
    // Inference (requires "inference" feature)
    enable_inference: true,
    inference_config: None,  // Uses defaults
};

let server = Server::new(MemoryStore::default(), config);
server.run().await?;
```

---

## cURL Examples

**Full workflow:**
```bash
# 1. Set initial state
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice",
    "axes": {"warmth": 0.8, "formality": 0.2}
  }'

# 2. Update with message inference
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "alice",
    "message": "Help! I urgently need assistance with my account!!!",
    "axes": {}
  }'

# 3. Get context for LLM
curl http://localhost:8080/v1/context/alice

# 4. Delete user data (GDPR)
curl -X DELETE http://localhost:8080/v1/state/alice
```

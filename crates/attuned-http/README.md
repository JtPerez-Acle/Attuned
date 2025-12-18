# attuned-http

HTTP reference server for [Attuned](https://github.com/JtPerez-Acle/Attuned).

## Overview

Production-ready HTTP server built with [Axum](https://github.com/tokio-rs/axum) providing:

- RESTful API for state management
- Authentication middleware
- Rate limiting
- OpenTelemetry tracing
- Prometheus metrics

## Quick Start

```rust
use attuned_http::{Server, ServerConfig, AuthConfig};
use attuned_store::MemoryStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        auth: AuthConfig::None,
        ..Default::default()
    };

    let store = Arc::new(MemoryStore::new());
    let server = Server::new(config, store);
    server.run().await?;

    Ok(())
}
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/state` | Upsert state (patch semantics) |
| GET | `/v1/state/{user_id}` | Get latest state |
| GET | `/v1/context/{user_id}` | Get translated PromptContext |
| DELETE | `/v1/state/{user_id}` | Delete state (GDPR) |
| POST | `/v1/translate` | Translate arbitrary state |
| GET | `/health` | Health check |
| GET | `/ready` | Readiness check |
| GET | `/metrics` | Prometheus metrics |

## Example Requests

```bash
# Set user state
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user_123", "axes": {"cognitive_load": 0.8}}'

# Get context for LLM
curl http://localhost:8080/v1/context/user_123
```

## Authentication

```rust
// API Key auth
AuthConfig::ApiKey { key: "secret".to_string() }

// Bearer token
AuthConfig::Bearer { secret: "jwt-secret".to_string() }

// No auth (development only)
AuthConfig::None
```

## License

Apache-2.0

# attuned-cli

CLI tool for [Attuned](https://github.com/JtPerez-Acle/Attuned) development and testing.

## Installation

```bash
cargo install attuned-cli
```

## Commands

### Start Server

```bash
# Start HTTP server on default port (8080)
attuned serve

# Custom port
attuned serve --port 3000

# With authentication
attuned serve --auth-key "your-api-key"
```

### Translate State

```bash
# Translate state to prompt context
attuned translate --cognitive-load 0.8 --stress-level 0.6

# Output as JSON
attuned translate --cognitive-load 0.8 --json
```

### List Axes

```bash
# Show all canonical axes
attuned axes

# Show specific axis details
attuned axes cognitive_load
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ATTUNED_HOST` | Server bind address | `127.0.0.1` |
| `ATTUNED_PORT` | Server port | `8080` |
| `ATTUNED_LOG` | Log level | `info` |
| `ATTUNED_AUTH_KEY` | API key for auth | None |

## Example Usage

```bash
# Start server in background
attuned serve &

# Set user state
curl -X POST http://localhost:8080/v1/state \
  -H "Content-Type: application/json" \
  -d '{"user_id": "test", "axes": {"cognitive_load": 0.9}}'

# Get context
curl http://localhost:8080/v1/context/test
```

## License

Apache-2.0

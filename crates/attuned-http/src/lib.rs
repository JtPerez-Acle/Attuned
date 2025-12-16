//! # attuned-http
//!
//! HTTP reference server for Attuned.
//!
//! This crate provides a ready-to-use HTTP server exposing the Attuned API.
//! It includes health checks, metrics, and OpenAPI documentation.
//!
//! ## Endpoints
//!
//! - `POST /v1/state` - Upsert state (patch semantics)
//! - `GET /v1/state/{user_id}` - Get latest state
//! - `GET /v1/context/{user_id}` - Get PromptContext
//! - `DELETE /v1/state/{user_id}` - Delete state
//! - `GET /health` - Health check
//! - `GET /metrics` - Prometheus metrics
//!
//! ## Example
//!
//! ```rust,ignore
//! use attuned_http::{Server, ServerConfig};
//! use attuned_store::MemoryStore;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = MemoryStore::default();
//!     let config = ServerConfig::default();
//!
//!     let server = Server::new(store, config);
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]

// TODO: Implement HTTP server (TASK-006)

mod config;
mod error;
pub mod handlers;
pub mod middleware;
mod server;

pub use config::ServerConfig;
pub use error::HttpError;
pub use handlers::AppState;
pub use middleware::{AuthConfig, RateLimitConfig, RateLimitKey};
pub use server::Server;

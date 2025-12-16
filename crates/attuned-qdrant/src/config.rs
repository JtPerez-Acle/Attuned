//! Configuration for the Qdrant store.

use std::time::Duration;

/// Configuration for connecting to and using Qdrant.
#[derive(Clone, Debug)]
pub struct QdrantStoreConfig {
    /// Qdrant server URL (e.g., "http://localhost:6334").
    pub url: String,

    /// Name of the collection to use.
    pub collection_name: String,

    /// Optional API key for authentication.
    pub api_key: Option<String>,

    /// Whether to store historical snapshots.
    pub enable_history: bool,

    /// Number of days to retain history (None = forever).
    pub history_retention_days: Option<u32>,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Request timeout.
    pub request_timeout: Duration,
}

impl Default for QdrantStoreConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            collection_name: "attuned_state".to_string(),
            api_key: None,
            enable_history: false,
            history_retention_days: None,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
        }
    }
}

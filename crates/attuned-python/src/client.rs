//! Async HTTP client for Attuned server.

use crate::snapshot::PyStateSnapshot;
use crate::translator::PyPromptContext;
use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// HTTP client for interacting with an Attuned server.
///
/// Provides methods to upsert state, get state, get translated context,
/// and delete state via the Attuned REST API.
///
/// Example:
///     >>> client = AttunedClient("http://localhost:8080")
///     >>> client.upsert_state(snapshot)
///     >>> context = client.get_context("user_123")
///     >>> print(context.tone)
///
/// With authentication:
///     >>> client = AttunedClient("http://localhost:8080", api_key="your-key")
#[pyclass(name = "AttunedClient")]
pub struct PyAttunedClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyAttunedClient {
    /// Create a new AttunedClient.
    ///
    /// Args:
    ///     base_url: The base URL of the Attuned server (e.g., "http://localhost:8080")
    ///     api_key: Optional API key for authentication
    ///     timeout_secs: Request timeout in seconds (default: 30)
    #[new]
    #[pyo3(signature = (base_url, api_key=None, timeout_secs=30))]
    fn new(base_url: String, api_key: Option<String>, timeout_secs: u64) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create HTTP client: {}", e)))?;

        Ok(PyAttunedClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client,
            runtime: Arc::new(runtime),
        })
    }

    /// Get the base URL.
    #[getter]
    fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if the client has an API key configured.
    #[getter]
    fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Upsert (create or update) a state snapshot.
    ///
    /// Args:
    ///     snapshot: The StateSnapshot to upsert
    ///
    /// Raises:
    ///     ConnectionError: If the server is unreachable
    ///     RuntimeError: If the request fails
    fn upsert_state(&self, snapshot: &PyStateSnapshot) -> PyResult<()> {
        let url = format!("{}/v1/state", self.base_url);
        let body = serde_json::to_string(&snapshot.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;

        self.runtime.block_on(async {
            let mut req = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body);

            if let Some(key) = &self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| PyConnectionError::new_err(format!("Connection error: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(PyRuntimeError::new_err(format!(
                    "Server error {}: {}",
                    status.as_u16(),
                    body
                )));
            }

            Ok(())
        })
    }

    /// Get the latest state for a user.
    ///
    /// Args:
    ///     user_id: The user ID to get state for
    ///
    /// Returns:
    ///     The latest StateSnapshot, or None if not found
    ///
    /// Raises:
    ///     ConnectionError: If the server is unreachable
    ///     RuntimeError: If the request fails
    fn get_state(&self, user_id: &str) -> PyResult<Option<PyStateSnapshot>> {
        let url = format!("{}/v1/state/{}", self.base_url, user_id);

        self.runtime.block_on(async {
            let mut req = self.client.get(&url);

            if let Some(key) = &self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| PyConnectionError::new_err(format!("Connection error: {}", e)))?;

            if resp.status().as_u16() == 404 {
                return Ok(None);
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(PyRuntimeError::new_err(format!(
                    "Server error {}: {}",
                    status.as_u16(),
                    body
                )));
            }

            let body = resp
                .text()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read response: {}", e)))?;

            let snapshot: attuned_core::StateSnapshot = serde_json::from_str(&body)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(Some(PyStateSnapshot::from(snapshot)))
        })
    }

    /// Get translated context for a user.
    ///
    /// Args:
    ///     user_id: The user ID to get context for
    ///
    /// Returns:
    ///     The translated PromptContext
    ///
    /// Raises:
    ///     ConnectionError: If the server is unreachable
    ///     RuntimeError: If the request fails
    ///     ValueError: If the user has no state
    fn get_context(&self, user_id: &str) -> PyResult<PyPromptContext> {
        let url = format!("{}/v1/context/{}", self.base_url, user_id);

        self.runtime.block_on(async {
            let mut req = self.client.get(&url);

            if let Some(key) = &self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| PyConnectionError::new_err(format!("Connection error: {}", e)))?;

            if resp.status().as_u16() == 404 {
                return Err(PyValueError::new_err(format!(
                    "No state found for user: {}",
                    user_id
                )));
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(PyRuntimeError::new_err(format!(
                    "Server error {}: {}",
                    status.as_u16(),
                    body
                )));
            }

            let body = resp
                .text()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read response: {}", e)))?;

            let context: attuned_core::PromptContext = serde_json::from_str(&body)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(PyPromptContext::from(context))
        })
    }

    /// Delete state for a user (GDPR compliance).
    ///
    /// Args:
    ///     user_id: The user ID to delete state for
    ///
    /// Returns:
    ///     True if state was deleted, False if no state existed
    ///
    /// Raises:
    ///     ConnectionError: If the server is unreachable
    ///     RuntimeError: If the request fails
    fn delete_state(&self, user_id: &str) -> PyResult<bool> {
        let url = format!("{}/v1/state/{}", self.base_url, user_id);

        self.runtime.block_on(async {
            let mut req = self.client.delete(&url);

            if let Some(key) = &self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| PyConnectionError::new_err(format!("Connection error: {}", e)))?;

            if resp.status().as_u16() == 404 {
                return Ok(false);
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(PyRuntimeError::new_err(format!(
                    "Server error {}: {}",
                    status.as_u16(),
                    body
                )));
            }

            Ok(true)
        })
    }

    /// Check server health.
    ///
    /// Returns:
    ///     True if server is healthy
    ///
    /// Raises:
    ///     ConnectionError: If the server is unreachable
    fn health(&self) -> PyResult<bool> {
        let url = format!("{}/health", self.base_url);

        self.runtime.block_on(async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| PyConnectionError::new_err(format!("Connection error: {}", e)))?;

            Ok(resp.status().is_success())
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "AttunedClient(base_url='{}', has_api_key={})",
            self.base_url,
            self.api_key.is_some()
        )
    }
}

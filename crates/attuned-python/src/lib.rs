//! Python bindings for Attuned.
//!
//! This crate provides PyO3-based Python bindings for the Attuned framework,
//! enabling Python developers to use human state translation in their LLM applications.

use pyo3::prelude::*;

mod axes;
mod client;
mod infer;
mod snapshot;
mod translator;

use axes::{PyAxisCategory, PyAxisDefinition};
use client::PyAttunedClient;
use infer::{
    extract_features, infer_text, PyAxisEstimate, PyInferenceEngine, PyInferenceSource,
    PyInferredState, PyLinguisticFeatures,
};
use snapshot::{PySource, PyStateSnapshot, PyStateSnapshotBuilder};
use translator::{PyPromptContext, PyRuleTranslator, PyThresholds, PyVerbosity};

/// Get an axis definition by name.
///
/// Args:
///     name: The canonical axis name (e.g., "cognitive_load", "warmth")
///
/// Returns:
///     AxisDefinition if found, None otherwise
///
/// Example:
///     >>> axis = attuned.get_axis("cognitive_load")
///     >>> print(axis.description)
///     >>> print(axis.forbidden_uses)
#[pyfunction]
fn get_axis(name: &str) -> Option<PyAxisDefinition> {
    attuned_core::get_axis(name).map(PyAxisDefinition::from)
}

/// Check if an axis name is valid (exists in CANONICAL_AXES).
///
/// Args:
///     name: The axis name to check
///
/// Returns:
///     True if the axis exists, False otherwise
#[pyfunction]
fn is_valid_axis_name(name: &str) -> bool {
    attuned_core::is_valid_axis_name(name)
}

/// Get all canonical axis names.
///
/// Returns:
///     List of all 23 canonical axis names
#[pyfunction]
fn get_axis_names() -> Vec<&'static str> {
    attuned_core::CANONICAL_AXES
        .iter()
        .map(|a| a.name)
        .collect()
}

/// Get all canonical axes.
///
/// Returns:
///     List of all 23 AxisDefinition objects with full metadata
#[pyfunction]
fn get_all_axes() -> Vec<PyAxisDefinition> {
    attuned_core::CANONICAL_AXES
        .iter()
        .map(PyAxisDefinition::from)
        .collect()
}

/// Python module for Attuned.
#[pymodule]
fn _attuned(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<PyStateSnapshot>()?;
    m.add_class::<PyStateSnapshotBuilder>()?;
    m.add_class::<PySource>()?;

    // Translator types
    m.add_class::<PyPromptContext>()?;
    m.add_class::<PyVerbosity>()?;
    m.add_class::<PyRuleTranslator>()?;
    m.add_class::<PyThresholds>()?;

    // Axis types
    m.add_class::<PyAxisDefinition>()?;
    m.add_class::<PyAxisCategory>()?;

    // Inference types
    m.add_class::<PyInferenceEngine>()?;
    m.add_class::<PyInferredState>()?;
    m.add_class::<PyAxisEstimate>()?;
    m.add_class::<PyInferenceSource>()?;
    m.add_class::<PyLinguisticFeatures>()?;

    // HTTP client
    m.add_class::<PyAttunedClient>()?;

    // Module functions
    m.add_function(wrap_pyfunction!(get_axis, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_axis_name, m)?)?;
    m.add_function(wrap_pyfunction!(get_axis_names, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_axes, m)?)?;

    // Inference functions
    m.add_function(wrap_pyfunction!(infer_text, m)?)?;
    m.add_function(wrap_pyfunction!(extract_features, m)?)?;

    Ok(())
}

//! Common types used throughout Attuned.

use serde::{Deserialize, Serialize};

/// The source of a state snapshot - how the data was obtained.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Source {
    /// User explicitly provided this state (highest trust).
    #[default]
    SelfReport,
    /// State was inferred from behavior or context.
    Inferred,
    /// Combination of self-report and inference.
    Mixed,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::SelfReport => write!(f, "self_report"),
            Source::Inferred => write!(f, "inferred"),
            Source::Mixed => write!(f, "mixed"),
        }
    }
}

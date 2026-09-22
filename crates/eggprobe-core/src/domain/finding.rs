//! Assertion findings kept separate from execution failures.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Severity of an evaluated assertion finding.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Informational observation.
    Info,
    /// An expectation did not hold.
    Warning,
    /// A required expectation did not hold.
    Error,
}

/// Outcome of an assertion evaluation.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingOutcome {
    /// The assertion passed.
    Passed,
    /// The assertion failed.
    Failed,
    /// The assertion could not be evaluated from available evidence.
    Unavailable,
}

/// A normalized assertion result, not a replacement for probe status/errors.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    /// Stable assertion identifier.
    pub assertion_id: String,
    /// Severity assigned by the assertion.
    pub severity: FindingSeverity,
    /// Evaluation outcome.
    pub outcome: FindingOutcome,
    /// Bounded explanation.
    pub message: String,
}

//! Deterministic JSON Schema generation for the public contract.

use schemars::{schema_for, Schema};

use crate::{ProbePlan, ProbeReport};

/// Generate the canonical plan schema.
#[must_use]
pub fn plan_schema() -> Schema {
    schema_for!(ProbePlan)
}

/// Generate the canonical report schema.
#[must_use]
pub fn report_schema() -> Schema {
    schema_for!(ProbeReport)
}

/// Render a schema deterministically for checked-in compatibility artifacts.
///
/// # Panics
///
/// Panics only if the in-memory schema cannot be serialized, which indicates
/// a programming error in the schema generator.
#[must_use]
pub fn pretty(schema: &Schema) -> String {
    serde_json::to_string_pretty(schema).expect("JSON Schema is serializable")
}

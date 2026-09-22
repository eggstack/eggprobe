//! Canonical report envelope and safe route/target summaries.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    finding::Finding,
    probe::ProbeResult,
    route::RouteSpec,
    target::TargetSpec,
    version::{SchemaVersion, ToolVersion},
};

/// Provenance identifying the producer of a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolProvenance {
    /// Product name.
    pub name: String,
    /// Product/library version, distinct from the schema version.
    pub version: ToolVersion,
}

/// A report-safe normalized target.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSummary {
    /// Normalized host.
    pub host: String,
    /// Selected service port, when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

impl From<&TargetSpec> for TargetSummary {
    fn from(target: &TargetSpec) -> Self {
        Self {
            host: target.host.clone(),
            port: target.port,
        }
    }
}

/// A redacted route summary. It cannot hold raw plan credentials.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RouteSummary {
    /// Direct access.
    Direct,
    /// Redacted Eggress route expression.
    Eggress {
        /// Safe expression suitable for reports and display.
        expression: String,
    },
}

/// Overall execution state; child probe states remain independently visible.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    /// All requested operations completed without a negative probe result.
    Ok,
    /// At least one probe failed.
    Failed,
    /// At least one requested operation was unsupported.
    Unsupported,
    /// The execution was cancelled.
    Cancelled,
}

/// The canonical machine-readable result of one plan execution.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeReport {
    /// Machine contract version.
    pub schema_version: SchemaVersion,
    /// Producer provenance.
    pub tool: ToolProvenance,
    /// Caller- or engine-provided execution identifier.
    pub execution_id: String,
    /// Safe target summary.
    pub target: TargetSummary,
    /// Safe route summary, intentionally distinct from `RouteSpec`.
    pub route: RouteSummary,
    /// Overall execution state.
    pub status: ReportStatus,
    /// Ordered child probe results.
    pub probes: Vec<ProbeResult>,
    /// Top-level assertion findings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
    /// Non-fatal report-level warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl ProbeReport {
    /// Build a report envelope from plan data without copying the raw route.
    #[must_use]
    pub fn from_plan(
        plan: &crate::ProbePlan,
        tool: ToolProvenance,
        execution_id: impl Into<String>,
        status: ReportStatus,
        probes: Vec<ProbeResult>,
    ) -> Self {
        Self {
            schema_version: plan.schema_version,
            tool,
            execution_id: execution_id.into(),
            target: (&plan.target).into(),
            route: plan.route.summary(),
            status,
            probes,
            findings: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Serialize the report in stable pretty JSON for local diagnostics.
    ///
    /// # Errors
    ///
    /// Returns the underlying Serde JSON error if a report field cannot be
    /// serialized.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl From<&RouteSpec> for RouteSummary {
    fn from(route: &RouteSpec) -> Self {
        route.summary()
    }
}

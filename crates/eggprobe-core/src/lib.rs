//! Canonical, JSON-first domain types for Eggprobe.
//!
//! This crate owns the data contract shared by future probe execution and the
//! command-line adapter. It intentionally contains no network implementation.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod domain;
pub mod render;

pub use domain::error::{DiagnosticError, DiagnosticErrorKind, DiagnosticStage};
pub use domain::finding::{Finding, FindingOutcome, FindingSeverity};
pub use domain::plan::{AssertionSpec, ExecutionPolicy, ProbePlan, ProbeSpec};
pub use domain::probe::{ProbeEvidence, ProbeKind, ProbeResult, ProbeStatus};
pub use domain::report::{ProbeReport, ReportStatus, RouteSummary, TargetSummary, ToolProvenance};
pub use domain::route::{EggressRoute, RouteSpec};
pub use domain::target::{TargetError, TargetSpec};
pub use domain::timing::{DurationMicros, PhaseTiming, Timing};
pub use domain::version::{SchemaVersion, ToolVersion, VersionParseError};

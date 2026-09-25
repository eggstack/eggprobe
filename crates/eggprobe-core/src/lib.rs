//! Canonical, JSON-first domain types and probe execution for Eggprobe.
//!
//! This crate owns the data contract and probe engine shared by probe execution
//! and the command-line adapter. The `domain` submodule intentionally contains
//! no network implementation.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod assertions;
pub mod domain;
pub mod engine;
pub mod render;
pub mod schema;

pub use assertions::{evaluate_assertions, exit_code, ExitCode};
pub use domain::error::{DiagnosticError, DiagnosticErrorKind, DiagnosticStage};
pub use domain::finding::{Finding, FindingOutcome, FindingSeverity};
pub use domain::plan::{
    AssertionKind, AssertionSpec, ExecutionPolicy, PlanValidationError, ProbePlan, ProbeSpec,
};
pub use domain::probe::{
    DnsResolutionScope, NativeAttemptOutcome, ProbeEvidence, ProbeKind, ProbeResult, ProbeStatus,
    TraceTermination, UdpOutcome,
};
pub use domain::report::{ProbeReport, ReportStatus, RouteSummary, TargetSummary, ToolProvenance};
pub use domain::route::{EggressRoute, RouteSpec};
pub use domain::target::{TargetError, TargetSpec};
pub use domain::timing::{DurationMicros, PhaseTiming, Timing};
pub use domain::version::{SchemaVersion, ToolVersion, VersionParseError};
pub use eggprobe_native::{trace_capability, TraceCapability};
pub use engine::{ProbeEngine, TargetPolicy};

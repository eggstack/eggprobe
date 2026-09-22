//! Pure assertion evaluation and process-exit reduction.

use crate::{
    AssertionKind, AssertionSpec, Finding, FindingOutcome, FindingSeverity, ProbeEvidence,
    ProbeReport, ProbeStatus,
};

/// Evaluate typed assertions without changing the underlying observations.
#[must_use]
pub fn evaluate_assertions(report: &ProbeReport, assertions: &[AssertionSpec]) -> Vec<Finding> {
    assertions
        .iter()
        .map(|spec| evaluate_one(report, spec))
        .collect()
}

fn evaluate_one(report: &ProbeReport, spec: &AssertionSpec) -> Finding {
    let (outcome, message) = match &spec.assertion {
        AssertionKind::HttpStatusRange { min, max } => match http_evidence(report) {
            Some((ProbeStatus::Ok, ProbeEvidence::Http(evidence))) => match evidence.status {
                Some(status) if status >= *min && status <= *max => (
                    FindingOutcome::Passed,
                    format!("HTTP status {status} is within {min}..={max}"),
                ),
                Some(status) => (
                    FindingOutcome::Failed,
                    format!("HTTP status {status} is outside {min}..={max}"),
                ),
                None => unavailable("HTTP status was not observed"),
            },
            Some((status, _)) => unavailable(&format!("HTTP probe completed as {status:?}")),
            None => unavailable("HTTP evidence was not observed"),
        },
        AssertionKind::RequiredHttpVersion { version } => match http_evidence(report) {
            Some((ProbeStatus::Ok, ProbeEvidence::Http(evidence))) => match &evidence.protocol {
                Some(actual) if actual == version => (
                    FindingOutcome::Passed,
                    format!("HTTP version {actual} observed"),
                ),
                Some(actual) => (
                    FindingOutcome::Failed,
                    format!("HTTP version {actual} does not match {version}"),
                ),
                None => unavailable("HTTP version was not observed"),
            },
            _ => unavailable("HTTP evidence was not observed"),
        },
        AssertionKind::RequiredAlpn { value }
        | AssertionKind::RequiredTlsVersion { version: value } => match tls_evidence(report) {
            Some((ProbeStatus::Ok, ProbeEvidence::Tls(evidence))) => {
                let actual = if matches!(spec.assertion, AssertionKind::RequiredAlpn { .. }) {
                    evidence.alpn.as_ref()
                } else {
                    evidence.version.as_ref()
                };
                match actual {
                    Some(actual) if actual == value => (
                        FindingOutcome::Passed,
                        format!("TLS value {actual} observed"),
                    ),
                    Some(actual) => (
                        FindingOutcome::Failed,
                        format!("TLS value {actual} does not match {value}"),
                    ),
                    None => unavailable("TLS value was not observed"),
                }
            }
            _ => unavailable("TLS evidence was not observed"),
        },
        AssertionKind::MaxTotalMicros { micros } => {
            match report.probes.iter().find_map(|probe| probe.timing.as_ref()) {
                Some(timing) if timing.total.as_micros() <= *micros => (
                    FindingOutcome::Passed,
                    format!(
                        "total timing {}µs is within {micros}µs",
                        timing.total.as_micros()
                    ),
                ),
                Some(timing) => (
                    FindingOutcome::Failed,
                    format!(
                        "total timing {}µs exceeds {micros}µs",
                        timing.total.as_micros()
                    ),
                ),
                None => unavailable("timing was not observed"),
            }
        }
    };
    Finding {
        assertion_id: spec.id.clone(),
        severity: FindingSeverity::Error,
        outcome,
        message,
    }
}

fn http_evidence(report: &ProbeReport) -> Option<(&ProbeStatus, &ProbeEvidence)> {
    report
        .probes
        .iter()
        .find_map(|probe| match (&probe.status, probe.evidence.as_ref()) {
            (status, Some(evidence @ ProbeEvidence::Http(_))) => Some((status, evidence)),
            _ => None,
        })
}

fn tls_evidence(report: &ProbeReport) -> Option<(&ProbeStatus, &ProbeEvidence)> {
    report
        .probes
        .iter()
        .find_map(|probe| match (&probe.status, probe.evidence.as_ref()) {
            (status, Some(evidence @ ProbeEvidence::Tls(_))) => Some((status, evidence)),
            _ => None,
        })
}

fn unavailable(message: &str) -> (FindingOutcome, String) {
    (FindingOutcome::Unavailable, message.into())
}

/// Coarse process status used by automation callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitCode {
    /// Completed with all assertions satisfied.
    Success = 0,
    /// A probe or assertion was negative.
    Negative = 1,
    /// Invocation or plan validation failed.
    Invalid = 2,
    /// Eggprobe failed internally.
    Internal = 3,
    /// The caller interrupted execution.
    Interrupted = 130,
}

/// Reduce a completed report to stable coarse process semantics.
#[must_use]
pub fn exit_code(report: &ProbeReport) -> ExitCode {
    if report.status == crate::ReportStatus::Cancelled {
        return ExitCode::Interrupted;
    }
    if report.status == crate::ReportStatus::Failed
        || report.status == crate::ReportStatus::Unsupported
    {
        return ExitCode::Negative;
    }
    if report
        .findings
        .iter()
        .any(|finding| finding.outcome != FindingOutcome::Passed)
    {
        return ExitCode::Negative;
    }
    ExitCode::Success
}

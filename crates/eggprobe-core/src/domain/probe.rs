//! Probe result envelopes and typed evidence.

use serde::{Deserialize, Serialize};

use super::{error::DiagnosticError, finding::Finding, timing::Timing};

/// The stable family of a probe result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeKind {
    /// DNS lookup.
    Dns,
    /// TCP connection.
    Tcp,
    /// Standalone TLS handshake.
    Tls,
    /// HTTP request.
    Http,
}

/// Result state of the probe operation itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    /// The requested operation completed.
    Ok,
    /// The operation ran but failed.
    Failed,
    /// The requested operation is not supported.
    Unsupported,
    /// The caller cancelled the operation.
    Cancelled,
}

/// Typed observations produced by each probe family.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
pub enum ProbeEvidence {
    /// DNS observations.
    Dns(DnsEvidence),
    /// TCP observations.
    Tcp(TcpEvidence),
    /// TLS observations.
    Tls(TlsEvidence),
    /// HTTP observations.
    Http(HttpEvidence),
}

/// DNS evidence available to a report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsEvidence {
    /// Answers observed by the selected route.
    pub addresses: Vec<String>,
}

/// TCP evidence available to a report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpEvidence {
    /// Selected peer address, when observable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
}

/// TLS evidence available to a report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsEvidence {
    /// Negotiated TLS protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Negotiated application protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<String>,
}

/// HTTP evidence available to a report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpEvidence {
    /// Observed response status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Negotiated HTTP protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
}

/// One ordered probe result in a report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeResult {
    /// Probe family.
    pub kind: ProbeKind,
    /// Operation result state.
    pub status: ProbeStatus,
    /// Measured timing, if the operation started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
    /// Structured failure, if the probe failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<DiagnosticError>,
    /// Typed observations, if any were produced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ProbeEvidence>,
    /// Facts that were relevant but unavailable through this observer seam.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unavailable: Vec<String>,
    /// Findings associated with this result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
}

//! Structured diagnostic failures.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Protocol-neutral diagnostic failure category.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticErrorKind {
    /// Name resolution failed.
    Dns,
    /// A configured deadline or timeout was exceeded.
    Timeout,
    /// The target actively refused a connection.
    ConnectionRefused,
    /// No route to the network was available.
    NetworkUnreachable,
    /// The target host was unreachable.
    HostUnreachable,
    /// Authentication or authorization failed.
    Authentication,
    /// TLS negotiation or verification failed.
    Tls,
    /// Protocol exchange failed.
    Protocol,
    /// Local policy rejected the operation.
    Policy,
    /// Local I/O failed.
    Io,
    /// The requested operation is not supported.
    Unsupported,
    /// Eggprobe could not classify the failure safely.
    Internal,
    /// A dependency-specific error did not fit the stable categories.
    Other,
}

/// The stage where a diagnostic error occurred.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticStage {
    /// Name resolution.
    Resolution,
    /// Direct connection establishment.
    DirectConnect,
    /// Proxy hop connection.
    HopConnect,
    /// Proxy hop handshake.
    HopHandshake,
    /// TLS handshake.
    TlsHandshake,
    /// HTTP request write.
    Request,
    /// HTTP response headers.
    ResponseHeaders,
    /// Response body.
    Body,
    /// Outer deadline handling.
    Deadline,
    /// A stage not yet represented by the contract.
    Other,
}

/// A bounded, redaction-safe diagnostic error.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticError {
    /// Stable category.
    pub kind: DiagnosticErrorKind,
    /// Observable failure stage.
    pub stage: DiagnosticStage,
    /// Bounded message with credentials removed by the producer.
    pub message: String,
    /// Optional retry/attempt context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    /// Zero-based Eggress route hop that reported the failure, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_hop_index: Option<usize>,
    /// Normalized Eggress hop protocol associated with the failure, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_protocol: Option<String>,
}

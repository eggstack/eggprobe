//! Probe result envelopes and typed evidence.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{error::DiagnosticError, finding::Finding, timing::Timing};

/// The stable family of a probe result.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
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
    /// Target-scoped local route/interface inspection.
    Route,
    /// ICMP echo.
    IcmpEcho,
    /// Direct UDP exchange.
    Udp,
    /// Traceroute.
    Trace,
    /// Active path MTU discovery.
    PathMtu,
}

/// Result state of the probe operation itself.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
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
    /// Route and interface observations.
    Route(RouteEvidence),
    /// ICMP echo attempt outcomes.
    IcmpEcho(IcmpEchoEvidence),
    /// UDP transmission and response outcome.
    Udp(UdpEvidence),
    /// Ordered traceroute hops and termination.
    Trace(TraceEvidence),
    /// Active path-MTU bounds and provenance.
    PathMtu(PathMtuEvidence),
}

/// Target-scoped route/interface evidence. Correlations are explicitly separate.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteEvidence {
    /// Address inspected.
    pub target: String,
    /// Kernel-selected local source address, when observed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Interface index, name, and MTU correlated with the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<InterfaceEvidence>,
    /// Candidate route-table entries; never implies authoritative selection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteCandidate>,
    /// Whether additional equally specific candidates were omitted by the cap.
    #[serde(default, skip_serializing_if = "is_false")]
    pub routes_truncated: bool,
    /// Provenance of any association.
    pub correlation: RouteCorrelation,
}

/// Correlated local interface facts.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceEvidence {
    /// Operating-system index.
    pub index: Option<u32>,
    /// Interface name.
    pub name: Option<String>,
    /// Interface addresses observed in the platform snapshot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<String>,
    /// Whether the interface is administratively up, when exposed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up: Option<bool>,
    /// Interface MTU, distinct from path MTU.
    pub mtu: Option<u32>,
}
/// Read-only route-table candidate.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteCandidate {
    /// Network prefix.
    pub destination: String,
    /// Interface index when observed.
    pub interface_index: Option<u32>,
    /// Gateway when observed.
    pub gateway: Option<String>,
    /// Metric when observed.
    pub metric: Option<u32>,
    /// Table identifier when observed.
    pub table: Option<u32>,
    /// Route protocol when exposed by the platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// Route scope when exposed by the platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}
/// How route and interface evidence was related.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCorrelation {
    /// Kernel source address observed and mapped to an interface.
    ObservedSource,
    /// Candidate route entries matched by prefix only.
    CorrelatedCandidates,
    /// More than one candidate remains.
    Ambiguous,
    /// Backend could not provide correlation.
    Unavailable,
}

/// One ICMP attempt.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IcmpAttempt {
    /// One-based sequence.
    pub sequence: u16,
    /// Outcome category.
    pub outcome: NativeAttemptOutcome,
    /// Responder address, when observed.
    pub responder: Option<String>,
    /// Round-trip microseconds, when observed.
    pub rtt_micros: Option<u64>,
}
/// Bounded ICMP evidence.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IcmpEchoEvidence {
    /// Destination address.
    pub destination: String,
    /// Address family.
    pub family: AddressFamily,
    /// Configured payload size.
    pub payload_bytes: u16,
    /// Attempts in send order.
    pub attempts: Vec<IcmpAttempt>,
}

/// IP address family.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressFamily {
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
}

/// Normalized outcome for a native packet attempt.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeAttemptOutcome {
    /// Echo or service response observed.
    Reply,
    /// Attempt timed out without a response.
    TimedOut,
    /// Destination reported unreachable.
    DestinationUnreachable,
    /// Network reported unreachable.
    NetworkUnreachable,
    /// Host reported unreachable.
    HostUnreachable,
    /// Intermediate router reported TTL/hop limit expiration.
    TimeExceeded,
    /// Destination reached during a trace.
    DestinationReached,
}
/// UDP receive result.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UdpOutcome {
    /// Datagram accepted by local socket.
    Sent,
    /// Response received.
    Response,
    /// OS reported unreachable.
    Unreachable,
    /// No response before timeout.
    Timeout,
}
/// Direct UDP evidence.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UdpEvidence {
    /// Local socket address.
    pub local: Option<String>,
    /// Request bytes sent.
    pub transmitted_bytes: u32,
    /// Outcome independent of service health.
    pub outcome: UdpOutcome,
    /// Response source when observed.
    pub response_source: Option<String>,
    /// Response bytes.
    pub response_bytes: Option<u32>,
    /// Explicitly requested bounded response sample.
    pub response_sample: Option<Vec<u8>>,
}
/// One trace attempt.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceAttempt {
    /// Responder address.
    pub responder: Option<String>,
    /// Round-trip microseconds.
    pub rtt_micros: Option<u64>,
    /// Stable outcome.
    pub outcome: NativeAttemptOutcome,
}
/// Ordered trace hop.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceHop {
    /// TTL/hop index.
    pub hop: u8,
    /// Attempts in send order.
    pub attempts: Vec<TraceAttempt>,
}
/// Path completion reason.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceTermination {
    /// Destination reached.
    DestinationReached,
    /// Hop bound exhausted.
    MaxHops,
    /// Global deadline elapsed.
    Deadline,
    /// Destination reported unreachable.
    Unreachable,
    /// Required OS privilege was unavailable.
    PermissionDenied,
    /// Backend unsupported.
    Unsupported,
}
/// Trace evidence.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceEvidence {
    /// Destination.
    pub destination: String,
    /// Ordered hops.
    pub hops: Vec<TraceHop>,
    /// Termination.
    pub termination: TraceTermination,
}
/// PMTU discovery result class.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathMtuStatus {
    /// Exact value established.
    Exact,
    /// Only lower/upper bounds established.
    Bounded,
    /// No trustworthy size signal.
    Inconclusive,
}
/// PMTU evidence, distinct from interface MTU.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathMtuEvidence {
    /// Destination.
    pub destination: String,
    /// Exact path MTU, if established.
    pub exact_bytes: Option<u32>,
    /// Proven lower bound.
    pub lower_bytes: Option<u32>,
    /// Explicitly signalled upper bound.
    pub upper_bytes: Option<u32>,
    /// Method provenance.
    pub method: PathMtuMethod,
    /// Result class.
    pub status: PathMtuStatus,
}

/// Active packet method used for PMTU discovery.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathMtuMethod {
    /// ICMP echo request with fragmentation control.
    IcmpEcho,
    /// Direct UDP probe with fragmentation control.
    Udp,
}

/// DNS evidence available to a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsEvidence {
    /// Answers observed by Eggprobe's local system resolver. This does not
    /// represent the resolver used by a remote Eggress proxy route.
    pub addresses: Vec<String>,
    /// Resolver scope for these answers.
    pub resolution_scope: DnsResolutionScope,
}

/// Resolver that produced the reported DNS answers.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsResolutionScope {
    /// Eggprobe's local client/system resolver.
    Client,
}

/// TCP evidence available to a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpEvidence {
    /// Selected peer address, when observable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    /// Local address when the provider exposes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    /// Number of address attempts made before success.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub attempts: u32,
}

/// TLS evidence available to a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsEvidence {
    /// Negotiated TLS protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Negotiated application protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<String>,
    /// Negotiated cipher suite when observed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher_suite: Option<String>,
}

/// HTTP evidence available to a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpEvidence {
    /// Observed response status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Negotiated HTTP protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// Number of body bytes retained in the bounded sample.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub body_sample_bytes: u32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

/// One ordered probe result in a report.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
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

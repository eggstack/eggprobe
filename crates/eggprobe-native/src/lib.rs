//! Backend-neutral request and observation types for native diagnostics.
//!
//! Platform adapters are deliberately kept behind this crate. This initial
//! substrate does not perform native operations; capabilities are explicit.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::{future::Future, net::IpAddr, pin::Pin};

/// Native operation requested by the core engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeRequest {
    /// Inspect local path selection and route metadata for one address.
    Route {
        /// Address inspected.
        target: IpAddr,
    },
    /// Send one bounded ICMP echo request.
    IcmpEcho {
        /// Destination address.
        target: IpAddr,
        /// Echo sequence.
        sequence: u16,
        /// Payload length.
        payload_bytes: u16,
    },
    /// Send one direct UDP datagram.
    Udp {
        /// Destination socket.
        target: std::net::SocketAddr,
        /// Bounded request payload.
        payload: Vec<u8>,
    },
    /// Trace one bounded path.
    Trace {
        /// Destination address.
        target: IpAddr,
        /// Maximum hop count.
        max_hops: u8,
        /// Maximum attempts at each hop.
        attempts_per_hop: u8,
    },
    /// Probe one packet size for active path-MTU discovery.
    PathMtu {
        /// Destination address.
        target: IpAddr,
        /// Packet size to probe.
        packet_bytes: u16,
    },
}

/// Stable result category independent of an operating-system error enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeErrorKind {
    /// The local process lacks the required permission.
    PermissionDenied,
    /// The operation is not available on this platform/backend.
    Unsupported,
    /// The finite operation deadline elapsed.
    Timeout,
    /// The remote network or host reported an unreachable condition.
    Unreachable,
    /// A local operation failed.
    Io,
}

/// A bounded, dependency-neutral backend failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeError {
    /// Stable category.
    pub kind: NativeErrorKind,
}

/// Backend-neutral observation. Detailed family results are added with their
/// owning capability milestones.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeObservation {
    /// A capability is absent on this platform.
    Unsupported,
    /// Request accepted by the backend; capability milestones refine this.
    Completed,
}

/// Boxed future returned by an injectable native backend.
pub type NativeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<NativeObservation, NativeError>> + Send + 'a>>;

/// Injectable native operations used by core normalization and tests.
pub trait NativeBackend: Send + Sync {
    /// Execute one native request, or report an explicit capability/error state.
    fn execute(&self, request: NativeRequest) -> NativeFuture<'_>;
}

/// Platform-neutral interface facts needed by the route probe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceObservation {
    /// OS-assigned interface index.
    pub index: u32,
    /// System interface name.
    pub name: String,
    /// Addresses assigned to the interface.
    pub addresses: Vec<IpAddr>,
    /// Whether the interface is administratively up.
    pub up: bool,
    /// Interface MTU in bytes, when available.
    pub mtu: Option<u32>,
}

/// Read-only route candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteObservation {
    /// Destination prefix, for example `192.0.2.0/24`.
    pub destination: String,
    /// Egress interface index when exposed.
    pub interface_index: Option<u32>,
    /// Gateway when exposed.
    pub gateway: Option<IpAddr>,
    /// Route metric when exposed.
    pub metric: Option<u32>,
    /// Routing table identifier when exposed.
    pub table: Option<u32>,
    /// Route protocol when exposed.
    pub protocol: Option<String>,
    /// Route scope when exposed.
    pub scope: Option<String>,
}

/// Result of source-address observation and route-table correlation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteInspection {
    /// Kernel-selected source address from a UDP connect with no payload.
    pub source: Option<IpAddr>,
    /// Interface matched by the observed source address.
    pub interface: Option<InterfaceObservation>,
    /// Best-prefix route candidates; these are not authoritative selection.
    pub candidates: Vec<RouteObservation>,
    /// Whether more equally specific candidates were present than the cap.
    pub candidates_truncated: bool,
    /// True when multiple equally specific candidates remain.
    pub ambiguous: bool,
}

/// Maximum request payload accepted for one direct UDP datagram.
pub const MAX_UDP_PAYLOAD_BYTES: usize = 1200;

/// Maximum response bytes retained in a UDP response sample.
pub const MAX_UDP_RESPONSE_SAMPLE_BYTES: usize = 1400;

/// Outcome of one direct UDP exchange, independent of remote service health.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UdpExchangeOutcome {
    /// The datagram was accepted by the local socket; no reply was awaited.
    Sent,
    /// One reply datagram was observed.
    Response,
    /// The operating system surfaced an unreachable condition.
    Unreachable,
    /// No reply arrived before the bounded receive window elapsed.
    Timeout,
}

/// Backend-neutral observation of one direct UDP exchange.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UdpExchange {
    /// Local socket address selected for the exchange.
    pub local: Option<std::net::SocketAddr>,
    /// Request bytes accepted by the local socket.
    pub transmitted_bytes: u32,
    /// Exchange outcome.
    pub outcome: UdpExchangeOutcome,
    /// Source of the observed reply, when one arrived.
    pub response_source: Option<std::net::SocketAddr>,
    /// Reply datagram bytes observed, when one arrived.
    pub response_bytes: Option<u32>,
    /// Bounded reply sample, present only when a reply was awaited.
    pub response_sample: Option<Vec<u8>>,
}

/// Exchange one bounded datagram with a directly connected UDP socket.
///
/// A successful local `send` reports transmission only; it never claims the
/// remote service is healthy. When `receive` is set, one reply is awaited
/// for at most `timeout`; silence yields [`UdpExchangeOutcome::Timeout`] and
/// an OS unreachable signal yields [`UdpExchangeOutcome::Unreachable`], both
/// as completed observations rather than local failures. Dropping the
/// returned future cancels the exchange and closes the socket.
///
/// # Errors
///
/// Returns a bounded native error when the payload exceeds
/// [`MAX_UDP_PAYLOAD_BYTES`], the socket cannot be bound or connected, or the
/// local send fails.
pub async fn udp_exchange(
    target: std::net::SocketAddr,
    payload: &[u8],
    receive: bool,
    timeout: std::time::Duration,
) -> Result<UdpExchange, NativeError> {
    if payload.len() > MAX_UDP_PAYLOAD_BYTES {
        return Err(NativeError {
            kind: NativeErrorKind::Io,
        });
    }
    let bind: std::net::SocketAddr = match target {
        std::net::SocketAddr::V4(_) => {
            std::net::SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0)
        }
        std::net::SocketAddr::V6(_) => {
            std::net::SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0)
        }
    };
    let socket = tokio::net::UdpSocket::bind(bind)
        .await
        .map_err(|error| normalize_io(&error))?;
    socket
        .connect(target)
        .await
        .map_err(|error| normalize_io(&error))?;
    let local = socket.local_addr().ok();
    let sent = socket
        .send(payload)
        .await
        .map_err(|error| normalize_io(&error))?;
    let transmitted_bytes = u32::try_from(sent).unwrap_or(u32::MAX);
    if !receive {
        return Ok(UdpExchange {
            local,
            transmitted_bytes,
            outcome: UdpExchangeOutcome::Sent,
            response_source: None,
            response_bytes: None,
            response_sample: None,
        });
    }
    let mut buffer = vec![0_u8; MAX_UDP_RESPONSE_SAMPLE_BYTES + 648];
    match tokio::time::timeout(timeout, socket.recv_from(&mut buffer)).await {
        Ok(Ok((len, source))) => {
            let response_bytes = u32::try_from(len).unwrap_or(u32::MAX);
            let sample = buffer
                .into_iter()
                .take(len.min(MAX_UDP_RESPONSE_SAMPLE_BYTES))
                .collect::<Vec<_>>();
            Ok(UdpExchange {
                local,
                transmitted_bytes,
                outcome: UdpExchangeOutcome::Response,
                response_source: Some(source),
                response_bytes: Some(response_bytes),
                response_sample: Some(sample),
            })
        }
        Ok(Err(error)) => match error.kind() {
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::HostUnreachable
            | std::io::ErrorKind::NetworkUnreachable => Ok(UdpExchange {
                local,
                transmitted_bytes,
                outcome: UdpExchangeOutcome::Unreachable,
                response_source: None,
                response_bytes: None,
                response_sample: None,
            }),
            _ => Err(normalize_io(&error)),
        },
        Err(_) => Ok(UdpExchange {
            local,
            transmitted_bytes,
            outcome: UdpExchangeOutcome::Timeout,
            response_source: None,
            response_bytes: None,
            response_sample: None,
        }),
    }
}

/// Outcome observed for one trace probe sent at one TTL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceProbeOutcome {
    /// The destination host answered at this TTL.
    DestinationReached,
    /// An intermediate router reported TTL expiration.
    TimeExceeded,
    /// A non-ICMP reply was observed.
    Reply,
    /// A router reported the destination unreachable.
    DestinationUnreachable,
    /// The probe was sent but no usable reply arrived.
    TimedOut,
}

/// Backend-neutral observation of one trace probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceProbeObservation {
    /// TTL the probe was sent with.
    pub ttl: u8,
    /// Address that answered, when one did.
    pub responder: Option<IpAddr>,
    /// Round-trip microseconds, when a reply arrived.
    pub rtt_micros: Option<u64>,
    /// Stable outcome.
    pub outcome: TraceProbeOutcome,
}

/// How one trace round completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceRoundReason {
    /// The destination answered during the round.
    TargetFound,
    /// The round exhausted its bounded duration.
    TimeLimit,
}

/// Backend-neutral observation of one trace round.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceRoundObservation {
    /// How the round completed.
    pub reason: TraceRoundReason,
    /// Probe observations in send order.
    pub probes: Vec<TraceProbeObservation>,
}

/// Backend-neutral result of one bounded trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceReport {
    /// Completed rounds in execution order; short when the timeout fired.
    pub rounds: Vec<TraceRoundObservation>,
    /// Whether every requested round completed.
    pub completed: bool,
}

/// Attempt evidence for one TTL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceAttemptSummary {
    /// Address that answered, when one did.
    pub responder: Option<IpAddr>,
    /// Round-trip microseconds, when observed.
    pub rtt_micros: Option<u64>,
    /// Stable outcome.
    pub outcome: TraceProbeOutcome,
}

/// Ordered evidence for one TTL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceHopSummary {
    /// TTL/hop index.
    pub hop: u8,
    /// Attempts in round order.
    pub attempts: Vec<TraceAttemptSummary>,
}

/// How a bounded trace terminated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceTerminationSummary {
    /// The destination answered.
    DestinationReached,
    /// A router reported the destination unreachable.
    Unreachable,
    /// Rounds completed without the destination answering.
    MaxHops,
    /// The bounded wait expired with rounds still pending.
    Deadline,
}

/// Ordered hops plus termination derived from a trace report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceSummary {
    /// Hops ordered by TTL.
    pub hops: Vec<TraceHopSummary>,
    /// Termination reason.
    pub termination: TraceTerminationSummary,
}

/// Fixed machine-visible message for traces denied by host privilege policy.
/// Dependency and operating-system error text is never forwarded into
/// reports.
pub const TRACE_PERMISSION_MESSAGE: &str = "UDP trace requires additional local privilege";

/// Whether the current host lets this process execute a UDP trace.
///
/// `Executable` means the backend mode selected for this host can run here:
/// unprivileged mode on macOS, or privileged mode where the required local
/// privilege is already effective. `PermissionDenied` means the host requires
/// a privilege this process does not hold. This derives from the same
/// decision as [`trace_path`] so host-aware tests stay truthful without
/// hard-coding per-OS expectations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceCapability {
    /// The trace backend mode selected for this host can execute here.
    Executable,
    /// Required local privilege is unavailable to this process.
    PermissionDenied,
}

/// Report the current host's trace capability without sending any probe.
#[must_use]
pub fn trace_capability() -> TraceCapability {
    match current_privilege_mode() {
        Ok(_) => TraceCapability::Executable,
        Err(_) => TraceCapability::PermissionDenied,
    }
}

/// Backend privilege mode selected for one trace. Trippy's privilege types
/// never cross this crate's public boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TracePrivilegeMode {
    /// Backend unprivileged mode, valid only where upstream documents it.
    Unprivileged,
    /// Backend privileged mode, valid only with already-effective privilege.
    Privileged,
}

/// Already-effective host privilege relevant to tracing, kept injectable so
/// mode selection is deterministically testable without raw-socket privilege.
///
/// Only already-effective privilege is consulted. `Privilege::needs_privileges`
/// is deliberately not used: whether the backend supports unprivileged mode
/// follows upstream documentation per target (today: macOS only for Trippy
/// 0.13 UDP), never runtime inference — inferring support was the M005 defect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PrivilegeFacts {
    /// The required local privilege is already effective (Linux effective
    /// `CAP_NET_RAW`, or an elevated token on Windows).
    has_privileges: bool,
}

/// Pure privilege-mode policy shared by production and tests.
///
/// - `unprivileged_supported` is true only where the backend documents
///   unprivileged tracing;
/// - `None` facts model a discovery failure and deny with the same bounded
///   category instead of leaking dependency text;
/// - a mere privilege deficit is always `PermissionDenied`, never
///   `Unsupported`: the trace family itself is supported, the host simply
///   withholds the required privilege.
fn select_privilege_mode(
    unprivileged_supported: bool,
    facts: Option<PrivilegeFacts>,
) -> Result<TracePrivilegeMode, NativeErrorKind> {
    if unprivileged_supported {
        return Ok(TracePrivilegeMode::Unprivileged);
    }
    if facts.is_some_and(|facts| facts.has_privileges) {
        Ok(TracePrivilegeMode::Privileged)
    } else {
        Err(NativeErrorKind::PermissionDenied)
    }
}

/// Decide the backend privilege mode from actual host capability.
///
/// macOS uses the backend's documented unprivileged mode. Everywhere else the
/// required privilege must already be effective. Discovery is read-only —
/// Eggprobe never acquires, raises, clears, or otherwise mutates privilege
/// state (no `sudo`, no file-capability changes, no elevation prompts), so
/// concurrent traces cannot race on process- or thread-wide capability sets
/// and no privilege-drop lifecycle is owed. A discovery failure denies with
/// the same bounded category instead of leaking dependency text.
fn current_privilege_mode() -> Result<TracePrivilegeMode, NativeErrorKind> {
    // Unprivileged support follows backend documentation per target (Trippy
    // 0.13 documents it for macOS only); discovery runs solely where a
    // privileged decision needs it.
    if cfg!(target_os = "macos") {
        return Ok(TracePrivilegeMode::Unprivileged);
    }
    select_privilege_mode(
        false,
        trippy_privilege::Privilege::discover()
            .ok()
            .map(|privilege| PrivilegeFacts {
                has_privileges: privilege.has_privileges(),
            }),
    )
}

/// Trace one bounded direct path with UDP probes.
///
/// macOS uses the backend's documented unprivileged mode. Linux and Windows
/// require already-effective local privilege (Linux effective `CAP_NET_RAW`,
/// Windows an elevated token) and report [`NativeErrorKind::PermissionDenied`]
/// otherwise; lack of privilege is an execution failure, never a silent hop.
///
/// Each round sends one probe per TTL from 1 through `max_hops` using the
/// classic (non-Paris) strategy and a fixed traditional destination port.
/// Silent TTLs are preserved as [`TraceProbeOutcome::TimedOut`] observations,
/// never as engine failures. Privilege discovery and channel construction
/// share one dedicated blocking worker; `timeout` bounds the wait, and expiry
/// keeps the rounds completed so far with `completed` set to false. No
/// reverse-DNS lookup is performed.
///
/// # Errors
///
/// Returns a bounded native error when required privilege is unavailable, the
/// tracer cannot be built, or the trace fails locally.
///
/// # Panics
///
/// Panics only if the internal round-collection lock is poisoned, which
/// cannot happen because the lock is held across non-panicking pushes.
pub async fn trace_path(
    target: IpAddr,
    max_hops: u8,
    rounds: u8,
    read_timeout: std::time::Duration,
    max_round_duration: std::time::Duration,
    timeout: std::time::Duration,
) -> Result<TraceReport, NativeError> {
    let rounds = usize::from(rounds.max(1));
    let max_ttl = max_hops.max(1);
    let src_port = next_trace_src_port();
    let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let worker_collected = std::sync::Arc::clone(&collected);
    let worker = tokio::task::spawn_blocking(move || {
        let mode = current_privilege_mode().map_err(|kind| NativeError { kind })?;
        let tracer = trippy_core::Builder::new(target)
            .privilege_mode(match mode {
                TracePrivilegeMode::Unprivileged => trippy_core::PrivilegeMode::Unprivileged,
                TracePrivilegeMode::Privileged => trippy_core::PrivilegeMode::Privileged,
            })
            // Eggprobe never acquires privilege (discovery is read-only), so
            // there is nothing to drop: enabling the backend drop would clear
            // an effective set the tracer did not raise.
            .drop_privileges(false)
            .protocol(trippy_core::Protocol::Udp)
            .multipath_strategy(trippy_core::MultipathStrategy::Classic)
            // Classic varies the destination port per probe, so the fixed
            // source port is the per-tracer identity and must be unique.
            .port_direction(trippy_core::PortDirection::FixedSrc(src_port))
            .first_ttl(1)
            .max_ttl(max_ttl)
            .max_rounds(Some(rounds))
            .min_round_duration(std::time::Duration::ZERO)
            .read_timeout(read_timeout)
            .max_round_duration(max_round_duration)
            .build()
            .map_err(|error| map_trace_error(&error))?;
        tracer
            .run_with(|round| {
                worker_collected
                    .lock()
                    .expect("trace rounds are pushed under a short-lived lock")
                    .push(observe_round(target, round));
            })
            .map_err(|error| map_trace_error(&error))
    });
    if let Ok(joined) = tokio::time::timeout(timeout, worker).await {
        joined.map_err(|_| NativeError {
            kind: NativeErrorKind::Io,
        })??;
        let rounds = std::mem::take(
            &mut *collected
                .lock()
                .expect("trace rounds are taken under a short-lived lock"),
        );
        Ok(TraceReport {
            rounds,
            completed: true,
        })
    } else {
        let rounds = std::mem::take(
            &mut *collected
                .lock()
                .expect("trace rounds are taken under a short-lived lock"),
        );
        Ok(TraceReport {
            rounds,
            completed: false,
        })
    }
}

/// Derive ordered hop evidence and termination from a trace report.
///
/// Probes for TTLs outside `1..=max_hops` are ignored defensively. An
/// incomplete report (the bounded wait expired) terminates as
/// [`TraceTerminationSummary::Deadline`] while retaining completed rounds.
#[must_use]
pub fn summarize_trace(target: IpAddr, max_hops: u8, report: &TraceReport) -> TraceSummary {
    let mut hops: Vec<TraceHopSummary> = Vec::new();
    for round in &report.rounds {
        for probe in &round.probes {
            if probe.ttl == 0 || probe.ttl > max_hops {
                continue;
            }
            let attempt = TraceAttemptSummary {
                responder: probe.responder,
                rtt_micros: probe.rtt_micros,
                outcome: probe.outcome,
            };
            match hops.iter_mut().find(|hop| hop.hop == probe.ttl) {
                Some(hop) => hop.attempts.push(attempt),
                None => hops.push(TraceHopSummary {
                    hop: probe.ttl,
                    attempts: vec![attempt],
                }),
            }
        }
    }
    hops.sort_by_key(|hop| hop.hop);
    let reached = report
        .rounds
        .iter()
        .flat_map(|round| &round.probes)
        .any(|probe| {
            probe.outcome == TraceProbeOutcome::DestinationReached
                && probe.responder == Some(target)
        });
    let unreachable = report
        .rounds
        .iter()
        .flat_map(|round| &round.probes)
        .any(|probe| probe.outcome == TraceProbeOutcome::DestinationUnreachable);
    let termination = if !report.completed {
        TraceTerminationSummary::Deadline
    } else if reached {
        TraceTerminationSummary::DestinationReached
    } else if unreachable {
        TraceTerminationSummary::Unreachable
    } else {
        TraceTerminationSummary::MaxHops
    };
    TraceSummary { hops, termination }
}

fn map_trace_error(error: &trippy_core::Error) -> NativeError {
    let kind = match error {
        trippy_core::Error::PrivilegeError(_) => NativeErrorKind::PermissionDenied,
        _ => NativeErrorKind::Io,
    };
    NativeError { kind }
}

/// Base source port; each trace takes the next port in a small window so
/// concurrent traces never share the local bind. Kept clear of the
/// traditional destination range and the well-known range.
const TRACE_SRC_PORT_BASE: u16 = 43534;
/// Width of the per-trace source port window.
const TRACE_SRC_PORT_WINDOW: u16 = 1000;

static NEXT_TRACE_SRC_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

fn next_trace_src_port() -> trippy_core::Port {
    let offset = NEXT_TRACE_SRC_PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        % TRACE_SRC_PORT_WINDOW;
    trippy_core::Port(TRACE_SRC_PORT_BASE + offset)
}

fn observe_round(target: IpAddr, round: &trippy_core::Round<'_>) -> TraceRoundObservation {
    TraceRoundObservation {
        reason: match round.reason {
            trippy_core::CompletionReason::TargetFound => TraceRoundReason::TargetFound,
            trippy_core::CompletionReason::RoundTimeLimitExceeded => TraceRoundReason::TimeLimit,
        },
        probes: round
            .probes
            .iter()
            .filter_map(|probe| observe_probe(target, probe))
            .collect(),
    }
}

fn observe_probe(
    target: IpAddr,
    probe: &trippy_core::ProbeStatus,
) -> Option<TraceProbeObservation> {
    match probe {
        trippy_core::ProbeStatus::Complete(complete) => {
            let rtt_micros = complete
                .received
                .duration_since(complete.sent)
                .ok()
                .map(|span| {
                    u64::try_from(span.as_micros().min(u128::from(u64::MAX))).unwrap_or(u64::MAX)
                });
            let outcome = if complete.host == target {
                TraceProbeOutcome::DestinationReached
            } else {
                match complete.icmp_packet_type {
                    trippy_core::IcmpPacketType::TimeExceeded(_) => TraceProbeOutcome::TimeExceeded,
                    trippy_core::IcmpPacketType::EchoReply(_)
                    | trippy_core::IcmpPacketType::NotApplicable => TraceProbeOutcome::Reply,
                    trippy_core::IcmpPacketType::Unreachable(_) => {
                        TraceProbeOutcome::DestinationUnreachable
                    }
                }
            };
            Some(TraceProbeObservation {
                ttl: complete.ttl.0,
                responder: Some(complete.host),
                rtt_micros,
                outcome,
            })
        }
        trippy_core::ProbeStatus::Awaited(awaited) => Some(TraceProbeObservation {
            ttl: awaited.ttl.0,
            responder: None,
            rtt_micros: None,
            outcome: TraceProbeOutcome::TimedOut,
        }),
        // A failed probe produced no usable reply; the attempt is preserved
        // as silence rather than failing the whole trace.
        trippy_core::ProbeStatus::Failed(failed) => Some(TraceProbeObservation {
            ttl: failed.ttl.0,
            responder: None,
            rtt_micros: None,
            outcome: TraceProbeOutcome::TimedOut,
        }),
        // Probes that were never transmitted are not attempts.
        trippy_core::ProbeStatus::NotSent | trippy_core::ProbeStatus::Skipped => None,
    }
}

/// Inspect local interface and route metadata for one resolved destination.
///
/// # Errors
///
/// Returns a bounded native error when the OS route snapshot cannot be read.
pub async fn inspect_route(target: IpAddr) -> Result<RouteInspection, NativeError> {
    let bind: std::net::SocketAddr = match target {
        IpAddr::V4(_) => std::net::SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => std::net::SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
    };
    let socket = tokio::net::UdpSocket::bind(bind)
        .await
        .map_err(|error| normalize_io(&error))?;
    socket
        .connect(std::net::SocketAddr::new(target, 33434))
        .await
        .map_err(|error| normalize_io(&error))?;
    let source = socket.local_addr().ok().map(|address| address.ip());
    drop(socket);

    let interfaces = netdev::get_interfaces();
    let interface = source.and_then(|source| {
        interfaces
            .iter()
            .find(|item| {
                item.ipv4
                    .iter()
                    .any(|network| IpAddr::V4(network.addr()) == source)
                    || item
                        .ipv6
                        .iter()
                        .any(|network| IpAddr::V6(network.addr()) == source)
            })
            .map(|item| InterfaceObservation {
                index: item.index,
                name: item.name.clone(),
                addresses: item.ip_addrs().into_iter().take(64).collect(),
                up: item.is_up(),
                mtu: item.mtu,
            })
    });

    let mut routes = netroute::list_routes().map_err(|error| normalize_io(&error))?;
    routes.retain(|route| {
        prefix_contains(route.destination.addr, route.destination.prefix_len, target)
    });
    routes.sort_by(|left, right| {
        right
            .destination
            .prefix_len
            .cmp(&left.destination.prefix_len)
            .then_with(|| left.destination.addr.cmp(&right.destination.addr))
            .then_with(|| left.ifindex.cmp(&right.ifindex))
            .then_with(|| left.metric.cmp(&right.metric))
    });
    let best_prefix = routes.first().map(|route| route.destination.prefix_len);
    routes.retain(|route| Some(route.destination.prefix_len) == best_prefix);
    let candidates_truncated = routes.len() > 32;
    let candidates = routes
        .into_iter()
        .take(32)
        .map(|route| RouteObservation {
            destination: route.destination.to_string(),
            interface_index: route.ifindex,
            gateway: route.gateway,
            metric: route.metric,
            table: route.table,
            protocol: route.protocol.map(|value| format!("{value:?}")),
            scope: route.scope.map(|value| format!("{value:?}")),
        })
        .collect::<Vec<_>>();
    let ambiguous = candidates.len() > 1;
    Ok(RouteInspection {
        source,
        interface,
        candidates,
        candidates_truncated,
        ambiguous,
    })
}

fn normalize_io(error: &std::io::Error) -> NativeError {
    let kind = match error.kind() {
        std::io::ErrorKind::PermissionDenied => NativeErrorKind::PermissionDenied,
        std::io::ErrorKind::TimedOut => NativeErrorKind::Timeout,
        std::io::ErrorKind::NetworkUnreachable | std::io::ErrorKind::HostUnreachable => {
            NativeErrorKind::Unreachable
        }
        _ => NativeErrorKind::Io,
    };
    NativeError { kind }
}

fn prefix_contains(prefix: IpAddr, bits: u8, target: IpAddr) -> bool {
    match (prefix, target) {
        (IpAddr::V4(prefix), IpAddr::V4(target)) if bits <= 32 => {
            let mask = if bits == 0 {
                0
            } else {
                u32::MAX << (32 - bits)
            };
            u32::from(prefix) & mask == u32::from(target) & mask
        }
        (IpAddr::V6(prefix), IpAddr::V6(target)) if bits <= 128 => {
            let mask = if bits == 0 {
                0
            } else {
                u128::MAX << (128 - bits)
            };
            u128::from(prefix) & mask == u128::from(target) & mask
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_backend_preserves_completed_and_permission_outcomes() {
        let request = NativeRequest::Route {
            target: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        };
        let success = FakeBackend {
            result: Ok(NativeObservation::Completed),
        }
        .execute(request.clone())
        .await;
        assert_eq!(success, Ok(NativeObservation::Completed));

        let denied = FakeBackend {
            result: Err(NativeError {
                kind: NativeErrorKind::PermissionDenied,
            }),
        }
        .execute(request.clone())
        .await;
        assert_eq!(
            denied,
            Err(NativeError {
                kind: NativeErrorKind::PermissionDenied,
            })
        );

        let unsupported = FakeBackend {
            result: Ok(NativeObservation::Unsupported),
        }
        .execute(request)
        .await;
        assert_eq!(unsupported, Ok(NativeObservation::Unsupported));
    }

    #[test]
    fn prefix_match_is_family_safe_and_handles_default_route() {
        assert!(prefix_contains(
            "0.0.0.0".parse().unwrap(),
            0,
            "203.0.113.8".parse().unwrap()
        ));
        assert!(!prefix_contains(
            "192.0.2.0".parse().unwrap(),
            24,
            "2001:db8::1".parse().unwrap()
        ));
    }

    #[tokio::test]
    async fn udp_exchange_reports_send_only_transmission() {
        let target: std::net::SocketAddr = "127.0.0.1:9".parse().unwrap();
        let exchange = udp_exchange(target, b"ping", false, std::time::Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(exchange.outcome, UdpExchangeOutcome::Sent);
        assert_eq!(exchange.transmitted_bytes, 4);
        assert!(exchange.local.is_some());
        assert_eq!(exchange.response_source, None);
        assert_eq!(exchange.response_sample, None);
    }

    #[tokio::test]
    async fn udp_exchange_observes_loopback_echo_reply() {
        let echo = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = echo.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut buffer = [0_u8; 2048];
            let (len, source) = echo.recv_from(&mut buffer).await.unwrap();
            echo.send_to(&buffer[..len], source).await.unwrap();
        });
        let exchange = udp_exchange(address, b"hello", true, std::time::Duration::from_secs(5))
            .await
            .unwrap();
        task.await.unwrap();
        assert_eq!(exchange.outcome, UdpExchangeOutcome::Response);
        assert_eq!(exchange.transmitted_bytes, 5);
        assert_eq!(exchange.response_bytes, Some(5));
        assert_eq!(exchange.response_sample, Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn udp_exchange_reports_silence_as_timeout_not_failure() {
        let silent = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = silent.local_addr().unwrap();
        let exchange = udp_exchange(
            address,
            b"ping",
            true,
            std::time::Duration::from_millis(200),
        )
        .await
        .unwrap();
        assert_eq!(exchange.outcome, UdpExchangeOutcome::Timeout);
        assert_eq!(exchange.transmitted_bytes, 4);
        drop(silent);
    }

    #[tokio::test]
    async fn udp_exchange_rejects_oversize_payload_before_socket_use() {
        let target: std::net::SocketAddr = "127.0.0.1:9".parse().unwrap();
        let payload = vec![0_u8; MAX_UDP_PAYLOAD_BYTES + 1];
        let result = udp_exchange(target, &payload, false, std::time::Duration::from_secs(1)).await;
        assert!(result.is_err());
    }

    fn silent_probe(ttl: u8) -> trippy_core::ProbeStatus {
        trippy_core::ProbeStatus::Awaited(trippy_core::Probe {
            sequence: trippy_core::Sequence(2),
            identifier: trippy_core::TraceId(0),
            src_port: trippy_core::Port(33434),
            dest_port: trippy_core::Port(33434),
            ttl: trippy_core::TimeToLive(ttl),
            round: trippy_core::RoundId(0),
            sent: std::time::SystemTime::UNIX_EPOCH,
            flags: trippy_core::Flags::empty(),
        })
    }

    fn observation(
        ttl: u8,
        responder: Option<IpAddr>,
        rtt_micros: Option<u64>,
        outcome: TraceProbeOutcome,
    ) -> TraceProbeObservation {
        TraceProbeObservation {
            ttl,
            responder,
            rtt_micros,
            outcome,
        }
    }

    /// Collect a reply-bearing backend probe state without touching the
    /// network. `ProbeComplete` fields are public, but `IcmpPacketCode` is not
    /// re-exported, so only the `NotApplicable` packet type is literally
    /// constructible: a non-target responder exercises the `Reply` arm and a
    /// target responder exercises `DestinationReached`. The
    /// `TimeExceeded`/`Unreachable`-code extraction arms stay pinned by
    /// review plus the structured summary tests below.
    fn complete_probe(ttl: u8, host: IpAddr) -> trippy_core::ProbeStatus {
        trippy_core::ProbeStatus::Complete(trippy_core::ProbeComplete {
            sequence: trippy_core::Sequence(2),
            identifier: trippy_core::TraceId(0),
            src_port: trippy_core::Port(43534),
            dest_port: trippy_core::Port(33434),
            ttl: trippy_core::TimeToLive(ttl),
            round: trippy_core::RoundId(0),
            sent: std::time::SystemTime::UNIX_EPOCH,
            host,
            received: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1),
            icmp_packet_type: trippy_core::IcmpPacketType::NotApplicable,
            tos: None,
            expected_udp_checksum: None,
            actual_udp_checksum: None,
            extensions: None,
        })
    }

    #[test]
    fn trace_summary_preserves_ordered_hops_across_rounds() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let router: IpAddr = "192.168.182.1".parse().unwrap();
        let report = TraceReport {
            rounds: vec![
                TraceRoundObservation {
                    reason: TraceRoundReason::TimeLimit,
                    probes: vec![
                        TraceProbeObservation {
                            ttl: 1,
                            responder: Some(router),
                            rtt_micros: Some(5000),
                            outcome: TraceProbeOutcome::TimeExceeded,
                        },
                        TraceProbeObservation {
                            ttl: 2,
                            responder: Some(target),
                            rtt_micros: Some(150),
                            outcome: TraceProbeOutcome::DestinationReached,
                        },
                    ],
                },
                TraceRoundObservation {
                    reason: TraceRoundReason::TargetFound,
                    probes: vec![
                        TraceProbeObservation {
                            ttl: 1,
                            responder: Some(router),
                            rtt_micros: Some(5100),
                            outcome: TraceProbeOutcome::TimeExceeded,
                        },
                        TraceProbeObservation {
                            ttl: 2,
                            responder: Some(target),
                            rtt_micros: Some(140),
                            outcome: TraceProbeOutcome::DestinationReached,
                        },
                    ],
                },
            ],
            completed: true,
        };
        let summary = summarize_trace(target, 30, &report);
        assert_eq!(
            summary.termination,
            TraceTerminationSummary::DestinationReached
        );
        assert_eq!(summary.hops.len(), 2);
        assert_eq!(summary.hops[0].hop, 1);
        assert_eq!(summary.hops[1].hop, 2);
        assert_eq!(summary.hops[0].attempts.len(), 2);
        assert_eq!(summary.hops[0].attempts[0].rtt_micros, Some(5000));
        assert_eq!(summary.hops[1].attempts[1].responder, Some(target));
    }

    #[test]
    fn trace_summary_keeps_silent_intermediate_hop_as_evidence() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let reached = complete_probe(3, target);
        let probes = vec![silent_probe(2), silent_probe(2), reached];
        let round = trippy_core::Round::new(
            &probes,
            trippy_core::TimeToLive(3),
            trippy_core::CompletionReason::TargetFound,
        );
        let observed = observe_round(target, &round);
        assert_eq!(observed.reason, TraceRoundReason::TargetFound);
        assert_eq!(observed.probes.len(), 3);
        assert_eq!(observed.probes[0].outcome, TraceProbeOutcome::TimedOut);
        assert_eq!(observed.probes[0].responder, None);
        assert_eq!(observed.probes[0].rtt_micros, None);
        assert_eq!(
            observed.probes[2].outcome,
            TraceProbeOutcome::DestinationReached
        );
        assert!(observed.probes[2].rtt_micros.is_some());
        let report = TraceReport {
            rounds: vec![TraceRoundObservation {
                reason: observed.reason,
                probes: observed.probes,
            }],
            completed: true,
        };
        let summary = summarize_trace(target, 30, &report);
        assert_eq!(summary.hops.len(), 2);
        assert_eq!(summary.hops[0].hop, 2);
        assert_eq!(summary.hops[0].attempts.len(), 2);
        assert_eq!(
            summary.hops[0].attempts[0].outcome,
            TraceProbeOutcome::TimedOut
        );
        assert_eq!(summary.hops[1].hop, 3);
        assert_eq!(
            summary.termination,
            TraceTerminationSummary::DestinationReached
        );
    }

    #[test]
    fn trace_summary_reports_unreachable_path_explicitly() {
        let target: IpAddr = "203.0.113.9".parse().unwrap();
        let router: IpAddr = "192.168.182.1".parse().unwrap();
        // A non-target responder with a non-ICMP reply is router-reported
        // evidence, not a reached destination. The `Unreachable`-code
        // extraction arm itself stays review-pinned (`IcmpPacketCode` has no
        // public constructor); termination below proves the summary honors a
        // `DestinationUnreachable` attempt wherever it was extracted.
        let observed = observe_probe(target, &complete_probe(2, router)).unwrap();
        assert_eq!(observed.outcome, TraceProbeOutcome::Reply);
        assert_eq!(observed.responder, Some(router));
        assert_eq!(observed.rtt_micros, Some(1000));
        let report = TraceReport {
            rounds: vec![TraceRoundObservation {
                reason: TraceRoundReason::TimeLimit,
                probes: vec![
                    observation(1, Some(router), Some(5000), TraceProbeOutcome::TimeExceeded),
                    observation(
                        2,
                        Some(router),
                        Some(1100),
                        TraceProbeOutcome::DestinationUnreachable,
                    ),
                ],
            }],
            completed: true,
        };
        let summary = summarize_trace(target, 30, &report);
        assert_eq!(summary.termination, TraceTerminationSummary::Unreachable);
        assert_eq!(
            summary.hops[1].attempts[0].outcome,
            TraceProbeOutcome::DestinationUnreachable
        );
    }

    #[test]
    fn trace_summary_terminates_at_max_hops_without_target() {
        let target: IpAddr = "203.0.113.9".parse().unwrap();
        let router: IpAddr = "192.168.182.1".parse().unwrap();
        let report = TraceReport {
            rounds: vec![TraceRoundObservation {
                reason: TraceRoundReason::TimeLimit,
                probes: vec![
                    observation(1, Some(router), Some(5000), TraceProbeOutcome::TimeExceeded),
                    observation(2, Some(router), Some(5100), TraceProbeOutcome::TimeExceeded),
                ],
            }],
            completed: true,
        };
        let summary = summarize_trace(target, 2, &report);
        assert_eq!(summary.termination, TraceTerminationSummary::MaxHops);
        assert_eq!(summary.hops.len(), 2);
    }

    #[test]
    fn trace_summary_keeps_partial_rounds_on_deadline() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let report = TraceReport {
            rounds: vec![TraceRoundObservation {
                reason: TraceRoundReason::TimeLimit,
                probes: vec![TraceProbeObservation {
                    ttl: 1,
                    responder: None,
                    rtt_micros: None,
                    outcome: TraceProbeOutcome::TimedOut,
                }],
            }],
            completed: false,
        };
        let summary = summarize_trace(target, 30, &report);
        assert_eq!(summary.termination, TraceTerminationSummary::Deadline);
        assert_eq!(summary.hops.len(), 1);
    }

    #[test]
    fn trace_summary_ignores_out_of_range_ttl_defensively() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let report = TraceReport {
            rounds: vec![TraceRoundObservation {
                reason: TraceRoundReason::TimeLimit,
                probes: vec![
                    TraceProbeObservation {
                        ttl: 0,
                        responder: None,
                        rtt_micros: None,
                        outcome: TraceProbeOutcome::TimedOut,
                    },
                    TraceProbeObservation {
                        ttl: 65,
                        responder: None,
                        rtt_micros: None,
                        outcome: TraceProbeOutcome::TimedOut,
                    },
                    TraceProbeObservation {
                        ttl: 1,
                        responder: Some(target),
                        rtt_micros: Some(100),
                        outcome: TraceProbeOutcome::DestinationReached,
                    },
                ],
            }],
            completed: true,
        };
        let summary = summarize_trace(target, 64, &report);
        assert_eq!(summary.hops.len(), 1);
        assert_eq!(summary.hops[0].hop, 1);
    }

    #[test]
    fn trace_probe_maps_constructed_reply_and_skips_unsent_probes() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let state = complete_probe(1, target);
        let observed = observe_probe(target, &state).unwrap();
        assert_eq!(observed.outcome, TraceProbeOutcome::DestinationReached);
        assert_eq!(observed.responder, Some(target));
        assert_eq!(observed.rtt_micros, Some(1000));
        let probes = vec![
            state,
            trippy_core::ProbeStatus::NotSent,
            trippy_core::ProbeStatus::Skipped,
        ];
        let round = trippy_core::Round::new(
            &probes,
            trippy_core::TimeToLive(1),
            trippy_core::CompletionReason::TargetFound,
        );
        let observed = observe_round(target, &round);
        assert_eq!(observed.probes.len(), 1);
    }

    #[test]
    fn trace_probe_tolerates_clock_skew_without_rtt() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let trippy_core::ProbeStatus::Complete(mut complete) = complete_probe(1, target) else {
            panic!("constructed fixture must complete");
        };
        complete.received = std::time::SystemTime::UNIX_EPOCH;
        complete.sent = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1);
        let observed =
            observe_probe(target, &trippy_core::ProbeStatus::Complete(complete)).unwrap();
        assert_eq!(observed.rtt_micros, None);
        assert_eq!(observed.outcome, TraceProbeOutcome::DestinationReached);
    }

    #[test]
    fn trace_backend_errors_map_to_bounded_kinds() {
        // `PrivilegeError` carries a platform-specific inner error with no
        // public constructor on this host, so only the fallback arm is
        // directly constructible; the privilege arm is review-covered.
        let local = map_trace_error(&trippy_core::Error::Other("backend failed".into()));
        assert_eq!(local.kind, NativeErrorKind::Io);
    }

    #[test]
    fn trace_path_rejects_unknown_interface_as_local_failure() {
        let tracer = trippy_core::Builder::new("127.0.0.1".parse().unwrap())
            .privilege_mode(trippy_core::PrivilegeMode::Unprivileged)
            .protocol(trippy_core::Protocol::Udp)
            .interface(Some("eggprobe-nonexistent-interface"))
            .max_rounds(Some(1))
            .build();
        match tracer {
            Ok(tracer) => {
                let run = tracer.run();
                assert!(run.is_err(), "bogus interface must fail the trace");
                assert_eq!(map_trace_error(&run.unwrap_err()).kind, NativeErrorKind::Io);
            }
            Err(error) => assert_eq!(
                map_trace_error(&error).kind,
                NativeErrorKind::Io,
                "bogus interface must normalize to a bounded local failure"
            ),
        }
    }

    #[test]
    fn trace_privilege_selection_uses_unprivileged_mode_where_documented() {
        // macOS-style selection: documented unprivileged support wins without
        // consulting host privilege facts.
        let without = select_privilege_mode(false, None);
        assert_eq!(without, Err(NativeErrorKind::PermissionDenied));
        let macos = select_privilege_mode(
            true,
            Some(PrivilegeFacts {
                has_privileges: false,
            }),
        );
        assert_eq!(macos, Ok(TracePrivilegeMode::Unprivileged));
    }

    #[test]
    fn trace_privilege_selection_requires_effective_privilege_elsewhere() {
        // Linux privilege available: permitted/effective `CAP_NET_RAW`
        // already present selects the privileged backend mode.
        let granted = select_privilege_mode(
            false,
            Some(PrivilegeFacts {
                has_privileges: true,
            }),
        );
        assert_eq!(granted, Ok(TracePrivilegeMode::Privileged));
        // Linux/Windows privilege unavailable: typed denial, never a silent
        // fallback or a generic failure.
        let denied = select_privilege_mode(
            false,
            Some(PrivilegeFacts {
                has_privileges: false,
            }),
        );
        assert_eq!(denied, Err(NativeErrorKind::PermissionDenied));
    }

    #[test]
    fn trace_privilege_selection_denies_boundedly_on_discovery_failure() {
        // A discovery error (facts unavailable) denies with the same bounded
        // category; no dependency text escapes and the deficit never becomes
        // `Unsupported`.
        let denied = select_privilege_mode(false, None);
        assert_eq!(denied, Err(NativeErrorKind::PermissionDenied));
        assert_ne!(denied, Err(NativeErrorKind::Unsupported));
    }

    #[test]
    fn trace_capability_agrees_with_privilege_decision() {
        // The public capability oracle reports exactly what `trace_path`
        // would decide: executable where the mode selection succeeds.
        let expected = match current_privilege_mode() {
            Ok(_) => TraceCapability::Executable,
            Err(_) => TraceCapability::PermissionDenied,
        };
        assert_eq!(trace_capability(), expected);
    }

    #[tokio::test]
    async fn trace_path_reaches_loopback_when_executable() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        match trace_capability() {
            TraceCapability::Executable => {
                let report = trace_path(
                    target,
                    3,
                    1,
                    std::time::Duration::from_millis(300),
                    std::time::Duration::from_secs(3),
                    std::time::Duration::from_secs(10),
                )
                .await
                .unwrap();
                assert!(report.completed);
                let summary = summarize_trace(target, 3, &report);
                assert_eq!(
                    summary.termination,
                    TraceTerminationSummary::DestinationReached
                );
                assert!(!summary.hops.is_empty());
                assert_eq!(summary.hops[0].hop, 1);
                assert_eq!(summary.hops[0].attempts[0].responder, Some(target));
            }
            TraceCapability::PermissionDenied => {
                let error = trace_path(
                    target,
                    3,
                    1,
                    std::time::Duration::from_millis(300),
                    std::time::Duration::from_secs(3),
                    std::time::Duration::from_secs(10),
                )
                .await
                .expect_err("trace without privilege must deny, not hang");
                assert_eq!(error.kind, NativeErrorKind::PermissionDenied);
            }
        }
    }

    #[tokio::test]
    async fn trace_path_reaches_ipv6_loopback_when_executable() {
        let target: IpAddr = "::1".parse().unwrap();
        match trace_capability() {
            TraceCapability::Executable => {
                let report = trace_path(
                    target,
                    3,
                    1,
                    std::time::Duration::from_millis(300),
                    std::time::Duration::from_secs(3),
                    std::time::Duration::from_secs(10),
                )
                .await
                .unwrap();
                assert!(report.completed);
                let summary = summarize_trace(target, 3, &report);
                assert_eq!(
                    summary.termination,
                    TraceTerminationSummary::DestinationReached
                );
            }
            TraceCapability::PermissionDenied => {
                let error = trace_path(
                    target,
                    3,
                    1,
                    std::time::Duration::from_millis(300),
                    std::time::Duration::from_secs(3),
                    std::time::Duration::from_secs(10),
                )
                .await
                .expect_err("trace without privilege must deny, not hang");
                assert_eq!(error.kind, NativeErrorKind::PermissionDenied);
            }
        }
    }

    #[tokio::test]
    async fn trace_path_reports_partial_report_when_bounded_wait_expires() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let report = trace_path(
            target,
            3,
            1,
            std::time::Duration::from_millis(300),
            std::time::Duration::from_secs(3),
            std::time::Duration::ZERO,
        )
        .await
        .unwrap();
        assert!(!report.completed);
    }
}

/// Deterministic backend for contract and engine tests.
#[derive(Clone, Debug)]
pub struct FakeBackend {
    /// Result returned for each request.
    pub result: Result<NativeObservation, NativeError>,
}

impl NativeBackend for FakeBackend {
    fn execute(&self, _request: NativeRequest) -> NativeFuture<'_> {
        let result = self.result.clone();
        Box::pin(async move { result })
    }
}

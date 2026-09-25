//! Core-owned asynchronous probe execution.

use std::{
    error::Error as StdError,
    net::IpAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use eggfetch_core::{DialError, DialErrorKind, DialFuture, DialStream, DialTarget, Dialer};
use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{lookup_host, TcpStream},
};
use tokio_rustls::TlsConnector;

use crate::{
    domain::{
        error::{DiagnosticError, DiagnosticErrorKind, DiagnosticStage},
        plan::{ProbePlan, ProbeSpec},
        probe::{
            DnsEvidence, HttpEvidence, ProbeEvidence, ProbeKind, ProbeResult, ProbeStatus,
            TcpEvidence, TlsEvidence, UdpEvidence, UdpOutcome,
        },
        report::{ProbeReport, ReportStatus, ToolProvenance},
        route::RouteSpec,
        timing::{DurationMicros, Timing},
    },
    ToolVersion,
};

const MAX_DNS_ANSWERS: usize = 32;
const MAX_BODY_SAMPLE: usize = 4096;
static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);

/// Policy applied before direct target execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetPolicy {
    /// Permit loopback, LAN, and documentation addresses for diagnostics.
    AllowPrivate,
    /// Reject private/reserved destinations after resolution.
    Strict,
}

/// A core-owned execution engine. It contains no CLI or renderer behavior.
#[derive(Clone, Debug)]
pub struct ProbeEngine {
    /// Address policy for direct destinations.
    pub target_policy: TargetPolicy,
}

impl Default for ProbeEngine {
    fn default() -> Self {
        Self {
            target_policy: TargetPolicy::AllowPrivate,
        }
    }
}

impl ProbeEngine {
    /// Execute a plan with one finite outer deadline.
    pub async fn execute(&self, plan: ProbePlan) -> ProbeReport {
        let tool = tool();
        let execution_id = execution_id();
        if let Err(error) = plan.validate() {
            let mut report =
                ProbeReport::from_plan(&plan, tool, execution_id, ReportStatus::Failed, vec![]);
            report.warnings.push(error.to_string());
            return report;
        }
        let deadline = Duration::from_micros(plan.execution.deadline.as_micros());
        let deadline_at = tokio::time::Instant::now() + deadline;
        if let Ok(report) = tokio::time::timeout_at(
            deadline_at,
            self.execute_inner(&plan, execution_id.clone(), deadline_at),
        )
        .await
        {
            report
        } else {
            let mut report = ProbeReport::from_plan(
                &plan,
                tool,
                execution_id,
                ReportStatus::Failed,
                deadline_results(&plan),
            );
            report.warnings.push("execution deadline exceeded".into());
            report
        }
    }

    async fn execute_inner(
        &self,
        plan: &ProbePlan,
        execution_id: String,
        deadline_at: tokio::time::Instant,
    ) -> ProbeReport {
        let repetitions = usize::try_from(plan.execution.repetitions).unwrap_or(usize::MAX);
        let mut results = Vec::with_capacity(plan.probes.len().saturating_mul(repetitions));
        for _ in 0..plan.execution.repetitions {
            for spec in &plan.probes {
                results.push(self.execute_probe(plan, spec, deadline_at).await);
            }
        }
        let status = if results.iter().any(|r| r.status == ProbeStatus::Failed) {
            ReportStatus::Failed
        } else if results.iter().any(|r| r.status == ProbeStatus::Unsupported) {
            ReportStatus::Unsupported
        } else if results.iter().any(|r| r.status == ProbeStatus::Cancelled) {
            ReportStatus::Cancelled
        } else {
            ReportStatus::Ok
        };
        ProbeReport::from_plan(plan, tool(), execution_id, status, results)
    }

    async fn execute_probe(
        &self,
        plan: &ProbePlan,
        spec: &ProbeSpec,
        deadline_at: tokio::time::Instant,
    ) -> ProbeResult {
        let started = Instant::now();
        let kind = match spec {
            ProbeSpec::Dns => ProbeKind::Dns,
            ProbeSpec::Tcp { .. } => ProbeKind::Tcp,
            ProbeSpec::Tls { .. } => ProbeKind::Tls,
            ProbeSpec::Http { .. } => ProbeKind::Http,
            ProbeSpec::Route => ProbeKind::Route,
            ProbeSpec::IcmpEcho { .. } => ProbeKind::IcmpEcho,
            ProbeSpec::Udp { .. } => ProbeKind::Udp,
            ProbeSpec::Trace { .. } => ProbeKind::Trace,
            ProbeSpec::PathMtu { .. } => ProbeKind::PathMtu,
        };
        let is_http = kind == ProbeKind::Http;
        let outcome = match spec {
            ProbeSpec::Dns => self.dns(plan).await,
            ProbeSpec::Tcp { port } => self.tcp(plan, *port, deadline_at).await,
            ProbeSpec::Tls { port, server_name } => {
                self.tls(plan, *port, server_name.as_deref(), deadline_at)
                    .await
            }
            ProbeSpec::Http { url, method } => self.http(plan, url, method, deadline_at).await,
            ProbeSpec::Route => self.route(plan).await,
            ProbeSpec::Udp {
                port,
                payload,
                receive,
            } => self.udp(plan, *port, payload, *receive, deadline_at).await,
            ProbeSpec::Trace {
                max_hops,
                attempts_per_hop,
            } => {
                self.trace(plan, *max_hops, *attempts_per_hop, deadline_at)
                    .await
            }
            ProbeSpec::IcmpEcho { .. } | ProbeSpec::PathMtu { .. } => {
                return ProbeResult {
                    kind,
                    status: ProbeStatus::Unsupported,
                    timing: Some(Timing {
                        total: elapsed_since(started),
                        phases: vec![],
                    }),
                    error: Some(simple_error(
                        DiagnosticErrorKind::Unsupported,
                        DiagnosticStage::Other,
                        "native capability is not available in this build",
                    )),
                    evidence: None,
                    unavailable: vec![],
                    findings: vec![],
                };
            }
        };
        let elapsed = DurationMicros::from_micros(
            u64::try_from(started.elapsed().as_micros().min(u128::from(u64::MAX)))
                .unwrap_or(u64::MAX),
        );
        match outcome {
            Ok(evidence) => ProbeResult {
                kind,
                status: ProbeStatus::Ok,
                timing: Some(Timing {
                    total: elapsed,
                    phases: vec![],
                }),
                error: None,
                evidence: Some(evidence),
                unavailable: if is_http {
                    vec![
                        "dns_phase_timing".into(),
                        "tcp_phase_timing".into(),
                        "tls_phase_timing".into(),
                    ]
                } else {
                    vec![]
                },
                findings: vec![],
            },
            Err(error) => ProbeResult {
                kind,
                status: if error.kind == DiagnosticErrorKind::Unsupported {
                    ProbeStatus::Unsupported
                } else {
                    ProbeStatus::Failed
                },
                timing: Some(Timing {
                    total: elapsed,
                    phases: vec![],
                }),
                error: Some(error),
                evidence: None,
                unavailable: vec![],
                findings: vec![],
            },
        }
    }

    async fn addresses(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Vec<std::net::SocketAddr>, DiagnosticError> {
        resolve_addresses(self.target_policy, host, port).await
    }

    async fn route(&self, plan: &ProbePlan) -> Result<ProbeEvidence, DiagnosticError> {
        if !matches!(plan.route, RouteSpec::Direct) {
            return Err(simple_error(
                DiagnosticErrorKind::Unsupported,
                DiagnosticStage::RouteInspection,
                "native diagnostics require the direct route",
            ));
        }
        let destination = self
            .addresses(&plan.target.host, plan.target.port.unwrap_or(33434))
            .await?
            .into_iter()
            .next()
            .expect("address resolver returns a non-empty list")
            .ip();
        if destination.is_unspecified()
            || destination.is_multicast()
            || matches!(destination, IpAddr::V4(address) if address == std::net::Ipv4Addr::BROADCAST)
            || matches!(destination, IpAddr::V6(address) if (address.segments()[0] & 0xffc0) == 0xfe80)
        {
            return Err(simple_error(
                DiagnosticErrorKind::Policy,
                DiagnosticStage::RouteInspection,
                "destination address class is not supported",
            ));
        }
        let observed = eggprobe_native::inspect_route(destination)
            .await
            .map_err(|error| {
                let kind = match error.kind {
                    eggprobe_native::NativeErrorKind::PermissionDenied => {
                        DiagnosticErrorKind::PermissionDenied
                    }
                    eggprobe_native::NativeErrorKind::Unsupported => {
                        DiagnosticErrorKind::Unsupported
                    }
                    eggprobe_native::NativeErrorKind::Timeout => DiagnosticErrorKind::Timeout,
                    eggprobe_native::NativeErrorKind::Unreachable => {
                        DiagnosticErrorKind::NetworkUnreachable
                    }
                    eggprobe_native::NativeErrorKind::Io => DiagnosticErrorKind::Io,
                };
                simple_error(
                    kind,
                    DiagnosticStage::RouteInspection,
                    "route inspection failed",
                )
            })?;
        let correlation = if observed.ambiguous {
            crate::domain::probe::RouteCorrelation::Ambiguous
        } else if observed.source.is_some() && observed.interface.is_some() {
            crate::domain::probe::RouteCorrelation::ObservedSource
        } else if !observed.candidates.is_empty() {
            crate::domain::probe::RouteCorrelation::CorrelatedCandidates
        } else {
            crate::domain::probe::RouteCorrelation::Unavailable
        };
        Ok(ProbeEvidence::Route(crate::domain::probe::RouteEvidence {
            target: destination.to_string(),
            source: observed.source.map(|address| address.to_string()),
            interface: observed.interface.map(|interface| {
                crate::domain::probe::InterfaceEvidence {
                    index: Some(interface.index),
                    name: Some(interface.name),
                    addresses: interface
                        .addresses
                        .into_iter()
                        .map(|address| address.to_string())
                        .collect(),
                    up: Some(interface.up),
                    mtu: interface.mtu,
                }
            }),
            routes: observed
                .candidates
                .into_iter()
                .map(|candidate| crate::domain::probe::RouteCandidate {
                    destination: candidate.destination,
                    interface_index: candidate.interface_index,
                    gateway: candidate.gateway.map(|address| address.to_string()),
                    metric: candidate.metric,
                    table: candidate.table,
                    protocol: candidate.protocol,
                    scope: candidate.scope,
                })
                .collect(),
            routes_truncated: observed.candidates_truncated,
            correlation,
        }))
    }

    async fn udp(
        &self,
        plan: &ProbePlan,
        port: u16,
        payload: &[u8],
        receive: bool,
        deadline_at: tokio::time::Instant,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        if !matches!(plan.route, RouteSpec::Direct) {
            return Err(simple_error(
                DiagnosticErrorKind::Unsupported,
                DiagnosticStage::PacketExchange,
                "native diagnostics require the direct route",
            ));
        }
        let destination = self
            .addresses(&plan.target.host, port)
            .await?
            .into_iter()
            .next()
            .expect("address resolver returns a non-empty list")
            .ip();
        if destination.is_unspecified()
            || destination.is_multicast()
            || matches!(destination, IpAddr::V4(address) if address == std::net::Ipv4Addr::BROADCAST)
            || matches!(destination, IpAddr::V4(address) if address.is_link_local())
            || matches!(destination, IpAddr::V6(address) if (address.segments()[0] & 0xffc0) == 0xfe80)
        {
            return Err(simple_error(
                DiagnosticErrorKind::Policy,
                DiagnosticStage::PacketExchange,
                "destination address class is not supported",
            ));
        }
        let timeout = deadline_at.saturating_duration_since(tokio::time::Instant::now());
        if timeout.is_zero() {
            return Err(simple_error(
                DiagnosticErrorKind::Timeout,
                DiagnosticStage::Deadline,
                "execution deadline exceeded",
            ));
        }
        let observed = eggprobe_native::udp_exchange(
            std::net::SocketAddr::new(destination, port),
            payload,
            receive,
            timeout,
        )
        .await
        .map_err(|error| {
            let kind = match error.kind {
                eggprobe_native::NativeErrorKind::PermissionDenied => {
                    DiagnosticErrorKind::PermissionDenied
                }
                eggprobe_native::NativeErrorKind::Unsupported => DiagnosticErrorKind::Unsupported,
                eggprobe_native::NativeErrorKind::Timeout => DiagnosticErrorKind::Timeout,
                eggprobe_native::NativeErrorKind::Unreachable => {
                    DiagnosticErrorKind::NetworkUnreachable
                }
                eggprobe_native::NativeErrorKind::Io => DiagnosticErrorKind::Io,
            };
            simple_error(kind, DiagnosticStage::PacketExchange, "UDP exchange failed")
        })?;
        // A transmitted datagram followed by timeout or unreachable feedback is
        // a completed diagnostic observation, not a local execution failure.
        // Only bind/connect/send failures above reach the Failed path.
        let outcome = match observed.outcome {
            eggprobe_native::UdpExchangeOutcome::Sent => UdpOutcome::Sent,
            eggprobe_native::UdpExchangeOutcome::Response => UdpOutcome::Response,
            eggprobe_native::UdpExchangeOutcome::Unreachable => UdpOutcome::Unreachable,
            eggprobe_native::UdpExchangeOutcome::Timeout => UdpOutcome::Timeout,
        };
        Ok(ProbeEvidence::Udp(UdpEvidence {
            local: observed.local.map(|address| address.to_string()),
            transmitted_bytes: observed.transmitted_bytes,
            outcome,
            response_source: observed.response_source.map(|address| address.to_string()),
            response_bytes: observed.response_bytes,
            response_sample: observed.response_sample,
        }))
    }

    async fn trace(
        &self,
        plan: &ProbePlan,
        max_hops: u8,
        attempts_per_hop: u8,
        deadline_at: tokio::time::Instant,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        if !matches!(plan.route, RouteSpec::Direct) {
            return Err(simple_error(
                DiagnosticErrorKind::Unsupported,
                DiagnosticStage::HopProbe,
                "native diagnostics require the direct route",
            ));
        }
        let destination = self
            .addresses(&plan.target.host, plan.target.port.unwrap_or(0))
            .await?
            .into_iter()
            .next()
            .expect("address resolver returns a non-empty list")
            .ip();
        if destination.is_unspecified()
            || destination.is_multicast()
            || matches!(destination, IpAddr::V4(address) if address == std::net::Ipv4Addr::BROADCAST)
            || matches!(destination, IpAddr::V4(address) if address.is_link_local())
            || matches!(destination, IpAddr::V6(address) if (address.segments()[0] & 0xffc0) == 0xfe80)
        {
            return Err(simple_error(
                DiagnosticErrorKind::Policy,
                DiagnosticStage::HopProbe,
                "destination address class is not supported",
            ));
        }
        let timeout = deadline_at.saturating_duration_since(tokio::time::Instant::now());
        if timeout.is_zero() {
            return Err(simple_error(
                DiagnosticErrorKind::Timeout,
                DiagnosticStage::Deadline,
                "execution deadline exceeded",
            ));
        }
        // Divide the remaining outer budget across rounds so the backend can
        // never outlive the plan deadline by design; the engine timeout stays
        // a defensive backstop. A small per-round margin keeps fully silent
        // traces inside the budget so they report MaxHops instead of racing
        // the outer timeout. The read timeout only sets backend loop-wakeup
        // granularity (it never cuts off slow replies), so it stays small.
        let rounds = u32::from(attempts_per_hop.max(1));
        let per_round = (timeout / rounds).max(std::time::Duration::from_millis(1));
        let max_round_duration = per_round
            .saturating_sub(std::time::Duration::from_millis(100))
            .max(std::time::Duration::from_millis(1));
        let read_timeout = std::time::Duration::from_millis(100).min(max_round_duration);
        let report = eggprobe_native::trace_path(
            destination,
            max_hops,
            attempts_per_hop,
            read_timeout,
            max_round_duration,
            timeout,
        )
        .await
        .map_err(|error| {
            let kind = match error.kind {
                eggprobe_native::NativeErrorKind::PermissionDenied => {
                    DiagnosticErrorKind::PermissionDenied
                }
                eggprobe_native::NativeErrorKind::Unsupported => DiagnosticErrorKind::Unsupported,
                eggprobe_native::NativeErrorKind::Timeout => DiagnosticErrorKind::Timeout,
                eggprobe_native::NativeErrorKind::Unreachable => {
                    DiagnosticErrorKind::NetworkUnreachable
                }
                eggprobe_native::NativeErrorKind::Io => DiagnosticErrorKind::Io,
            };
            // Privilege denial carries its own fixed message so operators can
            // distinguish a host policy refusal from a local failure without
            // ever receiving dependency or OS error text.
            let message = if kind == DiagnosticErrorKind::PermissionDenied {
                eggprobe_native::TRACE_PERMISSION_MESSAGE
            } else {
                "UDP trace failed"
            };
            simple_error(kind, DiagnosticStage::HopProbe, message)
        })?;
        let summary = eggprobe_native::summarize_trace(destination, max_hops, &report);
        let termination = trace_termination(report.completed, summary.termination);
        Ok(ProbeEvidence::Trace(crate::domain::probe::TraceEvidence {
            destination: destination.to_string(),
            hops: summary
                .hops
                .into_iter()
                .map(|hop| crate::domain::probe::TraceHop {
                    hop: hop.hop,
                    attempts: hop
                        .attempts
                        .into_iter()
                        .map(|attempt| crate::domain::probe::TraceAttempt {
                            responder: attempt.responder.map(|address| address.to_string()),
                            rtt_micros: attempt.rtt_micros,
                            outcome: trace_outcome(attempt.outcome),
                        })
                        .collect(),
                })
                .collect(),
            termination,
        }))
    }

    async fn dns(&self, plan: &ProbePlan) -> Result<ProbeEvidence, DiagnosticError> {
        let addresses = self
            .addresses(&plan.target.host, plan.target.port.unwrap_or(0))
            .await?;
        Ok(ProbeEvidence::Dns(DnsEvidence {
            addresses: addresses.into_iter().map(|a| a.ip().to_string()).collect(),
            resolution_scope: crate::domain::probe::DnsResolutionScope::Client,
        }))
    }

    async fn tcp(
        &self,
        plan: &ProbePlan,
        port: u16,
        deadline_at: tokio::time::Instant,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        if let RouteSpec::Eggress(route) = &plan.route {
            let connector = egress_connector(&route.expression).map_err(|()| {
                simple_error(
                    DiagnosticErrorKind::Protocol,
                    DiagnosticStage::HopHandshake,
                    "invalid or unsupported Eggress route",
                )
            })?;
            let (stream, info) = connector
                .connect_tcp_timeout_detailed(
                    &plan.target.host,
                    port,
                    route_connect_timeout(deadline_at),
                )
                .await
                .map_err(|error| diagnostic_from_route_error(&error))?;
            drop(stream);
            return Ok(ProbeEvidence::Tcp(TcpEvidence {
                peer: info.peer_addr.map(|p| p.to_string()),
                local: info.local_addr.map(|p| p.to_string()),
                attempts: 1,
            }));
        }
        let addresses = self.addresses(&plan.target.host, port).await?;
        let mut last = None;
        for (attempt, address) in addresses.into_iter().enumerate() {
            let result = TcpStream::connect(address).await;
            match result {
                Ok(stream) => {
                    let local = stream.local_addr().ok().map(|address| address.to_string());
                    drop(stream);
                    return Ok(ProbeEvidence::Tcp(TcpEvidence {
                        peer: Some(address.to_string()),
                        local,
                        attempts: u32::try_from(attempt + 1).unwrap_or(u32::MAX),
                    }));
                }
                Err(error) => {
                    last = Some((u32::try_from(attempt + 1).unwrap_or(u32::MAX), error));
                }
            }
        }
        let (attempt, error) = last.expect("addresses is non-empty");
        Err(normalize_io(
            &error,
            DiagnosticStage::DirectConnect,
            Some(attempt),
        ))
    }

    async fn tls(
        &self,
        plan: &ProbePlan,
        port: u16,
        server_name: Option<&str>,
        deadline_at: tokio::time::Instant,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        let name = ServerName::try_from(server_name.unwrap_or(&plan.target.host).to_owned())
            .map_err(|_| DiagnosticError {
                kind: DiagnosticErrorKind::Tls,
                stage: DiagnosticStage::TlsHandshake,
                message: "invalid TLS server name".into(),
                attempt: None,
                route_hop_index: None,
                route_protocol: None,
            })?;
        match &plan.route {
            RouteSpec::Direct => {
                let address = self
                    .addresses(&plan.target.host, port)
                    .await?
                    .into_iter()
                    .next()
                    .expect("addresses is non-empty");
                let stream = TcpStream::connect(address)
                    .await
                    .map_err(|e| normalize_io(&e, DiagnosticStage::DirectConnect, None))?;
                tls_handshake(name, stream).await
            }
            RouteSpec::Eggress(route) => {
                let connector = egress_connector(&route.expression).map_err(|()| {
                    simple_error(
                        DiagnosticErrorKind::Protocol,
                        DiagnosticStage::HopHandshake,
                        "invalid or unsupported Eggress route",
                    )
                })?;
                let (stream, _) = connector
                    .connect_tcp_timeout_detailed(
                        &plan.target.host,
                        port,
                        route_connect_timeout(deadline_at),
                    )
                    .await
                    .map_err(|error| diagnostic_from_route_error(&error))?;
                tls_handshake(name, EgressStream(stream)).await
            }
        }
    }

    async fn http(
        &self,
        plan: &ProbePlan,
        url: &str,
        method: &str,
        deadline_at: tokio::time::Instant,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        let method = eggfetch_core::Method::from_bytes(method.as_bytes()).map_err(|_| {
            simple_error(
                DiagnosticErrorKind::Protocol,
                DiagnosticStage::Request,
                "invalid HTTP method",
            )
        })?;
        let mut builder = eggfetch_core::Client::builder()
            .automatic_decompression(false)
            .max_decoded_body_size(MAX_BODY_SAMPLE);
        match &plan.route {
            RouteSpec::Direct => {
                builder = builder.dialer(PolicyDialer {
                    target_policy: self.target_policy,
                });
            }
            RouteSpec::Eggress(route) => {
                builder = builder.dialer(RouteDialer {
                    connector: Arc::new(egress_connector(&route.expression).map_err(|()| {
                        simple_error(
                            DiagnosticErrorKind::Protocol,
                            DiagnosticStage::HopHandshake,
                            "invalid or unsupported Eggress route",
                        )
                    })?),
                    timeout: route_connect_timeout(deadline_at),
                });
            }
        }
        let client = builder.build();
        let request = client.request(method, url).map_err(|_| {
            simple_error(
                DiagnosticErrorKind::Protocol,
                DiagnosticStage::Request,
                "invalid HTTP URL",
            )
        })?;
        let mut response = request
            .send_detailed()
            .await
            .map_err(|failure| normalize_http_failure(&failure))?;
        let status = response.status().as_u16();
        let protocol = Some(format!("{:?}", response.version()));
        let body = response.bytes().await.map_err(|_| {
            simple_error(
                DiagnosticErrorKind::Other,
                DiagnosticStage::Body,
                "response body failed",
            )
        })?;
        Ok(ProbeEvidence::Http(HttpEvidence {
            status: Some(status),
            protocol,
            body_sample_bytes: u32::try_from(body.len().min(MAX_BODY_SAMPLE)).unwrap_or(u32::MAX),
        }))
    }
}

async fn tls_handshake<S>(
    name: ServerName<'static>,
    stream: S,
) -> Result<ProbeEvidence, DiagnosticError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    tls_handshake_with_roots(name, stream, roots).await
}

async fn tls_handshake_with_roots<S>(
    name: ServerName<'static>,
    stream: S,
    roots: RootCertStore,
) -> Result<ProbeEvidence, DiagnosticError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("ring supports the configured TLS versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
    let mut config = config;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let stream = TlsConnector::from(Arc::new(config))
        .connect(name, stream)
        .await
        .map_err(|_| {
            simple_error(
                DiagnosticErrorKind::Tls,
                DiagnosticStage::TlsHandshake,
                "TLS handshake failed",
            )
        })?;
    let (_, connection) = stream.get_ref();
    Ok(ProbeEvidence::Tls(TlsEvidence {
        version: connection.protocol_version().map(|v| format!("{v:?}")),
        alpn: connection
            .alpn_protocol()
            .map(|v| String::from_utf8_lossy(v).into_owned()),
        cipher_suite: connection
            .negotiated_cipher_suite()
            .map(|v| format!("{v:?}")),
    }))
}

#[cfg(test)]
mod tls_fixture_tests {
    use super::*;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    fn fixture_cert() -> (CertificateDer<'static>, PrivateKeyDer<'static>) {
        let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert = CertificateDer::from(generated.cert.der().to_vec());
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(generated.key_pair.serialize_der()));
        (cert, key)
    }

    fn tls_server_config(
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
        alpn: &[&[u8]],
    ) -> Arc<rustls::ServerConfig> {
        let mut config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring supports the configured TLS versions")
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        config.alpn_protocols = alpn.iter().map(|protocol| protocol.to_vec()).collect();
        Arc::new(config)
    }

    async fn run_fixture(
        server_config: Arc<rustls::ServerConfig>,
        client_name: &str,
        roots: RootCertStore,
    ) -> Result<ProbeEvidence, DiagnosticError> {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let server = tokio::spawn(async move {
            let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
            let _ = acceptor.accept(server).await;
        });
        let name = ServerName::try_from(client_name.to_owned()).unwrap();
        let result = tls_handshake_with_roots(name, client, roots).await;
        server.abort();
        result
    }

    #[tokio::test]
    async fn trusted_tls_reports_negotiated_metadata_and_alpn() {
        let (cert, key) = fixture_cert();
        let mut roots = RootCertStore::empty();
        roots.add(cert.clone()).unwrap();
        let evidence = run_fixture(tls_server_config(cert, key, &[b"h2"]), "localhost", roots)
            .await
            .unwrap();
        let ProbeEvidence::Tls(evidence) = evidence else {
            panic!("expected TLS evidence");
        };
        assert!(evidence.version.is_some());
        assert!(evidence.cipher_suite.is_some());
        assert_eq!(evidence.alpn.as_deref(), Some("h2"));
    }

    #[tokio::test]
    async fn tls_without_server_alpn_reports_absence() {
        let (cert, key) = fixture_cert();
        let mut roots = RootCertStore::empty();
        roots.add(cert.clone()).unwrap();
        let evidence = run_fixture(tls_server_config(cert, key, &[]), "localhost", roots)
            .await
            .unwrap();
        let ProbeEvidence::Tls(evidence) = evidence else {
            panic!("expected TLS evidence");
        };
        assert_eq!(evidence.alpn, None);
    }

    #[tokio::test]
    async fn tls_hostname_mismatch_and_untrusted_issuer_fail_closed() {
        let (cert, key) = fixture_cert();
        let mut trusted_roots = RootCertStore::empty();
        trusted_roots.add(cert.clone()).unwrap();
        let mismatch = run_fixture(
            tls_server_config(cert.clone(), key, &[]),
            "mismatch.local",
            trusted_roots,
        )
        .await;
        assert!(mismatch.is_err());

        let (untrusted_cert, untrusted_key) = fixture_cert();
        let untrusted = run_fixture(
            tls_server_config(untrusted_cert, untrusted_key, &[]),
            "localhost",
            RootCertStore::empty(),
        )
        .await;
        assert!(untrusted.is_err());
    }

    #[tokio::test]
    async fn routed_tls_preserves_sni_and_hostname_verification() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (cert, key) = fixture_cert();
        let mut roots = RootCertStore::empty();
        roots.add(cert.clone()).unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(tls_server_config(cert, key, &[]));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_port = listener.local_addr().unwrap().port();
        let origin = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let _ = acceptor.accept(stream).await;
        });

        let config = eggress_embed::EggressConfig::from_toml_str(
            "version = 1\n\n[[listeners]]\nname = \"proxy\"\nbind = \"127.0.0.1:0\"\nprotocols = [\"socks5\"]\n",
        )
        .unwrap();
        let proxy = eggress_embed::EggressService::new(config)
            .start_blocking()
            .unwrap();
        let proxy_addr = proxy.bound_addresses().listener("proxy").unwrap();
        let connector = eggress_embed::outbound::OutboundConnector::from_pproxy_uri(&format!(
            "socks5://{proxy_addr}"
        ))
        .unwrap();
        let (stream, _) = connector
            .connect_tcp_timeout_detailed("127.0.0.1", origin_port, Duration::from_secs(2))
            .await
            .expect("Eggress should establish the routed TCP stream");
        let name = ServerName::try_from("localhost".to_owned()).unwrap();
        let evidence = tls_handshake_with_roots(name, EgressStream(stream), roots)
            .await
            .expect("trusted routed TLS handshake with localhost SNI");
        assert!(matches!(evidence, ProbeEvidence::Tls(_)));
        proxy.shutdown().await.unwrap();
        origin.abort();
    }

    // Keep both client paths together so the local HTTP/2 fixture compares
    // direct and routed TLS validation under exactly the same origin.
    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn eggfetch_h2_preserves_tls_validation_over_direct_and_eggress_routes() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (cert, key) = fixture_cert();
        let mut server_config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring supports the configured TLS versions")
        .with_no_client_auth()
        .with_single_cert(vec![cert.clone()], key)
        .unwrap();
        server_config.alpn_protocols = vec![b"h2".to_vec()];
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let origin = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let Ok(stream) = acceptor.accept(stream).await else {
                        return;
                    };
                    let Ok(mut connection) = h2::server::handshake(stream).await else {
                        return;
                    };
                    while let Some(Ok((_request, mut respond))) = connection.accept().await {
                        let response = http::Response::builder()
                            .status(200)
                            .body(())
                            .expect("valid fixture response");
                        if respond.send_response(response, true).is_err() {
                            return;
                        }
                    }
                });
            }
        });

        let tls_config = eggfetch_core::TlsConfig::builder()
            .ca_certificate_der(vec![cert.to_vec()])
            .expect("fixture CA is valid")
            .crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .build();
        // Keep localhost as the HTTP origin so Eggfetch performs hostname
        // validation and emits localhost SNI. The fixture dialer maps that
        // name to the loopback listener only for the Eggress test route.
        let url = format!("https://localhost:{port}/qualification");
        let mut direct = eggfetch_core::Client::builder()
            .dialer(PolicyDialer {
                target_policy: TargetPolicy::AllowPrivate,
            })
            .tls_config(tls_config.clone())
            .http_version_policy(eggfetch_core::HttpVersionPolicy::Http2Only)
            .build()
            .request(eggfetch_core::Method::GET, &url)
            .expect("valid direct URL")
            .send_detailed()
            .await
            .expect("trusted direct HTTP/2 request");
        assert_eq!(direct.status().as_u16(), 200);
        assert_eq!(format!("{:?}", direct.version()), "HTTP/2.0");
        let _ = direct.bytes().await.expect("read direct response");

        let config = eggress_embed::EggressConfig::from_toml_str(
            "version = 1\n\n[[listeners]]\nname = \"proxy\"\nbind = \"127.0.0.1:0\"\nprotocols = [\"socks5\"]\n",
        )
        .unwrap();
        let proxy = eggress_embed::EggressService::new(config)
            .start_blocking()
            .unwrap();
        let proxy_addr = proxy.bound_addresses().listener("proxy").unwrap();
        let connector = eggress_embed::outbound::OutboundConnector::from_pproxy_uri(&format!(
            "socks5://{proxy_addr}"
        ))
        .unwrap();
        let mut routed = eggfetch_core::Client::builder()
            .dialer(LocalhostMappedRouteDialer {
                connector: Arc::new(connector),
                timeout: Duration::from_secs(2),
            })
            .tls_config(tls_config)
            .http_version_policy(eggfetch_core::HttpVersionPolicy::Http2Only)
            .build()
            .request(eggfetch_core::Method::GET, &url)
            .expect("valid routed URL")
            .send_detailed()
            .await
            .unwrap_or_else(|failure| {
                let details = failure
                    .error()
                    .custom_transport_error()
                    .and_then(StdError::source)
                    .and_then(|source| {
                        source.downcast_ref::<eggress_embed::outbound::OutboundConnectError>()
                    })
                    .map(|error| {
                        format!(
                            "{} ({} {}) hop={:?} protocol={:?}",
                            error,
                            error.kind(),
                            error.stage(),
                            error.hop_index(),
                            error.protocol()
                        )
                    });
                panic!("trusted routed HTTP/2 request failed: {details:?}");
            });
        assert_eq!(routed.status().as_u16(), 200);
        assert_eq!(format!("{:?}", routed.version()), "HTTP/2.0");
        let _ = routed.bytes().await.expect("read routed response");
        drop(routed);
        proxy.shutdown().await.unwrap();
        origin.abort();
    }

    #[derive(Clone)]
    struct LocalhostMappedRouteDialer {
        connector: Arc<eggress_embed::outbound::OutboundConnector>,
        timeout: Duration,
    }

    impl Dialer for LocalhostMappedRouteDialer {
        fn dial(&self, target: DialTarget) -> DialFuture<'_> {
            Box::pin(async move {
                let host = if target.host() == "localhost" {
                    "127.0.0.1"
                } else {
                    target.host()
                };
                self.connector
                    .connect_tcp_timeout_detailed(host, target.port(), self.timeout)
                    .await
                    .map(|(stream, _)| Box::new(EgressStream(stream)) as DialStream)
                    .map_err(|error| {
                        DialError::with_source(
                            DialErrorKind::Connection,
                            "fixture route connection failed",
                            error,
                        )
                    })
            })
        }
    }
}

#[derive(Clone)]
struct PolicyDialer {
    target_policy: TargetPolicy,
}

impl Dialer for PolicyDialer {
    fn dial(&self, target: DialTarget) -> DialFuture<'_> {
        Box::pin(async move {
            let addresses = resolve_addresses(self.target_policy, target.host(), target.port())
                .await
                .map_err(|error| {
                    let kind = if error.kind == DiagnosticErrorKind::Policy {
                        DialErrorKind::Rejected
                    } else {
                        DialErrorKind::Other
                    };
                    DialError::new(kind, dial_error_message(&error))
                })?;
            let mut last = None;
            for address in addresses {
                match TcpStream::connect(address).await {
                    Ok(stream) => return Ok(Box::new(stream) as DialStream),
                    Err(error) => last = Some(error),
                }
            }
            let error = last.expect("resolve_addresses returns at least one address");
            Err(DialError::with_source(
                DialErrorKind::Connection,
                safe_io_message(&error),
                error,
            ))
        })
    }
}

#[derive(Clone)]
struct RouteDialer {
    connector: Arc<eggress_embed::outbound::OutboundConnector>,
    timeout: Duration,
}

impl Dialer for RouteDialer {
    fn dial(&self, target: DialTarget) -> DialFuture<'_> {
        Box::pin(async move {
            self.connector
                .connect_tcp_timeout_detailed(target.host(), target.port(), self.timeout)
                .await
                .map(|(stream, _)| Box::new(EgressStream(stream)) as DialStream)
                .map_err(|error| {
                    let kind = match error.kind() {
                        eggress_embed::outbound::OutboundConnectErrorKind::Timeout => {
                            DialErrorKind::Timeout
                        }
                        eggress_embed::outbound::OutboundConnectErrorKind::Authentication => {
                            DialErrorKind::Authentication
                        }
                        eggress_embed::outbound::OutboundConnectErrorKind::Policy => {
                            DialErrorKind::Rejected
                        }
                        eggress_embed::outbound::OutboundConnectErrorKind::Other => {
                            DialErrorKind::Other
                        }
                        _ => DialErrorKind::Connection,
                    };
                    DialError::with_source(kind, "route connection failed", error)
                })
        })
    }
}

struct EgressStream(eggress_core::BoxStream);

impl AsyncRead for EgressStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for EgressStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        data: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, data)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

fn egress_connector(expression: &str) -> Result<eggress_embed::outbound::OutboundConnector, ()> {
    // Eggress builds a default rustls client config without an explicit
    // provider, so the host process must supply one before any routed
    // execution. Install ring (the same provider the direct path uses)
    // idempotently: every routed probe funnels through here, and a previous
    // installation (for example by an embedding test setup) is unaffected.
    let _ = rustls::crypto::ring::default_provider().install_default();
    eggress_embed::outbound::OutboundConnector::from_pproxy_uri(expression).map_err(|_| ())
}

fn route_connect_timeout(deadline_at: tokio::time::Instant) -> Duration {
    let remaining = deadline_at.saturating_duration_since(tokio::time::Instant::now());
    // Let Eggress return its typed deadline slightly before the outer engine
    // deadline so the timeout retains route-stage provenance.
    let translation_reserve = Duration::from_millis(10).min(remaining / 100);
    remaining.saturating_sub(translation_reserve)
}

async fn resolve_addresses(
    target_policy: TargetPolicy,
    host: &str,
    port: u16,
) -> Result<Vec<std::net::SocketAddr>, DiagnosticError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        if target_policy == TargetPolicy::Strict && is_private(ip) {
            return Err(policy_error());
        }
        return Ok(vec![std::net::SocketAddr::new(ip, port)]);
    }
    let results = lookup_host((host, port))
        .await
        .map_err(|error| DiagnosticError {
            kind: DiagnosticErrorKind::Dns,
            stage: DiagnosticStage::Resolution,
            message: safe_io_message(&error),
            attempt: None,
            route_hop_index: None,
            route_protocol: None,
        })?;
    let mut addresses = Vec::new();
    for address in results.take(MAX_DNS_ANSWERS) {
        if target_policy == TargetPolicy::Strict && is_private(address.ip()) {
            return Err(policy_error());
        }
        addresses.push(address);
    }
    if addresses.is_empty() {
        return Err(DiagnosticError {
            kind: DiagnosticErrorKind::Dns,
            stage: DiagnosticStage::Resolution,
            message: "no addresses returned".into(),
            attempt: None,
            route_hop_index: None,
            route_protocol: None,
        });
    }
    Ok(addresses)
}

fn dial_error_message(error: &DiagnosticError) -> String {
    match error.kind {
        DiagnosticErrorKind::Policy => "target rejected by policy".into(),
        DiagnosticErrorKind::Dns => "DNS resolution failed".into(),
        _ => "direct connection failed".into(),
    }
}

fn deadline_results(plan: &ProbePlan) -> Vec<ProbeResult> {
    plan.probes
        .iter()
        .map(|spec| ProbeResult {
            kind: match spec {
                ProbeSpec::Dns => ProbeKind::Dns,
                ProbeSpec::Tcp { .. } => ProbeKind::Tcp,
                ProbeSpec::Tls { .. } => ProbeKind::Tls,
                ProbeSpec::Http { .. } => ProbeKind::Http,
                ProbeSpec::Route => ProbeKind::Route,
                ProbeSpec::IcmpEcho { .. } => ProbeKind::IcmpEcho,
                ProbeSpec::Udp { .. } => ProbeKind::Udp,
                ProbeSpec::Trace { .. } => ProbeKind::Trace,
                ProbeSpec::PathMtu { .. } => ProbeKind::PathMtu,
            },
            status: ProbeStatus::Failed,
            timing: None,
            error: Some(simple_error(
                DiagnosticErrorKind::Timeout,
                DiagnosticStage::Deadline,
                "execution deadline exceeded",
            )),
            evidence: None,
            unavailable: vec![],
            findings: vec![],
        })
        .collect()
}

fn normalize_http_failure(failure: &eggfetch_core::RequestFailure) -> DiagnosticError {
    if failure.is_timeout() {
        return simple_error(
            DiagnosticErrorKind::Timeout,
            DiagnosticStage::Deadline,
            "HTTP deadline exceeded",
        );
    }
    if let Some(kind) = failure.network_failure_kind() {
        return match kind {
            eggfetch_core::NetworkFailureKind::Dns => simple_error(
                DiagnosticErrorKind::Dns,
                DiagnosticStage::Resolution,
                "DNS resolution failed",
            ),
            eggfetch_core::NetworkFailureKind::ConnectionRefused => simple_error(
                DiagnosticErrorKind::ConnectionRefused,
                DiagnosticStage::DirectConnect,
                "connection refused",
            ),
            _ => simple_error(
                DiagnosticErrorKind::Io,
                DiagnosticStage::DirectConnect,
                "direct connection failed",
            ),
        };
    }
    if let Some(error) = failure.error().custom_transport_error() {
        if let Some(route_error) = StdError::source(error).and_then(|source| {
            source.downcast_ref::<eggress_embed::outbound::OutboundConnectError>()
        }) {
            return diagnostic_from_route_error(route_error);
        }
        return match error.kind() {
            DialErrorKind::Rejected => simple_error(
                DiagnosticErrorKind::Policy,
                DiagnosticStage::Resolution,
                "target rejected by policy",
            ),
            DialErrorKind::Timeout => simple_error(
                DiagnosticErrorKind::Timeout,
                DiagnosticStage::DirectConnect,
                "direct connection timed out",
            ),
            DialErrorKind::Authentication => simple_error(
                DiagnosticErrorKind::Authentication,
                DiagnosticStage::DirectConnect,
                "direct connection authentication failed",
            ),
            DialErrorKind::Connection => simple_error(
                DiagnosticErrorKind::Io,
                DiagnosticStage::DirectConnect,
                "direct connection failed",
            ),
            DialErrorKind::Other => simple_error(
                DiagnosticErrorKind::Other,
                DiagnosticStage::DirectConnect,
                "direct connection failed",
            ),
        };
    }
    match failure.error().kind() {
        "tls" | "certificate_verification" | "hostname_verification" => simple_error(
            DiagnosticErrorKind::Tls,
            DiagnosticStage::TlsHandshake,
            "TLS handshake failed",
        ),
        "protocol" | "http2_protocol" | "h3_protocol" => simple_error(
            DiagnosticErrorKind::Protocol,
            DiagnosticStage::ResponseHeaders,
            "HTTP protocol exchange failed",
        ),
        "invalid_url" | "invalid_method" | "request_build" => simple_error(
            DiagnosticErrorKind::Protocol,
            DiagnosticStage::Request,
            "invalid HTTP request",
        ),
        "connect" | "io" => simple_error(
            DiagnosticErrorKind::Io,
            DiagnosticStage::DirectConnect,
            "direct connection failed",
        ),
        "unsupported" => simple_error(
            DiagnosticErrorKind::Unsupported,
            DiagnosticStage::Request,
            "HTTP operation unsupported",
        ),
        _ => simple_error(
            DiagnosticErrorKind::Other,
            DiagnosticStage::ResponseHeaders,
            "HTTP request failed",
        ),
    }
}

fn diagnostic_from_route_error(
    error: &eggress_embed::outbound::OutboundConnectError,
) -> DiagnosticError {
    use eggress_embed::outbound::{
        OutboundConnectErrorKind as Kind, OutboundConnectStage as Stage,
    };

    let kind = match error.kind() {
        Kind::Timeout => DiagnosticErrorKind::Timeout,
        Kind::Dns => DiagnosticErrorKind::Dns,
        Kind::ConnectionRefused => DiagnosticErrorKind::ConnectionRefused,
        Kind::NetworkUnreachable => DiagnosticErrorKind::NetworkUnreachable,
        Kind::HostUnreachable => DiagnosticErrorKind::HostUnreachable,
        Kind::Authentication => DiagnosticErrorKind::Authentication,
        Kind::Tls => DiagnosticErrorKind::Tls,
        Kind::Protocol => DiagnosticErrorKind::Protocol,
        Kind::Policy => DiagnosticErrorKind::Policy,
        _ => DiagnosticErrorKind::Other,
    };
    let stage = match error.stage() {
        Stage::DirectConnect => DiagnosticStage::DirectConnect,
        Stage::HopConnect => DiagnosticStage::HopConnect,
        Stage::HopHandshake => DiagnosticStage::HopHandshake,
        Stage::Deadline => DiagnosticStage::Deadline,
        _ => DiagnosticStage::Other,
    };
    let message = match kind {
        DiagnosticErrorKind::Timeout => "routed connection timed out",
        DiagnosticErrorKind::Dns => "routed name resolution failed",
        DiagnosticErrorKind::ConnectionRefused => "routed connection was refused",
        DiagnosticErrorKind::NetworkUnreachable => "routed network is unreachable",
        DiagnosticErrorKind::HostUnreachable => "routed host is unreachable",
        DiagnosticErrorKind::Authentication => "routed authentication failed",
        DiagnosticErrorKind::Tls => "routed TLS negotiation failed",
        DiagnosticErrorKind::Protocol => "routed protocol exchange failed",
        DiagnosticErrorKind::Policy => "routed operation rejected by policy",
        _ => "routed connection failed",
    };
    DiagnosticError {
        kind,
        stage,
        message: message.into(),
        attempt: None,
        route_hop_index: error.hop_index(),
        route_protocol: error.protocol().map(str::to_owned),
    }
}
fn tool() -> ToolProvenance {
    ToolProvenance {
        name: "eggprobe".into(),
        version: ToolVersion::new(env!("CARGO_PKG_VERSION")).expect("package version is valid"),
    }
}
fn execution_id() -> String {
    format!("exec-{}", NEXT_EXECUTION_ID.fetch_add(1, Ordering::Relaxed))
}
fn elapsed_since(started: Instant) -> DurationMicros {
    DurationMicros::from_micros(
        u64::try_from(started.elapsed().as_micros().min(u128::from(u64::MAX))).unwrap_or(u64::MAX),
    )
}
fn trace_termination(
    completed: bool,
    termination: eggprobe_native::TraceTerminationSummary,
) -> crate::domain::probe::TraceTermination {
    use crate::domain::probe::TraceTermination as Domain;
    use eggprobe_native::TraceTerminationSummary as Native;
    if completed {
        match termination {
            Native::DestinationReached => Domain::DestinationReached,
            Native::Unreachable => Domain::Unreachable,
            Native::MaxHops => Domain::MaxHops,
            Native::Deadline => Domain::Deadline,
        }
    } else {
        Domain::Deadline
    }
}
fn trace_outcome(
    outcome: eggprobe_native::TraceProbeOutcome,
) -> crate::domain::probe::NativeAttemptOutcome {
    use crate::domain::probe::NativeAttemptOutcome as Domain;
    use eggprobe_native::TraceProbeOutcome as Native;
    match outcome {
        Native::DestinationReached => Domain::DestinationReached,
        Native::TimeExceeded => Domain::TimeExceeded,
        Native::Reply => Domain::Reply,
        Native::DestinationUnreachable => Domain::DestinationUnreachable,
        Native::TimedOut => Domain::TimedOut,
    }
}
fn simple_error(
    kind: DiagnosticErrorKind,
    stage: DiagnosticStage,
    message: &str,
) -> DiagnosticError {
    DiagnosticError {
        kind,
        stage,
        message: message.into(),
        attempt: None,
        route_hop_index: None,
        route_protocol: None,
    }
}
fn safe_io_message(error: &std::io::Error) -> String {
    error.kind().to_string()
}
fn normalize_io(
    error: &std::io::Error,
    stage: DiagnosticStage,
    attempt: Option<u32>,
) -> DiagnosticError {
    let kind = match error.kind() {
        std::io::ErrorKind::ConnectionRefused => DiagnosticErrorKind::ConnectionRefused,
        std::io::ErrorKind::TimedOut => DiagnosticErrorKind::Timeout,
        std::io::ErrorKind::NetworkUnreachable => DiagnosticErrorKind::NetworkUnreachable,
        std::io::ErrorKind::HostUnreachable => DiagnosticErrorKind::HostUnreachable,
        _ => DiagnosticErrorKind::Io,
    };
    DiagnosticError {
        kind,
        stage,
        message: safe_io_message(error),
        attempt,
        route_hop_index: None,
        route_protocol: None,
    }
}
fn policy_error() -> DiagnosticError {
    simple_error(
        DiagnosticErrorKind::Policy,
        DiagnosticStage::Resolution,
        "target rejected by policy",
    )
}
fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

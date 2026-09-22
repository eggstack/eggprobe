//! Core-owned asynchronous probe execution.

use std::{
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
    time::timeout,
};
use tokio_rustls::TlsConnector;

use crate::{
    domain::{
        error::{DiagnosticError, DiagnosticErrorKind, DiagnosticStage},
        plan::{ProbePlan, ProbeSpec},
        probe::{
            DnsEvidence, HttpEvidence, ProbeEvidence, ProbeKind, ProbeResult, ProbeStatus,
            TcpEvidence, TlsEvidence,
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
        if let Ok(report) = timeout(deadline, self.execute_inner(&plan, execution_id.clone())).await
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

    async fn execute_inner(&self, plan: &ProbePlan, execution_id: String) -> ProbeReport {
        let repetitions = usize::try_from(plan.execution.repetitions).unwrap_or(usize::MAX);
        let mut results = Vec::with_capacity(plan.probes.len().saturating_mul(repetitions));
        for _ in 0..plan.execution.repetitions {
            for spec in &plan.probes {
                results.push(self.execute_probe(plan, spec).await);
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

    async fn execute_probe(&self, plan: &ProbePlan, spec: &ProbeSpec) -> ProbeResult {
        let started = Instant::now();
        let kind = match spec {
            ProbeSpec::Dns => ProbeKind::Dns,
            ProbeSpec::Tcp { .. } => ProbeKind::Tcp,
            ProbeSpec::Tls { .. } => ProbeKind::Tls,
            ProbeSpec::Http { .. } => ProbeKind::Http,
        };
        let is_http = kind == ProbeKind::Http;
        let outcome = match spec {
            ProbeSpec::Dns => self.dns(plan).await,
            ProbeSpec::Tcp { port } => self.tcp(plan, *port).await,
            ProbeSpec::Tls { port, server_name } => {
                self.tls(plan, *port, server_name.as_deref()).await
            }
            ProbeSpec::Http { url, method } => self.http(plan, url, method).await,
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
                status: ProbeStatus::Failed,
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

    async fn dns(&self, plan: &ProbePlan) -> Result<ProbeEvidence, DiagnosticError> {
        let addresses = self
            .addresses(&plan.target.host, plan.target.port.unwrap_or(0))
            .await?;
        Ok(ProbeEvidence::Dns(DnsEvidence {
            addresses: addresses.into_iter().map(|a| a.ip().to_string()).collect(),
        }))
    }

    async fn tcp(&self, plan: &ProbePlan, port: u16) -> Result<ProbeEvidence, DiagnosticError> {
        if let RouteSpec::Eggress(route) = &plan.route {
            let connector = egress_connector(&route.expression).map_err(|()| {
                simple_error(
                    DiagnosticErrorKind::Protocol,
                    DiagnosticStage::HopHandshake,
                    "invalid or unsupported Eggress route",
                )
            })?;
            let (stream, info) = connector
                .connect_tcp(&plan.target.host, port)
                .await
                .map_err(|_| {
                    simple_error(
                        DiagnosticErrorKind::Other,
                        DiagnosticStage::HopConnect,
                        "route connection failed",
                    )
                })?;
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
    ) -> Result<ProbeEvidence, DiagnosticError> {
        let name = ServerName::try_from(server_name.unwrap_or(&plan.target.host).to_owned())
            .map_err(|_| DiagnosticError {
                kind: DiagnosticErrorKind::Tls,
                stage: DiagnosticStage::TlsHandshake,
                message: "invalid TLS server name".into(),
                attempt: None,
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
                    .connect_tcp(&plan.target.host, port)
                    .await
                    .map_err(|_| {
                        simple_error(
                            DiagnosticErrorKind::Other,
                            DiagnosticStage::HopConnect,
                            "route connection failed",
                        )
                    })?;
                tls_handshake(name, EgressStream(stream)).await
            }
        }
    }

    async fn http(
        &self,
        plan: &ProbePlan,
        url: &str,
        method: &str,
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
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
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
}

impl Dialer for RouteDialer {
    fn dial(&self, target: DialTarget) -> DialFuture<'_> {
        Box::pin(async move {
            self.connector
                .connect_tcp(target.host(), target.port())
                .await
                .map(|(stream, _)| Box::new(EgressStream(stream)) as DialStream)
                .map_err(|_| DialError::new(DialErrorKind::Connection, "route connection failed"))
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
    eggress_embed::outbound::OutboundConnector::from_pproxy_uri(expression).map_err(|_| ())
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
fn tool() -> ToolProvenance {
    ToolProvenance {
        name: "eggprobe".into(),
        version: ToolVersion::new(env!("CARGO_PKG_VERSION")).expect("package version is valid"),
    }
}
fn execution_id() -> String {
    format!("exec-{}", NEXT_EXECUTION_ID.fetch_add(1, Ordering::Relaxed))
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

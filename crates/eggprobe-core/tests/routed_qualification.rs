use std::{net::SocketAddr, sync::Once, time::Duration};

use eggprobe_core::{
    DiagnosticErrorKind, DiagnosticStage, DurationMicros, EggressRoute, ExecutionPolicy,
    ProbeEngine, ProbeEvidence, ProbePlan, ProbeSpec, ProbeStatus, RouteSpec, SchemaVersion,
    TargetSpec,
};
use tokio::{io::AsyncReadExt, net::TcpListener};

static CRYPTO_PROVIDER: Once = Once::new();

fn install_crypto_provider() {
    CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn plan(host: &str, port: Option<u16>, route: &str, probes: Vec<ProbeSpec>) -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target: TargetSpec::new(host, port).unwrap(),
        route: RouteSpec::Eggress(EggressRoute {
            expression: route.to_owned(),
        }),
        probes,
        execution: ExecutionPolicy {
            deadline: DurationMicros::from_micros(2_000_000),
            ..ExecutionPolicy::default()
        },
        assertions: vec![],
    }
}

fn direct_plan(port: u16, deadline_micros: u64) -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target: TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
        route: RouteSpec::Direct,
        probes: vec![ProbeSpec::Tls {
            port,
            server_name: Some("localhost".into()),
        }],
        execution: ExecutionPolicy {
            deadline: DurationMicros::from_micros(deadline_micros),
            ..ExecutionPolicy::default()
        },
        assertions: vec![],
    }
}

fn start_proxy(
    protocols: &str,
    auth: Option<(&str, &str)>,
) -> (eggress_embed::EggressHandle, SocketAddr) {
    install_crypto_provider();
    let auth = auth.map_or_else(String::new, |(username, password)| {
        format!(
            "\n[listeners.auth]\ntype = \"password\"\nusername = \"{username}\"\npassword = \"{password}\"\n"
        )
    });
    let config = eggress_embed::EggressConfig::from_toml_str(&format!(
        "version = 1\n\n[[listeners]]\nname = \"proxy\"\nbind = \"127.0.0.1:0\"\nprotocols = {protocols}\n{auth}"
    ))
    .unwrap();
    let handle = eggress_embed::EggressService::new(config)
        .start_blocking()
        .unwrap();
    let address = handle.bound_addresses().listeners[0].addr;
    (handle, address)
}

async fn closed_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn serve_one_http_response() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let Ok((mut stream, _)) = listener.accept().await else {
            return;
        };
        let mut request = Vec::new();
        let mut chunk = [0u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let Ok(count) = stream.read(&mut chunk).await else {
                return;
            };
            if count == 0 {
                return;
            }
            request.extend_from_slice(&chunk[..count]);
            if request.len() > 16 * 1024 {
                return;
            }
        }
        let _ = tokio::io::AsyncWriteExt::write_all(
            &mut stream,
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        )
        .await;
    });
    port
}

async fn hang_on_accept() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                drop(stream);
            });
        }
    });
    port
}

#[tokio::test]
async fn routed_tcp_uses_egress_connector_without_fallback() {
    let (proxy, proxy_addr) = start_proxy("[\"socks5\"]", None);
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    let accepted = tokio::spawn(async move { target.accept().await.unwrap() });

    let report = ProbeEngine::default()
        .execute(plan(
            "127.0.0.1",
            Some(target_port),
            &format!("socks5://{proxy_addr}"),
            vec![ProbeSpec::Tcp { port: target_port }],
        ))
        .await;

    assert_eq!(report.probes[0].status, ProbeStatus::Ok, "{report:?}");
    assert!(matches!(
        report.probes[0].evidence,
        Some(ProbeEvidence::Tcp(_))
    ));
    drop(accepted.await.unwrap());
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn routed_hop_connect_error_keeps_typed_stage_and_never_dials_target_directly() {
    install_crypto_provider();
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    // A dropped port returns to the pool immediately, so a parallel fixture
    // on a loaded runner can claim it between drop and dial. A stolen port
    // accepts the SYN and then stalls the proxy handshake (surfacing as a
    // timeout or handshake error) instead of refusing. Retry with a fresh
    // dropped port so the outcome is deterministic: only a genuine refusal
    // is accepted, and repeated theft panics with the last error attached.
    let mut attempts = 0;
    loop {
        attempts += 1;
        let hop_port = closed_port().await;
        let mut diagnostic_plan = plan(
            "127.0.0.1",
            Some(target_port),
            &format!("http://127.0.0.1:{hop_port}"),
            vec![ProbeSpec::Tcp { port: target_port }],
        );
        diagnostic_plan.execution.deadline = DurationMicros::from_micros(500_000);
        let report = ProbeEngine::default().execute(diagnostic_plan).await;
        let error = report.probes[0].error.as_ref().unwrap();
        if error.kind == DiagnosticErrorKind::ConnectionRefused {
            assert_eq!(error.stage, DiagnosticStage::HopConnect, "{error:?}");
            assert_eq!(error.route_hop_index, Some(0));
            assert_eq!(error.route_protocol, None);
            break;
        }
        assert!(
            attempts < 10,
            "hop port claimed by a parallel fixture {attempts} times in a row: {error:?}"
        );
    }
    assert!(
        tokio::time::timeout(Duration::from_millis(80), target.accept())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn routed_auth_failure_keeps_protocol_and_redacts_credentials() {
    let (proxy, proxy_addr) =
        start_proxy("[\"socks5\"]", Some(("expected-user", "expected-password")));
    let secret = "wrong-password-must-not-escape";
    let route = format!("socks5://user:{secret}@{proxy_addr}");
    let report = ProbeEngine::default()
        .execute(plan(
            "example.invalid",
            Some(80),
            &route,
            vec![ProbeSpec::Tcp { port: 80 }],
        ))
        .await;

    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Authentication);
    assert_eq!(error.stage, DiagnosticStage::HopHandshake);
    assert_eq!(error.route_hop_index, Some(0));
    assert_eq!(error.route_protocol.as_deref(), Some("socks5"));
    let json = report.to_json().unwrap();
    assert!(!json.contains(secret));
    assert!(!json.contains("expected-password"));
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn two_hop_socks_route_reaches_local_target() {
    let (first, first_addr) = start_proxy("[\"socks5\"]", None);
    let (second, second_addr) = start_proxy("[\"socks5\"]", None);
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    let accepted = tokio::spawn(async move { target.accept().await.unwrap() });

    let report = ProbeEngine::default()
        .execute(plan(
            "127.0.0.1",
            Some(target_port),
            &format!("socks5://{first_addr}__socks5://{second_addr}"),
            vec![ProbeSpec::Tcp { port: target_port }],
        ))
        .await;

    assert_eq!(report.probes[0].status, ProbeStatus::Ok, "{report:?}");
    drop(accepted.await.unwrap());
    first.shutdown_blocking().unwrap();
    second.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn two_hop_route_failure_keeps_later_hop_provenance() {
    let (first, first_addr) = start_proxy("[\"http\"]", None);
    let (second, second_addr) =
        start_proxy("[\"http\"]", Some(("expected-user", "expected-password")));
    let route = format!("http://{first_addr}__http://wrong-user:wrong-password@{second_addr}");
    let report = ProbeEngine::default()
        .execute(plan(
            "example.invalid",
            Some(80),
            &route,
            vec![ProbeSpec::Tcp { port: 80 }],
        ))
        .await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Authentication, "{error:?}");
    assert_eq!(error.route_hop_index, Some(1), "{error:?}");
    assert_eq!(error.stage, DiagnosticStage::HopHandshake, "{error:?}");
    assert_eq!(error.route_protocol.as_deref(), Some("http"));
    first.shutdown_blocking().unwrap();
    second.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn http_connect_route_reaches_http1_origin() {
    let (proxy, proxy_addr) = start_proxy("[\"http\"]", None);
    let origin_port = serve_one_http_response().await;
    let report = ProbeEngine::default()
        .execute(plan(
            "127.0.0.1",
            Some(origin_port),
            &format!("http://{proxy_addr}"),
            vec![ProbeSpec::Http {
                url: format!("http://127.0.0.1:{origin_port}/health"),
                method: "GET".into(),
            }],
        ))
        .await;
    assert_eq!(report.probes[0].status, ProbeStatus::Ok, "{report:?}");
    let Some(ProbeEvidence::Http(evidence)) = &report.probes[0].evidence else {
        panic!("expected HTTP evidence: {report:?}");
    };
    assert_eq!(evidence.status, Some(200));
    assert_eq!(evidence.protocol.as_deref(), Some("HTTP/1.1"));
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn http_connect_auth_rejection_keeps_typed_provenance() {
    let (proxy, proxy_addr) =
        start_proxy("[\"http\"]", Some(("expected-user", "expected-password")));
    let route = format!("http://wrong-user:wrong-password@{proxy_addr}");
    let report = ProbeEngine::default()
        .execute(plan(
            "127.0.0.1",
            Some(443),
            &route,
            vec![ProbeSpec::Tcp { port: 443 }],
        ))
        .await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Authentication);
    assert_eq!(error.stage, DiagnosticStage::HopHandshake);
    assert_eq!(error.route_hop_index, Some(0));
    assert_eq!(error.route_protocol.as_deref(), Some("http"));
    assert!(!report.to_json().unwrap().contains("wrong-password"));
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn eggfetch_over_eggress_retains_typed_connect_failure() {
    let (proxy, proxy_addr) =
        start_proxy("[\"http\"]", Some(("expected-user", "expected-password")));
    let route = format!("http://wrong-user:wrong-password@{proxy_addr}");
    let report = ProbeEngine::default()
        .execute(plan(
            "example.invalid",
            Some(80),
            &route,
            vec![ProbeSpec::Http {
                url: "http://example.invalid/".into(),
                method: "GET".into(),
            }],
        ))
        .await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Authentication, "{error:?}");
    assert_eq!(error.stage, DiagnosticStage::HopHandshake, "{error:?}");
    assert_eq!(error.route_hop_index, Some(0));
    assert_eq!(error.route_protocol.as_deref(), Some("http"));
    assert!(!report.to_json().unwrap().contains("wrong-password"));
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn routed_connect_deadline_keeps_deadline_stage() {
    install_crypto_provider();
    let hanging_proxy = hang_on_accept().await;
    let mut diagnostic_plan = plan(
        "example.invalid",
        Some(80),
        &format!("http://127.0.0.1:{hanging_proxy}"),
        vec![ProbeSpec::Tcp { port: 80 }],
    );
    diagnostic_plan.execution.deadline = DurationMicros::from_micros(120_000);
    let report = ProbeEngine::default().execute(diagnostic_plan).await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Timeout, "{error:?}");
    assert_eq!(error.stage, DiagnosticStage::Deadline, "{error:?}");
    assert_eq!(error.route_hop_index, None);

    let hanging_http_proxy = hang_on_accept().await;
    let mut http_plan = plan(
        "example.invalid",
        Some(80),
        &format!("http://127.0.0.1:{hanging_http_proxy}"),
        vec![ProbeSpec::Http {
            url: "http://example.invalid/".into(),
            method: "GET".into(),
        }],
    );
    http_plan.execution.deadline = DurationMicros::from_micros(120_000);
    let report = ProbeEngine::default().execute(http_plan).await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Timeout, "{error:?}");
    assert_eq!(error.stage, DiagnosticStage::Deadline, "{error:?}");
}

#[tokio::test]
async fn standalone_tls_handshake_deadline_is_structured() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;
    });
    let report = ProbeEngine::default()
        .execute(direct_plan(port, 30_000))
        .await;
    let error = report.probes[0].error.as_ref().unwrap();
    assert_eq!(error.kind, DiagnosticErrorKind::Timeout);
    assert_eq!(error.stage, DiagnosticStage::Deadline);
}

#[tokio::test]
async fn caller_cancellation_drops_routed_tls_transport() {
    let (proxy, proxy_addr) = start_proxy("[\"socks5\"]", None);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let observed_close = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = [0; 1024];
        loop {
            if stream.read(&mut bytes).await.unwrap() == 0 {
                break;
            }
        }
    });
    let diagnostic_plan = plan(
        "127.0.0.1",
        Some(port),
        &format!("socks5://{proxy_addr}"),
        vec![ProbeSpec::Tls {
            port,
            server_name: Some("localhost".into()),
        }],
    );
    let execution =
        tokio::spawn(async move { ProbeEngine::default().execute(diagnostic_plan).await });
    tokio::time::sleep(Duration::from_millis(30)).await;
    execution.abort();
    let _ = execution.await;
    tokio::time::timeout(Duration::from_millis(200), observed_close)
        .await
        .expect("cancelled routed transport must close")
        .unwrap();
    proxy.shutdown_blocking().unwrap();
}

#[tokio::test]
async fn caller_cancellation_drops_standalone_tls_transport() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let observed_close = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = [0; 1024];
        loop {
            if stream.read(&mut bytes).await.unwrap() == 0 {
                break;
            }
        }
    });
    let cancelled = tokio::time::timeout(
        Duration::from_millis(25),
        ProbeEngine::default().execute(direct_plan(port, 1_000_000)),
    )
    .await;
    assert!(
        cancelled.is_err(),
        "outer test deadline should cancel execution"
    );
    tokio::time::timeout(Duration::from_millis(100), observed_close)
        .await
        .expect("cancelled connection must close")
        .unwrap();
}

#[tokio::test]
async fn routed_dns_evidence_labels_local_client_scope() {
    let (proxy, proxy_addr) = start_proxy("[\"socks5\"]", None);
    let report = ProbeEngine::default()
        .execute(plan(
            "localhost",
            None,
            &format!("socks5://{proxy_addr}"),
            vec![ProbeSpec::Dns],
        ))
        .await;
    let Some(ProbeEvidence::Dns(evidence)) = &report.probes[0].evidence else {
        panic!("expected DNS evidence: {report:?}");
    };
    assert_eq!(
        evidence.resolution_scope,
        eggprobe_core::DnsResolutionScope::Client
    );
    proxy.shutdown_blocking().unwrap();
}

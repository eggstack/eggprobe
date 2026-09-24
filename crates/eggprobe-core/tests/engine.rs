use eggprobe_core::{
    evaluate_assertions, AssertionKind, AssertionSpec, ExecutionPolicy, PlanValidationError,
    ProbeEngine, ProbePlan, ProbeSpec, ReportStatus, RouteSpec, SchemaVersion, TargetPolicy,
    TargetSpec,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

fn plan(target: TargetSpec, probes: Vec<ProbeSpec>) -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target,
        route: RouteSpec::Direct,
        probes,
        execution: ExecutionPolicy::default(),
        assertions: vec![],
    }
}

#[tokio::test]
async fn tcp_probe_reports_local_fixture_success() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream.shutdown().await.unwrap();
    });
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
            vec![ProbeSpec::Tcp { port }],
        ))
        .await;
    task.await.unwrap();
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Tcp(evidence)) = &report.probes[0].evidence else {
        panic!("TCP evidence missing");
    };
    assert!(evidence.local.is_some());
}

#[tokio::test]
async fn route_probe_reports_kernel_source_and_correlated_interface() {
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", None).unwrap(),
            vec![ProbeSpec::Route],
        ))
        .await;
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Route(evidence)) = &report.probes[0].evidence else {
        panic!("route evidence missing");
    };
    assert_eq!(evidence.target, "127.0.0.1");
    assert_eq!(evidence.source.as_deref(), Some("127.0.0.1"));
    assert!(evidence
        .interface
        .as_ref()
        .is_some_and(|item| item.index.is_some()));
    assert!(evidence.routes.len() <= 32);
}

#[tokio::test]
async fn route_probe_supports_ipv6_loopback_when_available() {
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("::1", None).unwrap(),
            vec![ProbeSpec::Route],
        ))
        .await;
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Route(evidence)) = &report.probes[0].evidence else {
        panic!("IPv6 route evidence missing");
    };
    assert_eq!(evidence.target, "::1");
    assert_eq!(evidence.source.as_deref(), Some("::1"));
}

#[tokio::test]
async fn strict_policy_rejects_native_route_probe_before_platform_observation() {
    let report = ProbeEngine {
        target_policy: TargetPolicy::Strict,
    }
    .execute(plan(
        TargetSpec::new("127.0.0.1", None).unwrap(),
        vec![ProbeSpec::Route],
    ))
    .await;
    assert_eq!(report.status, ReportStatus::Failed);
    assert_eq!(
        report.probes[0].error.as_ref().unwrap().kind,
        eggprobe_core::DiagnosticErrorKind::Policy
    );
}

#[tokio::test]
async fn native_contract_operations_are_typed_unsupported_until_backend_lands() {
    let specs = vec![
        ProbeSpec::Route,
        ProbeSpec::IcmpEcho {
            count: 1,
            payload_bytes: 56,
        },
        ProbeSpec::Udp {
            port: 53,
            payload: vec![],
            receive: false,
        },
        ProbeSpec::Trace {
            max_hops: 3,
            attempts_per_hop: 1,
        },
        ProbeSpec::PathMtu {
            min_bytes: 1280,
            max_bytes: 1500,
            attempts: 1,
        },
    ];
    let report = ProbeEngine::default()
        .execute(plan(TargetSpec::new("127.0.0.1", None).unwrap(), specs))
        .await;
    let kinds = [
        eggprobe_core::ProbeKind::Route,
        eggprobe_core::ProbeKind::IcmpEcho,
        eggprobe_core::ProbeKind::Udp,
        eggprobe_core::ProbeKind::Trace,
        eggprobe_core::ProbeKind::PathMtu,
    ];
    assert_eq!(
        report
            .probes
            .iter()
            .map(|probe| &probe.kind)
            .collect::<Vec<_>>(),
        kinds.iter().collect::<Vec<_>>()
    );
    assert_eq!(report.probes[0].status, eggprobe_core::ProbeStatus::Ok);
    assert!(report.probes[1..]
        .iter()
        .all(|probe| probe.status == eggprobe_core::ProbeStatus::Unsupported));
}

#[tokio::test]
async fn native_probe_never_falls_back_from_eggress_to_direct() {
    let mut routed = plan(
        TargetSpec::new("127.0.0.1", None).unwrap(),
        vec![ProbeSpec::Route],
    );
    routed.route = RouteSpec::Eggress(eggprobe_core::EggressRoute {
        expression: "socks5://127.0.0.1:9".into(),
    });
    let report = ProbeEngine::default().execute(routed).await;
    assert_eq!(report.status, ReportStatus::Unsupported);
    assert_eq!(
        report.probes[0].status,
        eggprobe_core::ProbeStatus::Unsupported
    );
}

#[tokio::test]
async fn http_status_is_observed_even_when_assertion_fails() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        // Consume the request headers with a bound before responding.
        // Closing a Windows socket while request bytes remain unread sends
        // RST, which can discard the queued response before Eggfetch reads
        // it; Unix delivers the queued bytes with FIN instead, which is why
        // the previous write-and-drop fixture passed everywhere but Windows.
        let mut request = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let count =
                tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut chunk))
                    .await
                    .expect("fixture request read must not hang")
                    .unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() > 16 * 1024 {
                break;
            }
        }
        stream
            .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        stream.flush().await.unwrap();
        stream.shutdown().await.unwrap();
    });
    let mut diagnostic_plan = plan(
        TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
        vec![ProbeSpec::Http {
            url: format!("http://127.0.0.1:{port}/"),
            method: "GET".into(),
        }],
    );
    diagnostic_plan.assertions.push(AssertionSpec {
        id: "status".into(),
        assertion: AssertionKind::HttpStatusRange { min: 200, max: 299 },
    });
    let report = ProbeEngine::default()
        .execute(diagnostic_plan.clone())
        .await;
    task.await.unwrap();
    assert_eq!(report.status, ReportStatus::Ok);
    assert_eq!(
        report.probes[0].status,
        eggprobe_core::ProbeStatus::Ok,
        "{report:?}"
    );
    let Some(eggprobe_core::ProbeEvidence::Http(evidence)) = &report.probes[0].evidence else {
        panic!("HTTP evidence missing: {report:?}");
    };
    assert_eq!(evidence.status, Some(503), "{report:?}");
    let findings = evaluate_assertions(&report, &diagnostic_plan.assertions);
    assert_eq!(findings[0].outcome, eggprobe_core::FindingOutcome::Failed);
}

#[tokio::test]
async fn strict_policy_rejects_loopback_before_dialing() {
    let report = ProbeEngine {
        target_policy: TargetPolicy::Strict,
    }
    .execute(plan(
        TargetSpec::new("127.0.0.1", Some(9)).unwrap(),
        vec![ProbeSpec::Tcp { port: 9 }],
    ))
    .await;
    assert_eq!(report.status, ReportStatus::Failed);
    assert_eq!(
        report.probes[0].error.as_ref().unwrap().kind,
        eggprobe_core::DiagnosticErrorKind::Policy
    );
}

#[tokio::test]
async fn strict_policy_is_enforced_by_the_actual_direct_http_dialer() {
    let report = ProbeEngine {
        target_policy: TargetPolicy::Strict,
    }
    .execute(plan(
        TargetSpec::new("127.0.0.1", None).unwrap(),
        vec![ProbeSpec::Http {
            url: "http://127.0.0.1/".into(),
            method: "GET".into(),
        }],
    ))
    .await;
    assert_eq!(report.status, ReportStatus::Failed);
    assert_eq!(
        report.probes[0].error.as_ref().unwrap().kind,
        eggprobe_core::DiagnosticErrorKind::Policy
    );
}

#[tokio::test]
async fn deadline_is_a_structured_probe_failure_with_one_execution_id() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    });
    let mut diagnostic_plan = plan(
        TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
        vec![ProbeSpec::Http {
            url: format!("http://127.0.0.1:{port}/"),
            method: "GET".into(),
        }],
    );
    diagnostic_plan.execution.deadline = eggprobe_core::DurationMicros::from_micros(20_000);
    let report = ProbeEngine::default().execute(diagnostic_plan).await;
    task.await.unwrap();
    assert_eq!(report.status, ReportStatus::Failed);
    assert!(report.execution_id.starts_with("exec-"));
    assert_eq!(report.probes.len(), 1, "{report:?}");
    assert_eq!(report.probes[0].status, eggprobe_core::ProbeStatus::Failed);
    assert_eq!(
        report.probes[0].error.as_ref().unwrap().kind,
        eggprobe_core::DiagnosticErrorKind::Timeout
    );
}

#[test]
fn retries_are_rejected_instead_of_ignored() {
    let mut diagnostic_plan = plan(
        TargetSpec::new("example.com", None).unwrap(),
        vec![ProbeSpec::Dns],
    );
    diagnostic_plan.execution.retries = 1;
    assert!(matches!(
        diagnostic_plan.validate(),
        Err(PlanValidationError::UnsupportedRetries(1))
    ));
}

#[test]
fn invalid_http_assertion_ranges_are_rejected() {
    let mut diagnostic_plan = plan(
        TargetSpec::new("example.com", None).unwrap(),
        vec![ProbeSpec::Dns],
    );
    diagnostic_plan.assertions.push(AssertionSpec {
        id: "status".into(),
        assertion: AssertionKind::HttpStatusRange { min: 500, max: 200 },
    });
    assert!(matches!(
        diagnostic_plan.validate(),
        Err(PlanValidationError::InvalidAssertionRange { min: 500, max: 200 })
    ));
}

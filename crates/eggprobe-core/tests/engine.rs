use eggprobe_core::{
    evaluate_assertions, trace_capability, AssertionKind, AssertionSpec, ExecutionPolicy,
    PlanValidationError, ProbeEngine, ProbePlan, ProbeSpec, ReportStatus, RouteSpec, SchemaVersion,
    TargetPolicy, TargetSpec, TraceCapability,
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
        ProbeSpec::IcmpEcho {
            count: 1,
            payload_bytes: 56,
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
        eggprobe_core::ProbeKind::IcmpEcho,
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
    assert!(report
        .probes
        .iter()
        .all(|probe| probe.status == eggprobe_core::ProbeStatus::Unsupported));
}

#[tokio::test]
async fn udp_probe_reports_loopback_echo_response() {
    use tokio::net::UdpSocket;
    let echo = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = echo.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let mut buffer = [0_u8; 2048];
        let (len, source) = echo.recv_from(&mut buffer).await.unwrap();
        echo.send_to(&buffer[..len], source).await.unwrap();
    });
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
            vec![ProbeSpec::Udp {
                port,
                payload: b"echo-probe".to_vec(),
                receive: true,
            }],
        ))
        .await;
    task.await.unwrap();
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Udp(evidence)) = &report.probes[0].evidence else {
        panic!("UDP evidence missing");
    };
    assert_eq!(evidence.outcome, eggprobe_core::UdpOutcome::Response);
    assert_eq!(evidence.transmitted_bytes, 10);
    assert!(evidence.local.is_some());
    assert!(evidence.response_source.is_some());
    assert_eq!(evidence.response_bytes, Some(10));
    assert_eq!(evidence.response_sample, Some(b"echo-probe".to_vec()));
    // Request payload bytes are input-only and never reproduced in reports.
    let rendered = serde_json::to_string(&report).unwrap();
    assert!(!rendered.contains("echo-probe"));
}

#[tokio::test]
async fn udp_probe_reports_ipv6_loopback_echo_response() {
    use tokio::net::UdpSocket;
    let echo = UdpSocket::bind("[::1]:0").await.unwrap();
    let port = echo.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let mut buffer = [0_u8; 2048];
        let (len, source) = echo.recv_from(&mut buffer).await.unwrap();
        echo.send_to(&buffer[..len], source).await.unwrap();
    });
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("::1", Some(port)).unwrap(),
            vec![ProbeSpec::Udp {
                port,
                payload: vec![1, 2, 3],
                receive: true,
            }],
        ))
        .await;
    task.await.unwrap();
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Udp(evidence)) = &report.probes[0].evidence else {
        panic!("IPv6 UDP evidence missing");
    };
    assert_eq!(evidence.outcome, eggprobe_core::UdpOutcome::Response);
    assert_eq!(evidence.response_bytes, Some(3));
}

#[tokio::test]
async fn udp_probe_without_receive_reports_transmission_only() {
    use tokio::net::UdpSocket;
    let discard = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = discard.local_addr().unwrap().port();
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
            vec![ProbeSpec::Udp {
                port,
                payload: vec![],
                receive: false,
            }],
        ))
        .await;
    drop(discard);
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Udp(evidence)) = &report.probes[0].evidence else {
        panic!("UDP evidence missing");
    };
    assert_eq!(evidence.outcome, eggprobe_core::UdpOutcome::Sent);
    assert_eq!(evidence.transmitted_bytes, 0);
    assert_eq!(evidence.response_sample, None);
}

#[tokio::test]
async fn udp_probe_reports_silence_as_timeout_observation() {
    use tokio::net::UdpSocket;
    let silent = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = silent.local_addr().unwrap().port();
    let mut quiet = plan(
        TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
        vec![ProbeSpec::Udp {
            port,
            payload: b"ping".to_vec(),
            receive: true,
        }],
    );
    quiet.execution.deadline = eggprobe_core::DurationMicros::from_micros(500_000);
    let report = ProbeEngine::default().execute(quiet).await;
    drop(silent);
    // Local transmission succeeded, so silence is a completed Timeout
    // observation rather than a local execution failure.
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Udp(evidence)) = &report.probes[0].evidence else {
        panic!("UDP evidence missing");
    };
    assert_eq!(evidence.outcome, eggprobe_core::UdpOutcome::Timeout);
    assert_eq!(evidence.transmitted_bytes, 4);
}

#[tokio::test]
async fn udp_probe_reports_closed_port_without_brittle_timing() {
    use tokio::net::UdpSocket;
    let claimed = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = claimed.local_addr().unwrap().port();
    drop(claimed);
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
            vec![ProbeSpec::Udp {
                port,
                payload: b"ping".to_vec(),
                receive: true,
            }],
        ))
        .await;
    // ICMP port-unreachable delivery is platform-specific: Linux surfaces it
    // on the connected socket while other hosts may stay silent. Either
    // completed observation preserves the transmitted byte count.
    assert_eq!(report.status, ReportStatus::Ok);
    let Some(eggprobe_core::ProbeEvidence::Udp(evidence)) = &report.probes[0].evidence else {
        panic!("UDP evidence missing");
    };
    assert!(matches!(
        evidence.outcome,
        eggprobe_core::UdpOutcome::Unreachable | eggprobe_core::UdpOutcome::Timeout
    ));
    assert_eq!(evidence.transmitted_bytes, 4);
}

#[tokio::test]
async fn udp_probe_rejects_restricted_destination_classes() {
    for host in [
        "0.0.0.0",
        "255.255.255.255",
        "224.0.0.1",
        "169.254.169.254",
        "::",
        "ff02::1",
    ] {
        let report = ProbeEngine::default()
            .execute(plan(
                TargetSpec::new(host, Some(53)).unwrap(),
                vec![ProbeSpec::Udp {
                    port: 53,
                    payload: vec![],
                    receive: false,
                }],
            ))
            .await;
        assert_eq!(report.status, ReportStatus::Failed, "host {host}");
        assert_eq!(
            report.probes[0].error.as_ref().unwrap().kind,
            eggprobe_core::DiagnosticErrorKind::Policy,
            "host {host}"
        );
    }
}

#[tokio::test]
async fn udp_probe_rejects_private_target_under_strict_policy() {
    let report = ProbeEngine {
        target_policy: TargetPolicy::Strict,
    }
    .execute(plan(
        TargetSpec::new("127.0.0.1", Some(53)).unwrap(),
        vec![ProbeSpec::Udp {
            port: 53,
            payload: vec![],
            receive: false,
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
async fn udp_probe_never_falls_back_from_eggress_to_direct() {
    use tokio::net::UdpSocket;
    let discard = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = discard.local_addr().unwrap().port();
    let mut routed = plan(
        TargetSpec::new("127.0.0.1", Some(port)).unwrap(),
        vec![ProbeSpec::Udp {
            port,
            payload: vec![],
            receive: false,
        }],
    );
    routed.route = RouteSpec::Eggress(eggprobe_core::EggressRoute {
        expression: "socks5://127.0.0.1:9".into(),
    });
    let report = ProbeEngine::default().execute(routed).await;
    drop(discard);
    assert_eq!(report.status, ReportStatus::Unsupported);
    assert_eq!(
        report.probes[0].status,
        eggprobe_core::ProbeStatus::Unsupported
    );
}

#[test]
fn udp_plan_bounds_reject_zero_port_and_oversize_payload() {
    let oversize = ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target: TargetSpec::new("127.0.0.1", Some(53)).unwrap(),
        route: RouteSpec::Direct,
        probes: vec![ProbeSpec::Udp {
            port: 53,
            payload: vec![0_u8; 1201],
            receive: false,
        }],
        execution: ExecutionPolicy::default(),
        assertions: vec![],
    };
    assert_eq!(
        oversize.validate(),
        Err(PlanValidationError::InvalidNativeBounds)
    );
    let zero_port = ProbePlan {
        probes: vec![ProbeSpec::Udp {
            port: 0,
            payload: vec![],
            receive: false,
        }],
        ..oversize.clone()
    };
    assert_eq!(zero_port.validate(), Err(PlanValidationError::InvalidPort));
}

#[tokio::test]
async fn trace_probe_reaches_loopback_with_ordered_hop_evidence() {
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", None).unwrap(),
            vec![ProbeSpec::Trace {
                max_hops: 4,
                attempts_per_hop: 1,
            }],
        ))
        .await;
    match trace_capability() {
        TraceCapability::Executable => {
            assert_eq!(report.status, ReportStatus::Ok);
            let Some(eggprobe_core::ProbeEvidence::Trace(evidence)) = &report.probes[0].evidence
            else {
                panic!("trace evidence missing");
            };
            assert_eq!(evidence.destination, "127.0.0.1");
            assert_eq!(
                evidence.termination,
                eggprobe_core::TraceTermination::DestinationReached
            );
            assert!(!evidence.hops.is_empty());
            assert!(evidence.hops.len() <= 4);
            assert_eq!(evidence.hops[0].hop, 1);
            assert_eq!(evidence.hops[0].attempts.len(), 1);
            assert_eq!(
                evidence.hops[0].attempts[0].responder.as_deref(),
                Some("127.0.0.1")
            );
            assert_eq!(
                evidence.hops[0].attempts[0].outcome,
                eggprobe_core::NativeAttemptOutcome::DestinationReached
            );
            assert!(evidence.hops[0].attempts[0].rtt_micros.is_some());
            // Hop order is TTL order and no reverse-DNS names appear.
            let rendered = serde_json::to_string(&report).unwrap();
            assert!(!rendered.contains("localhost"));
        }
        TraceCapability::PermissionDenied => assert_trace_permission_denied(&report),
        TraceCapability::Unsupported => assert_trace_unsupported(&report),
    }
}

#[tokio::test]
async fn trace_probe_reaches_ipv6_loopback() {
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("::1", None).unwrap(),
            vec![ProbeSpec::Trace {
                max_hops: 3,
                attempts_per_hop: 1,
            }],
        ))
        .await;
    match trace_capability() {
        TraceCapability::Executable => {
            assert_eq!(report.status, ReportStatus::Ok);
            let Some(eggprobe_core::ProbeEvidence::Trace(evidence)) = &report.probes[0].evidence
            else {
                panic!("IPv6 trace evidence missing");
            };
            assert_eq!(
                evidence.termination,
                eggprobe_core::TraceTermination::DestinationReached
            );
        }
        TraceCapability::PermissionDenied => assert_trace_permission_denied(&report),
        TraceCapability::Unsupported => assert_trace_unsupported(&report),
    }
}

#[tokio::test]
async fn trace_probe_respects_configured_bounds() {
    let report = ProbeEngine::default()
        .execute(plan(
            TargetSpec::new("127.0.0.1", None).unwrap(),
            vec![ProbeSpec::Trace {
                max_hops: 5,
                attempts_per_hop: 2,
            }],
        ))
        .await;
    match trace_capability() {
        TraceCapability::Executable => {
            assert_eq!(report.status, ReportStatus::Ok);
            let Some(eggprobe_core::ProbeEvidence::Trace(evidence)) = &report.probes[0].evidence
            else {
                panic!("trace evidence missing");
            };
            assert!(evidence.hops.len() <= 5);
            assert!(evidence.hops.iter().all(|hop| hop.attempts.len() <= 2));
            // Bounded output must stay machine-serializable.
            serde_json::to_value(&report).unwrap();
        }
        TraceCapability::PermissionDenied => assert_trace_permission_denied(&report),
        TraceCapability::Unsupported => assert_trace_unsupported(&report),
    }
}

/// Shared host-policy-aware denial assertion: where the runner lacks trace
/// privilege, the loopback smoke must report the typed bounded denial at
/// `HopProbe` — never a generic failure, timeout, or silent skip.
fn assert_trace_permission_denied(report: &eggprobe_core::ProbeReport) {
    assert_eq!(report.status, eggprobe_core::ReportStatus::Failed);
    let error = report.probes[0]
        .error
        .as_ref()
        .expect("denied trace must carry a diagnostic error");
    assert_eq!(
        error.kind,
        eggprobe_core::DiagnosticErrorKind::PermissionDenied
    );
    assert_eq!(error.stage, eggprobe_core::DiagnosticStage::HopProbe);
    assert_eq!(
        error.message,
        "UDP trace requires additional local privilege"
    );
    // No dependency or OS error text may leak into the machine report.
    let rendered = serde_json::to_string(report).unwrap();
    assert!(!rendered.contains("expression"));
}

/// Shared host-policy-aware refusal assertion: where the backend cannot
/// execute the trace family even with privilege, the smoke must report typed
/// `Unsupported` at `HopProbe` with the fixed refusal message — never an
/// abort, a generic failure, or a silent skip.
fn assert_trace_unsupported(report: &eggprobe_core::ProbeReport) {
    assert_eq!(report.status, eggprobe_core::ReportStatus::Unsupported);
    let error = report.probes[0]
        .error
        .as_ref()
        .expect("refused trace must carry a diagnostic error");
    assert_eq!(error.kind, eggprobe_core::DiagnosticErrorKind::Unsupported);
    assert_eq!(error.stage, eggprobe_core::DiagnosticStage::HopProbe);
    assert_eq!(error.message, "UDP trace is not supported on this platform");
    let rendered = serde_json::to_string(report).unwrap();
    assert!(!rendered.contains("expression"));
}

#[tokio::test]
async fn trace_probe_rejects_restricted_destination_classes() {
    for host in [
        "0.0.0.0",
        "255.255.255.255",
        "224.0.0.1",
        "169.254.169.254",
        "::",
        "ff02::1",
    ] {
        let report = ProbeEngine::default()
            .execute(plan(
                TargetSpec::new(host, None).unwrap(),
                vec![ProbeSpec::Trace {
                    max_hops: 3,
                    attempts_per_hop: 1,
                }],
            ))
            .await;
        assert_eq!(report.status, ReportStatus::Failed, "host {host}");
        assert_eq!(
            report.probes[0].error.as_ref().unwrap().kind,
            eggprobe_core::DiagnosticErrorKind::Policy,
            "host {host}"
        );
    }
}

#[tokio::test]
async fn trace_probe_rejects_private_target_under_strict_policy() {
    let report = ProbeEngine {
        target_policy: TargetPolicy::Strict,
    }
    .execute(plan(
        TargetSpec::new("127.0.0.1", None).unwrap(),
        vec![ProbeSpec::Trace {
            max_hops: 3,
            attempts_per_hop: 1,
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
async fn trace_probe_never_falls_back_from_eggress_to_direct() {
    let mut routed = plan(
        TargetSpec::new("127.0.0.1", None).unwrap(),
        vec![ProbeSpec::Trace {
            max_hops: 3,
            attempts_per_hop: 1,
        }],
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

#[test]
fn trace_plan_bounds_reject_out_of_range_hops_and_attempts() {
    let base = ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target: TargetSpec::new("127.0.0.1", None).unwrap(),
        route: RouteSpec::Direct,
        probes: vec![ProbeSpec::Trace {
            max_hops: 30,
            attempts_per_hop: 3,
        }],
        execution: ExecutionPolicy::default(),
        assertions: vec![],
    };
    base.validate().unwrap();
    for spec in [
        ProbeSpec::Trace {
            max_hops: 0,
            attempts_per_hop: 3,
        },
        ProbeSpec::Trace {
            max_hops: 65,
            attempts_per_hop: 3,
        },
        ProbeSpec::Trace {
            max_hops: 30,
            attempts_per_hop: 0,
        },
        ProbeSpec::Trace {
            max_hops: 30,
            attempts_per_hop: 6,
        },
    ] {
        let plan = ProbePlan {
            probes: vec![spec],
            ..base.clone()
        };
        assert_eq!(
            plan.validate(),
            Err(PlanValidationError::InvalidNativeBounds)
        );
    }
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

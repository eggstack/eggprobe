use eggprobe_core::{
    evaluate_assertions, AssertionKind, AssertionSpec, ExecutionPolicy, ProbeEngine, ProbePlan,
    ProbeSpec, ReportStatus, RouteSpec, SchemaVersion, TargetPolicy, TargetSpec,
};
use tokio::{io::AsyncWriteExt, net::TcpListener};

fn plan(target: TargetSpec, probes: Vec<ProbeSpec>) -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::INITIAL,
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
    assert!(report.probes[0].evidence.is_some());
}

#[tokio::test]
async fn http_status_is_observed_even_when_assertion_fails() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream
            .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
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

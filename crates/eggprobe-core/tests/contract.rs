use eggprobe_core::{
    render::render_human, DiagnosticError, DiagnosticErrorKind, DiagnosticStage, DurationMicros,
    EggressRoute, ExecutionPolicy, Finding, FindingOutcome, FindingSeverity, PhaseTiming,
    ProbeEvidence, ProbeKind, ProbePlan, ProbeReport, ProbeResult, ProbeSpec, ProbeStatus,
    ReportStatus, RouteSpec, SchemaVersion, TargetSpec, Timing, ToolProvenance, ToolVersion,
};

fn fixture_plan() -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::INITIAL,
        target: TargetSpec::new("example.com", Some(443)).expect("fixture target is valid"),
        route: RouteSpec::Eggress(EggressRoute {
            expression: "http://alice:super-secret@example.net:8080?token=fixture-token".into(),
        }),
        probes: vec![ProbeSpec::Dns, ProbeSpec::Tcp { port: 443 }],
        execution: ExecutionPolicy {
            deadline: DurationMicros::from_micros(5_000_000),
            repetitions: 1,
            retries: 0,
        },
        assertions: vec![],
    }
}

fn fixture_report() -> ProbeReport {
    let plan = fixture_plan();
    let tool = ToolProvenance {
        name: "eggprobe".into(),
        version: ToolVersion::new("0.1.0").expect("fixture version is valid"),
    };
    ProbeReport {
        schema_version: plan.schema_version,
        tool,
        execution_id: "fixture-0001".into(),
        target: (&plan.target).into(),
        route: plan.route.summary(),
        status: ReportStatus::Failed,
        probes: vec![
            ProbeResult {
                kind: ProbeKind::Dns,
                status: ProbeStatus::Ok,
                timing: Some(Timing {
                    total: DurationMicros::from_micros(1250),
                    phases: vec![PhaseTiming {
                        stage: DiagnosticStage::Resolution,
                        duration: DurationMicros::from_micros(1200),
                    }],
                }),
                error: None,
                evidence: Some(ProbeEvidence::Dns(
                    eggprobe_core::domain::probe::DnsEvidence {
                        addresses: vec!["192.0.2.10".into()],
                    },
                )),
                unavailable: vec![],
                findings: vec![],
            },
            ProbeResult {
                kind: ProbeKind::Tcp,
                status: ProbeStatus::Failed,
                timing: Some(Timing {
                    total: DurationMicros::from_micros(2300),
                    phases: vec![],
                }),
                error: Some(DiagnosticError {
                    kind: DiagnosticErrorKind::ConnectionRefused,
                    stage: DiagnosticStage::DirectConnect,
                    message: "connection refused".into(),
                    attempt: Some(1),
                }),
                evidence: None,
                unavailable: vec![],
                findings: vec![],
            },
        ],
        findings: vec![Finding {
            assertion_id: "tcp-available".into(),
            severity: FindingSeverity::Error,
            outcome: FindingOutcome::Failed,
            message: "TCP probe did not establish a connection".into(),
        }],
        warnings: vec!["fixture uses synthetic addresses".into()],
    }
}

#[test]
fn plan_round_trips_against_deterministic_fixture() {
    let plan: ProbePlan = serde_json::from_str(include_str!("fixtures/plan.json")).unwrap();
    let actual = serde_json::to_string_pretty(&plan).unwrap();
    let expected = include_str!("fixtures/plan.json").trim_end();
    assert_eq!(actual, expected);
    assert_eq!(serde_json::from_str::<ProbePlan>(&actual).unwrap(), plan);
}

#[test]
fn report_matches_deterministic_json_and_round_trips() {
    let report = fixture_report();
    let actual = report.to_json().unwrap();
    let expected = include_str!("fixtures/report.json").trim_end();
    assert_eq!(actual, expected);
    assert_eq!(
        serde_json::from_str::<ProbeReport>(&actual).unwrap(),
        report
    );
    assert!(actual.contains("\"total\": 1250"));
    assert!(!actual.contains("super-secret"));
    assert!(!actual.contains("fixture-token"));
}

#[test]
fn route_debug_display_report_and_human_output_redact_credentials() {
    let plan = fixture_plan();
    let debug = format!("{plan:?}");
    let display = plan.route.to_string();
    let report = ProbeReport::from_plan(
        &plan,
        ToolProvenance {
            name: "eggprobe".into(),
            version: ToolVersion::new("0.1.0").unwrap(),
        },
        "redaction-test",
        ReportStatus::Ok,
        vec![],
    );
    let json = report.to_json().unwrap();
    let human = render_human(&report);
    for rendered in [debug, display, json, human] {
        assert!(
            !rendered.contains("super-secret"),
            "leaked password: {rendered}"
        );
        assert!(
            !rendered.contains("fixture-token"),
            "leaked token: {rendered}"
        );
    }
}

#[test]
fn malformed_required_discriminators_fail_cleanly() {
    let error = serde_json::from_str::<ProbePlan>(
        r#"{"schema_version":"0.1","target":{"host":"example.com"},"route":{"kind":"unknown"},"probes":[]}"#,
    )
    .unwrap_err();
    assert!(error.to_string().contains("unknown variant"));
}

#[test]
fn malformed_targets_fail_during_json_deserialization() {
    let error = serde_json::from_str::<ProbePlan>(
        r#"{"schema_version":"0.1","target":{"host":"not a host","port":0},"route":{"kind":"direct"},"probes":[]}"#,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("target host must not contain whitespace"));
}

#[test]
fn version_and_duration_values_are_explicit_and_distinct() {
    let report = fixture_report();
    let value: serde_json::Value = serde_json::to_value(report).unwrap();
    assert_eq!(value["schema_version"], "0.1");
    assert_eq!(value["tool"]["version"], "0.1.0");
    assert_eq!(value["probes"][0]["timing"]["total"], 1250);
}

use eggprobe_core::{
    render::render_human, DiagnosticError, DiagnosticErrorKind, DiagnosticStage, DurationMicros,
    EggressRoute, ExecutionPolicy, Finding, FindingOutcome, FindingSeverity, PhaseTiming,
    ProbeEvidence, ProbeKind, ProbePlan, ProbeReport, ProbeResult, ProbeSpec, ProbeStatus,
    ReportStatus, RouteSpec, RouteSummary, SchemaVersion, TargetSpec, Timing, ToolProvenance,
    ToolVersion,
};

fn fixture_plan() -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::CURRENT,
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
                        resolution_scope: eggprobe_core::DnsResolutionScope::Client,
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
                    route_hop_index: None,
                    route_protocol: None,
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
fn opaque_route_boundary_handles_arbitrary_supported_future_syntax() {
    let expressions = [
        "socks5://u1:p1@hop1:1080__http://u2:p2@hop2:8080?token=t2",
        "socks5://user:raw@password@hop:1080?api_key=secret",
        "http://username-only@hop:8080?password=secret&safe=value",
    ];
    for expression in expressions {
        let route = RouteSpec::Eggress(EggressRoute {
            expression: expression.into(),
        });
        assert_eq!(route.to_string(), "eggress(<redacted>)");
        assert_eq!(route.summary(), RouteSummary::Eggress);
        assert!(!format!("{route:?}").contains(expression));
    }
}

#[test]
fn invalid_tool_versions_are_rejected_during_deserialization() {
    let error = serde_json::from_str::<ToolVersion>("\"\"").unwrap_err();
    assert!(error.to_string().contains("tool version must not be empty"));
}

#[test]
fn contradictory_http_target_is_rejected_by_plan_validation() {
    let mut plan = fixture_plan();
    plan.probes = vec![ProbeSpec::Http {
        url: "http://other.example/".into(),
        method: "GET".into(),
    }];
    assert!(matches!(
        plan.validate(),
        Err(eggprobe_core::PlanValidationError::TargetMismatch { .. })
    ));
}

#[test]
fn http_authority_normalization_uses_scheme_defaults_and_ipv6() {
    let mut plan = fixture_plan();
    plan.target = TargetSpec::new("EXAMPLE.COM", Some(443)).unwrap();
    plan.probes = vec![ProbeSpec::Http {
        url: "https://example.com/".into(),
        method: "GET".into(),
    }];
    assert!(plan.validate().is_ok());

    plan.probes = vec![ProbeSpec::Http {
        url: "http://example.com/".into(),
        method: "GET".into(),
    }];
    assert!(matches!(
        plan.validate(),
        Err(eggprobe_core::PlanValidationError::PortMismatch {
            expected: 443,
            actual: 80
        })
    ));

    plan.target = TargetSpec::new("2001:DB8::1", Some(8443)).unwrap();
    plan.probes = vec![ProbeSpec::Http {
        url: "https://[2001:db8::1]:8443/".into(),
        method: "GET".into(),
    }];
    assert!(plan.validate().is_ok());
}

#[test]
fn http_authority_rejects_non_http_schemes_and_userinfo() {
    let mut plan = fixture_plan();
    for url in [
        "ftp://example.com/",
        "https://user@example.com/",
        "https://user:password@example.com/",
    ] {
        plan.probes = vec![ProbeSpec::Http {
            url: url.into(),
            method: "GET".into(),
        }];
        assert!(matches!(
            plan.validate(),
            Err(eggprobe_core::PlanValidationError::InvalidHttpUrl(_))
        ));
    }
}

#[test]
fn current_reports_have_structurally_secret_free_route_summaries() {
    let report = ProbeReport::from_plan(
        &fixture_plan(),
        ToolProvenance {
            name: "eggprobe".into(),
            version: ToolVersion::new("0.1.0").unwrap(),
        },
        "structural-route-test",
        ReportStatus::Ok,
        vec![],
    );
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(value["route"], serde_json::json!({"kind": "eggress"}));
    assert!(!value.to_string().contains("expression"));
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
    assert_eq!(value["schema_version"], "0.3");
    assert_eq!(value["tool"]["version"], "0.1.0");
    assert_eq!(value["probes"][0]["timing"]["total"], 1250);
}

use eggprobe_cli::render_report;
use eggprobe_core::{
    ProbeReport, ReportStatus, RouteSummary, SchemaVersion, TargetSummary, ToolProvenance,
    ToolVersion,
};

#[test]
fn renderer_consumes_the_core_report_type() {
    let report = ProbeReport {
        schema_version: SchemaVersion::CURRENT,
        tool: ToolProvenance {
            name: "eggprobe".into(),
            version: ToolVersion::new("0.1.0").unwrap(),
        },
        execution_id: "renderer-test".into(),
        target: TargetSummary {
            host: "example.com".into(),
            port: None,
        },
        route: RouteSummary::Direct,
        status: ReportStatus::Ok,
        probes: vec![],
        findings: vec![],
        warnings: vec![],
    };

    let rendered = render_report(&report);
    assert!(rendered.contains("eggprobe 0.1.0"));
    assert!(rendered.contains("Target: example.com"));
}

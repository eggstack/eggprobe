use eggprobe_core::schema::{plan_schema, pretty, report_schema};

#[test]
fn generated_schemas_are_deterministic_and_redacted() {
    let plan = pretty(&plan_schema());
    let report = pretty(&report_schema());
    assert_eq!(plan, pretty(&plan_schema()));
    assert_eq!(report, pretty(&report_schema()));
    assert!(plan.contains("ProbePlan"));
    assert!(report.contains("ProbeReport"));
    assert!(report.contains("expression"));
}

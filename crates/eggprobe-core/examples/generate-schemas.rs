use std::path::Path;

use eggprobe_core::schema::{plan_schema, pretty, report_schema};
use serde_json::Value;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let plan = plan_schema();
    let report = report_schema();
    write_schema(&root.join("schemas/plan-0.3.json"), &plan);
    write_schema(&root.join("schemas/report-0.3.json"), &report);
}

fn write_schema(path: &Path, schema: &schemars::Schema) {
    let mut value: Value = serde_json::from_str(&pretty(schema)).expect("schema JSON");
    value["properties"]["schema_version"] = serde_json::json!({"const": "0.3"});
    let text = serde_json::to_string_pretty(&value).expect("schema serializes");
    std::fs::write(path, format!("{text}\n")).expect("schema is writable");
}

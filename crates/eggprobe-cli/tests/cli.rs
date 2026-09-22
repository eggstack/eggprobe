use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(1);

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_eggprobe"))
}

#[test]
fn help_is_available() {
    let output = binary().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("JSON-first network diagnostics"));
    assert!(stdout.contains("Usage: eggprobe"));
}

#[test]
fn version_is_available() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "eggprobe 0.1.0"
    );
}

#[test]
fn check_without_url_does_not_create_http_finding() {
    let output = binary()
        .args(["check", "127.0.0.1", "--port", "9", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.get("findings").is_none());
    assert!(value["probes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|probe| { probe["kind"] != serde_json::Value::String("http".into()) }));
}

#[test]
fn invalid_check_range_is_usage_error() {
    let output = binary()
        .args([
            "check",
            "127.0.0.1",
            "--port",
            "9",
            "--expect-status-min",
            "500",
            "--expect-status-max",
            "200",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeds maximum"));
}

#[test]
fn batch_ndjson_is_input_ordered_and_pure() {
    let path = write_temp(
        "batch",
        &serde_json::json!({
            "plans": [empty_plan("direct"), empty_plan("direct"), empty_plan("direct")]
        }),
    );
    let output = binary()
        .args([
            "run",
            path.to_str().unwrap(),
            "--ndjson",
            "--concurrency",
            "2",
        ])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let lines: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["execution_id"], "batch-0");
    assert_eq!(lines[1]["execution_id"], "batch-1");
    assert_eq!(lines[2]["execution_id"], "batch-2");
}

#[test]
fn compare_rejects_wrong_route_roles() {
    let direct_plan = empty_plan("direct");
    let routed_plan = empty_plan("direct");
    let direct = write_temp("direct", &direct_plan);
    let routed = write_temp("routed", &routed_plan);
    let output = binary()
        .args([
            "compare",
            direct.to_str().unwrap(),
            routed.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    std::fs::remove_file(direct).unwrap();
    std::fs::remove_file(routed).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("routed input"));
}

#[tokio::test]
async fn compare_keeps_side_specific_statistics_and_negative_exit() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accept = tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
    });
    let direct_plan = tcp_plan("direct", port);
    let routed_plan = tcp_plan("not-a-route", port);
    let direct = write_temp("direct", &direct_plan);
    let routed = write_temp("routed", &routed_plan);
    let output = binary()
        .args([
            "compare",
            direct.to_str().unwrap(),
            routed.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    accept.await.unwrap();
    std::fs::remove_file(direct).unwrap();
    std::fs::remove_file(routed).unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["direct"]["successes"], 1);
    assert_eq!(value["routed"]["failures"], 1);
    assert!(value["attempts"]["direct"].is_array());
    assert!(value["attempts"]["routed"].is_array());
}

fn empty_plan(route: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "0.3",
        "target": {"host": "example.com"},
        "route": if route == "direct" {
            serde_json::json!({"kind": "direct"})
        } else {
            serde_json::json!({"kind": "eggress", "expression": route})
        },
        "probes": [],
        "execution": {"deadline": 1_000_000, "repetitions": 1, "retries": 0},
        "assertions": []
    })
}

fn tcp_plan(route: &str, port: u16) -> serde_json::Value {
    let mut plan = empty_plan(route);
    plan["target"] = serde_json::json!({"host": "127.0.0.1", "port": port});
    plan["probes"] = serde_json::json!([{"kind": "tcp", "port": port}]);
    plan
}

fn write_temp(label: &str, value: &serde_json::Value) -> std::path::PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "eggprobe-cli-{label}-{}-{id}.json",
        std::process::id()
    ));
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
}

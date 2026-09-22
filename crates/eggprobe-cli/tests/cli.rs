use std::process::Command;

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

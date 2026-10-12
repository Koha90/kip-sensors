use std::process::Command;

/// Verifies that invalid CLI input produces a nonzero exit code.
#[test]
fn invalid_sensor_id_exits_with_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_kip-sensors"))
        .args(["show", "кот"])
        .output()
        .expect("CLI process must start");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("invalid sensor ID: кот"));
}

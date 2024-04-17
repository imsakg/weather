use std::{
    path::PathBuf,
    process::{Command, Output},
};
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/forecast.json")
}
fn weather(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weather"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn help_and_invalid_arguments_do_not_require_network() {
    assert!(weather(&["--help"]).status.success());
    let empty = weather(&[]);
    assert!(!empty.status.success());
    assert!(empty.stdout.is_empty());
    assert!(!weather(&["London", "--days", "17"]).status.success());
}
#[test]
fn offline_fixture_renders_forecasts_charts_and_future_hours() {
    let output = weather(&[
        "--input",
        fixture().to_str().unwrap(),
        "--chart",
        "--hourly",
        "2",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Istanbul"));
    assert!(text.contains("12.5 °C"));
    assert!(text.contains("Daily high temperatures"));
    assert!(text.contains("Daily precipitation"));
    assert!(text.contains("2024-03-01T18:00"));
    assert!(text.contains("2024-03-01T19:00"));
    assert!(!text.contains("2024-03-01T17:00"));
}
#[test]
fn json_stdout_is_only_a_machine_readable_report() {
    let output = weather(&["--input", fixture().to_str().unwrap(), "--json"]);
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["kind"], "forecast");
    assert_eq!(json["schema_version"], 1);
}
#[test]
fn offline_cache_miss_has_no_partial_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let output = weather(&[
        "London",
        "--offline",
        "--cache-dir",
        dir.path().to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no cached weather"));
}

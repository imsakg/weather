use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use structopt::StructOpt;
use weather::{
    cache::Cache,
    cli::Options,
    report::{Report, ReportKind},
};

fn fixture() -> Report {
    serde_json::from_str(include_str!("fixtures/forecast.json")).unwrap()
}
fn options(extra: &[&str]) -> Options {
    let mut args = vec!["weather", "Istanbul", "TR", "--days", "3"];
    args.extend_from_slice(extra);
    Options::from_iter_safe(args).unwrap()
}
fn input(report: &Report, extra: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("report.json");
    std::fs::write(&file, serde_json::to_vec(report).unwrap()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_weather"))
        .arg("--input")
        .arg(file)
        .args(extra)
        .output()
        .unwrap()
}
#[test]
fn every_report_requires_nonempty_daily_data() {
    let mut report = fixture();
    report.data.daily = None;
    assert!(report.validate().is_err());
    assert!(report.json().is_err());
    let mut report = fixture();
    let daily = report.data.daily.as_mut().unwrap();
    daily.time.clear();
    daily.temperature_2m_max.clear();
    daily.temperature_2m_min.clear();
    daily.precipitation_sum.clear();
    daily.weather_code.clear();
    assert!(report.validate().is_err());
}
#[test]
fn malformed_measurements_are_rejected_consistently() {
    for (field, value) in [("relative_humidity_2m", 250.0), ("wind_speed_10m", -20.0)] {
        let mut value_json = serde_json::to_value(fixture()).unwrap();
        value_json["data"]["current"][field] = value.into();
        let report: Report = serde_json::from_value(value_json).unwrap();
        assert!(report.validate().is_err(), "{field}");
    }
    let mut report = fixture();
    report.data.daily.as_mut().unwrap().precipitation_sum[0] = Some(-10.0);
    assert!(report.validate().is_err());
    let mut report = fixture();
    report
        .data
        .hourly
        .as_mut()
        .unwrap()
        .precipitation_probability[0] = Some(150.0);
    assert!(report.validate().is_err());
    let mut report = fixture();
    report.data.daily.as_mut().unwrap().temperature_2m_max[0] = Some(1.0);
    assert!(report.validate().is_err());
    let mut report = fixture();
    report.data.current.as_mut().unwrap().temperature_2m = Some(f64::NAN);
    assert!(report.validate().is_err());
}
#[test]
fn malformed_reversed_duplicate_and_gapped_dates_are_errors() {
    for date in [
        "not-a-date",
        "2024-02-30",
        "2024-03-01",
        "2024-02-29",
        "2024-03-04",
    ] {
        let mut report = fixture();
        report.data.daily.as_mut().unwrap().time[1] = date.into();
        assert!(report.validate().is_err(), "{date}");
    }
    let mut report = fixture();
    report.data.hourly.as_mut().unwrap().time[1] = "2024-03-01T16:00".into();
    assert!(report.validate().is_err());
}
#[test]
fn null_measurements_remain_valid_and_unknown_codes_are_not_fabricated() {
    let mut report = fixture();
    report.data.current.as_mut().unwrap().weather_code = Some(999);
    report.data.daily.as_mut().unwrap().temperature_2m_max[0] = None;
    report.validate().unwrap();
    assert_eq!(report.data.daily.unwrap().rows().unwrap()[0].high, None);
}
#[test]
fn request_consistency_includes_units_kind_coordinates_and_date_coverage() {
    fixture().validate_for(&options(&[])).unwrap();
    assert!(fixture()
        .validate_for(&options(&["--units", "imperial"]))
        .is_err());
    let coordinates = Options::from_iter_safe([
        "weather",
        "--latitude",
        "1",
        "--longitude",
        "2",
        "--days",
        "3",
    ])
    .unwrap();
    assert!(fixture().validate_for(&coordinates).is_err());
    let wrong_length =
        Options::from_iter_safe(["weather", "Istanbul", "TR", "--days", "2"]).unwrap();
    assert!(fixture().validate_for(&wrong_length).is_err());
    let mut history = fixture();
    history.kind = ReportKind::History;
    history.data.current = None;
    history.data.hourly = None;
    assert!(history.validate_for(&options(&[])).is_err());
    let matching = options(&["--start", "2024-03-01", "--end", "2024-03-03"]);
    history.validate_for(&matching).unwrap();
    let shifted = options(&["--start", "2024-03-02", "--end", "2024-03-04"]);
    assert!(history.validate_for(&shifted).is_err());
}
#[test]
fn daily_period_must_begin_on_current_local_date_when_known() {
    let mut report = fixture();
    report.data.current.as_mut().unwrap().time = "2024-03-02T18:00".into();
    assert!(report.validate().is_err());
}
#[test]
fn failed_hourly_render_does_not_emit_partial_forecast_stdout() {
    let mut report = fixture();
    report.data.current = None;
    let output = input(&report, &["--hourly", "2"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("current timestamp"));
}
#[test]
fn imported_history_kind_and_units_control_presentation() {
    let mut report = fixture();
    report.kind = ReportKind::History;
    report.data.current = None;
    report.data.hourly = None;
    report.units = weather::units::Units::Imperial;
    let output = input(&report, &[]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Historical weather"));
    assert!(text.contains("High °F"));
    assert!(!text.contains("Current observations"));
}
#[test]
fn cached_reports_are_revalidated_before_offline_json_output() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf());
    let args = options(&[
        "--offline",
        "--cache-dir",
        dir.path().to_str().unwrap(),
        "--json",
    ]);
    let key = args.request_key();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    for mutation in ["ragged", "units", "kind", "missing"] {
        cache.store(&key, now, &fixture()).unwrap();
        let path = cache.path_for(&key);
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match mutation {
            "ragged" => {
                value["report"]["data"]["daily"]["temperature_2m_max"] = serde_json::json!([])
            }
            "units" => value["report"]["units"] = "imperial".into(),
            "kind" => {
                value["report"]["kind"] = "history".into();
                value["report"]["data"]["current"] = serde_json::Value::Null;
                value["report"]["data"]["hourly"] = serde_json::Value::Null;
            }
            "missing" => value["report"]["data"]["daily"] = serde_json::Value::Null,
            _ => unreachable!(),
        }
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_weather"))
            .args([
                "Istanbul",
                "TR",
                "--days",
                "3",
                "--offline",
                "--json",
                "--cache-dir",
            ])
            .arg(dir.path())
            .output()
            .unwrap();
        assert!(!output.status.success(), "{mutation}");
        assert!(output.stdout.is_empty(), "{mutation}");
    }
}
#[test]
fn valid_stale_cache_is_explicit_and_offline() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf());
    let args = options(&[
        "--offline",
        "--cache-dir",
        dir.path().to_str().unwrap(),
        "--json",
    ]);
    cache.store(&args.request_key(), 1000, &fixture()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weather"))
        .args([
            "Istanbul",
            "TR",
            "--days",
            "3",
            "--offline",
            "--json",
            "--cache-dir",
        ])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("STALE"));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["units"], "metric");
}
#[test]
fn timezone_and_report_version_are_validated_without_guessing() {
    let mut report = fixture();
    report.schema_version = 9;
    assert!(report.validate().is_err());
    let mut report = fixture();
    report.data.timezone = String::new();
    assert!(report.validate().is_err());
}
#[test]
fn normal_fixture_is_readable_from_the_exported_report_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/forecast.json");
    Report::read(&path).unwrap().validate().unwrap();
}

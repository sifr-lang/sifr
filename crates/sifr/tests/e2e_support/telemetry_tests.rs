use super::*;
use serde_json::json;

#[test]
fn test_cargo_unit_observations_include_fresh_and_rebuilt_attempts() {
    let units = telemetry::cargo_units(
        &[
            json!({"reason":"compiler-artifact","fresh":true}).to_string(),
            json!({"reason":"compiler-artifact","fresh":false}).to_string(),
            json!({"reason":"compiler-artifact","fresh":false}).to_string(),
            json!({"reason":"compiler-message"}).to_string(),
            "not-json".to_owned(),
        ]
        .join("\n"),
    );
    assert_eq!(units["fresh_units"], 1);
    assert_eq!(units["rebuilt_units"], 2);
    assert!(units["link_us"].is_null());
}

#[test]
fn test_completed_and_unfinished_spans_are_distinct() {
    let mut command = Command::new(env::current_exe().expect("test executable"));
    command
        .args([
            "e2e_support::telemetry_tests::telemetry_child",
            "--exact",
            "--ignored",
            "--nocapture",
        ])
        .env("SIFR_E2E_TELEMETRY_CHILD", "1");
    let output = run_capture_bounded(command, std::time::Duration::from_secs(10));
    assert!(output.status_ok, "{}", output.stderr);
    let rows = output
        .stderr
        .lines()
        .filter_map(|line| line.strip_prefix("[sifr-e2e-event] "))
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON event"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 6);
    let starts = rows
        .iter()
        .filter(|v| v["event"] == "start")
        .collect::<Vec<_>>();
    let ends = rows
        .iter()
        .filter(|v| v["event"] == "end")
        .collect::<Vec<_>>();
    assert_eq!(starts.len(), 3);
    assert_eq!(
        ends.iter().filter(|v| v["status"] == "incomplete").count(),
        1
    );
    for end in ends {
        let start = starts
            .iter()
            .find(|v| v["id"] == end["id"])
            .expect("matching start");
        assert_eq!(
            end["clock_us"].as_u64().unwrap() - start["clock_us"].as_u64().unwrap(),
            end["elapsed_us"].as_u64().unwrap()
        );
    }
}

#[test]
#[ignore = "explicit subprocess control invoked by test_completed_and_unfinished_spans_are_distinct"]
fn telemetry_child() {
    assert_eq!(env::var("SIFR_E2E_TELEMETRY_CHILD").as_deref(), Ok("1"));
    let root = telemetry::Span::start("control", None, json!({}));
    let done = telemetry::Span::start("completed", Some(&root.id), json!({"selected":1}));
    done.finish("pass", json!({"passed":1}));
    drop(telemetry::Span::start(
        "unfinished",
        Some(&root.id),
        json!({"selected":1}),
    ));
    root.finish("fail", json!({"incomplete":1}));
}

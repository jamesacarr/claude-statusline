use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

/// Helper to build a Command for the claude-statusline binary.
fn cmd() -> Command {
    cargo_bin_cmd!("claude-statusline")
}

/// Build a full JSON input matching the Claude Code schema.
fn full_json() -> String {
    r#"{
        "cwd": "/Users/jamescarr/Git/jamesacarr/claude-statusline",
        "session_id": "test-session-123",
        "transcript_path": "/tmp/transcript.json",
        "model": {
            "id": "claude-opus-4-20250514",
            "display_name": "Claude Opus 4"
        },
        "workspace": {
            "current_dir": "/Users/jamescarr/Git/jamesacarr/claude-statusline",
            "project_dir": "/Users/jamescarr/Git/jamesacarr/claude-statusline"
        },
        "version": "1.0.0",
        "cost": {
            "total_cost_usd": 0.05,
            "total_duration_ms": 1000,
            "total_api_duration_ms": 500,
            "total_lines_added": 10,
            "total_lines_removed": 5
        },
        "context_window": {
            "total_input_tokens": 55000,
            "total_output_tokens": 15000,
            "context_window_size": 200000,
            "used_percentage": 30.0,
            "remaining_percentage": 70.0,
            "current_usage": {
                "input_tokens": 50000,
                "output_tokens": 10000,
                "cache_creation_input_tokens": 5000,
                "cache_read_input_tokens": 5000
            }
        },
        "exceeds_200k_tokens": false,
        "agent": {
            "name": "claude"
        }
    }"#
    .to_string()
}

// --- Test 1: Valid full input ---

#[test]
fn valid_full_input_exits_zero_and_contains_expected_output() {
    cmd()
        .write_stdin(full_json())
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Opus 4"))
        .stdout(predicate::str::contains(
            "…/Git/jamesacarr/claude-statusline",
        ))
        .stdout(predicate::str::contains("\u{2588}"))
        .stdout(predicate::str::contains("\u{2591}"))
        .stdout(predicate::str::contains("30%"))
        // current_usage input tokens: 50000 + 5000 + 5000 = 60000
        .stdout(predicate::str::contains("(60.0k)"));
}

// --- Test 2: Valid input with null optionals ---

#[test]
fn valid_input_with_null_optionals_does_not_panic() {
    let json = r#"{
        "model": {
            "display_name": "Claude Opus 4"
        },
        "workspace": {
            "current_dir": "/tmp"
        },
        "session_id": "test-session",
        "context_window": {
            "total_input_tokens": 1000,
            "total_output_tokens": 500,
            "used_percentage": 10.0,
            "remaining_percentage": 90.0,
            "current_usage": null
        }
    }"#;

    cmd()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Opus 4"))
        .stdout(predicate::str::contains("/tmp"));
}

// --- Test 3: Valid input with no context percentages ---

#[test]
fn valid_input_with_no_context_percentages_omits_bar_graph() {
    let json = r#"{
        "model": {
            "display_name": "Opus"
        },
        "workspace": {
            "current_dir": "/tmp"
        },
        "context_window": {
            "remaining_percentage": null,
            "used_percentage": null
        }
    }"#;

    let output = cmd()
        .write_stdin(json)
        .output()
        .expect("command should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\u{2588}"),
        "should not contain filled bar blocks when no context percentages"
    );
    assert!(
        !stdout.contains("\u{2591}"),
        "should not contain empty bar blocks when no context percentages"
    );
}

// --- Test 4: Invalid JSON ---

#[test]
fn invalid_json_exits_zero_with_empty_stdout() {
    cmd()
        .write_stdin("not json")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// --- Test 5: Empty stdin ---

#[test]
fn empty_stdin_exits_zero_with_empty_stdout() {
    cmd()
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// --- Test 6: NO_COLOR environment variable ---

#[test]
fn no_color_env_strips_ansi_escape_sequences() {
    let json = r#"{
        "model": {
            "display_name": "Opus"
        },
        "workspace": {
            "current_dir": "/tmp"
        },
        "context_window": {
            "total_input_tokens": 5000,
            "total_output_tokens": 1000,
            "used_percentage": 50.0,
            "remaining_percentage": 50.0
        }
    }"#;

    let output = cmd()
        .env("NO_COLOR", "1")
        .write_stdin(json)
        .output()
        .expect("command should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "should not contain ANSI escape sequences when NO_COLOR is set"
    );
    assert!(
        stdout.contains("\u{2588}") || stdout.contains("\u{2591}"),
        "should still contain bar graph characters"
    );
    assert!(stdout.contains("%"), "should still contain percentage");
}

// --- Test 7: Directory truncation for deep path ---

#[test]
fn deep_directory_path_is_truncated_in_output() {
    let json = r#"{
        "model": {
            "display_name": "Opus"
        },
        "workspace": {
            "current_dir": "/Users/jamescarr/Git/jamesacarr/claude-statusline"
        }
    }"#;

    cmd()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "…/Git/jamesacarr/claude-statusline",
        ));
}

// --- Test 8: Short directory unchanged ---

#[test]
fn short_directory_path_is_not_truncated() {
    let json = r#"{
        "model": {
            "display_name": "Opus"
        },
        "workspace": {
            "current_dir": "/tmp"
        }
    }"#;

    let output = cmd()
        .write_stdin(json)
        .output()
        .expect("command should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/tmp"), "should contain /tmp");
    assert!(
        !stdout.contains("…/tmp"),
        "should not prefix short path with …/"
    );
}

// --- Test 9: Context threshold green ---

#[test]
fn context_threshold_green_at_low_usage() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "workspace": { "current_dir": "/tmp" },
        "context_window": {
            "total_input_tokens": 1000,
            "total_output_tokens": 0,
            "remaining_percentage": 92.0,
            "used_percentage": 8.0
        }
    }"#;

    // raw_used=8 -> green (< 50)
    cmd()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[32m"))
        .stdout(predicate::str::contains("8%"));
}

// --- Test 10: Context threshold orange ---

#[test]
fn context_threshold_orange_at_high_usage() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "workspace": { "current_dir": "/tmp" },
        "context_window": {
            "total_input_tokens": 10000,
            "total_output_tokens": 5000,
            "remaining_percentage": 30.0,
            "used_percentage": 70.0
        }
    }"#;

    // raw_used=70 -> orange (>= 65, < 80)
    cmd()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;5;208m"))
        .stdout(predicate::str::contains("70%"));
}

// --- Test 11: Context threshold blinking red ---

#[test]
fn context_threshold_blinking_red_at_critical_usage() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "workspace": { "current_dir": "/tmp" },
        "context_window": {
            "total_input_tokens": 90000,
            "total_output_tokens": 10000,
            "remaining_percentage": 4.0,
            "used_percentage": 96.0
        }
    }"#;

    // raw_used=96 -> blinking red (>= 80)
    cmd()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[5;31m"))
        .stdout(predicate::str::contains("96%"));
}

// --- Test 12: Separator between dir and context bar ---

#[test]
fn full_input_has_separator_between_dir_and_context_bar() {
    let output = cmd()
        .write_stdin(full_json())
        .output()
        .expect("command should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The statusline should have the structure: model | dir | context_bar
    // Split on the box-drawing separator to confirm three segments exist
    let separator = " \u{2502} ";
    let segments: Vec<&str> = stdout.trim_end().split(separator).collect();
    assert_eq!(
        segments.len(),
        3,
        "expected 3 segments (model | dir | context_bar), got {}: {:?}",
        segments.len(),
        stdout
    );

    // The third segment must contain bar graph characters, confirming context_bar is present
    assert!(
        segments[2].contains('\u{2588}') || segments[2].contains('\u{2591}'),
        "third segment should contain bar graph characters: {:?}",
        segments[2]
    );
}

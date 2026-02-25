use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::BridgeData;

/// Write a bridge file for gsd-context-monitor.js compatibility.
///
/// Creates `{tmpdir}/claude-ctx-{session_id}.json` with session context data.
/// Uses an atomic write pattern (write to .tmp, then rename).
/// All errors are silently ignored -- bridge writing is best-effort.
pub fn write_bridge(session_id: &str, remaining_percentage: f64, scaled_used: u32) {
    let dir = std::env::temp_dir();
    write_bridge_to(&dir, session_id, remaining_percentage, scaled_used);
}

/// Internal implementation that accepts a custom directory for testing.
pub(crate) fn write_bridge_to(
    dir: &Path,
    session_id: &str,
    remaining_percentage: f64,
    scaled_used: u32,
) {
    if session_id.is_empty()
        || session_id.contains('/')
        || session_id.contains("..")
        || session_id.contains('\0')
    {
        return;
    }

    let filename = format!("claude-ctx-{}.json", session_id);
    let final_path = dir.join(&filename);
    let tmp_path = dir.join(format!("{}.tmp", filename));

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return,
    };

    let bridge = BridgeData {
        session_id: session_id.to_string(),
        remaining_percentage,
        used_pct: scaled_used,
        timestamp,
    };

    let json = match serde_json::to_string(&bridge) {
        Ok(j) => j,
        Err(_) => return,
    };

    if std::fs::write(&tmp_path, &json).is_err() {
        return;
    }

    // Atomic rename; ignore errors (best-effort)
    let _ = std::fs::rename(&tmp_path, &final_path);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn writes_bridge_file_with_correct_json_fields() {
        let tmp = tempdir().unwrap();
        let session_id = "sess-abc-123";

        super::write_bridge_to(tmp.path(), session_id, 92.0, 10);

        let path = tmp.path().join(format!("claude-ctx-{}.json", session_id));
        assert!(path.exists(), "bridge file should exist");

        let content = fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(data["session_id"], "sess-abc-123");
        assert_eq!(data["remaining_percentage"], 92.0);
        assert_eq!(data["used_pct"], 10);
        assert!(
            data["timestamp"].as_u64().unwrap() > 1700000000,
            "timestamp should be a reasonable unix epoch"
        );
    }

    #[test]
    fn bridge_file_schema_matches_expected_types() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "sess-type-check", 85.5, 25);

        let path = tmp.path().join("claude-ctx-sess-type-check.json");
        let content = fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(data["session_id"].is_string(), "session_id should be a string");
        assert!(data["remaining_percentage"].is_f64(), "remaining_percentage should be a number");
        assert!(data["used_pct"].is_u64(), "used_pct should be a number");
        assert!(data["timestamp"].is_u64(), "timestamp should be a number");
    }

    #[test]
    fn does_not_write_file_for_empty_session_id() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "", 92.0, 10);

        let entries: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
        assert!(entries.is_empty(), "no file should be written for empty session_id");
    }

    #[test]
    fn does_not_write_file_for_session_id_with_path_traversal() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "../etc", 92.0, 10);

        let entries: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
        assert!(entries.is_empty(), "no file should be written for path traversal session_id");
    }

    #[test]
    fn does_not_write_file_for_session_id_with_slash() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "foo/bar", 92.0, 10);

        let entries: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
        assert!(entries.is_empty(), "no file should be written for session_id containing slash");
    }

    #[test]
    fn does_not_write_file_for_session_id_with_null_byte() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "sess\0id", 92.0, 10);

        let entries: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
        assert!(entries.is_empty(), "no file should be written for session_id containing null byte");
    }

    #[test]
    fn timestamp_is_a_reasonable_unix_epoch() {
        let tmp = tempdir().unwrap();

        super::write_bridge_to(tmp.path(), "sess-ts-check", 50.0, 50);

        let path = tmp.path().join("claude-ctx-sess-ts-check.json");
        let content = fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();

        let ts = data["timestamp"].as_u64().unwrap();
        assert!(ts > 1700000000, "timestamp {} should be after Nov 2023", ts);
    }
}

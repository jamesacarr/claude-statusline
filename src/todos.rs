use std::path::Path;

use crate::types::TodoItem;

/// Read the current in-progress task for a given session.
///
/// Looks up todo files in `~/.claude/todos/` matching the session_id pattern,
/// reads the most recent one by mtime, and returns the `activeForm` of the
/// first in-progress item.
pub fn get_current_task(session_id: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let todos_dir = home.join(".claude/todos");
    get_current_task_from(&todos_dir, session_id)
}

/// Internal implementation that accepts a custom base directory for testing.
pub(crate) fn get_current_task_from(base_dir: &Path, session_id: &str) -> Option<String> {
    if session_id.is_empty() {
        return None;
    }

    if !base_dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(base_dir).ok()?;

    // Collect matching entries with their mtimes
    let mut matching: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with(session_id) && name.contains("-agent-") && name.ends_with(".json") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(mtime) = metadata.modified() {
                    matching.push((entry.path(), mtime));
                }
            }
        }
    }

    // Sort by mtime descending (most recent first)
    matching.sort_by(|a, b| b.1.cmp(&a.1));

    // Try each file starting from most recent
    for (path, _) in &matching {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let items: Vec<TodoItem> = match serde_json::from_str(&content) {
            Ok(i) => i,
            Err(_) => continue,
        };

        if let Some(item) = items
            .iter()
            .find(|t| t.status.as_deref() == Some("in_progress"))
        {
            return Some(item.active_form.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper to create a todo JSON file in the given directory
    fn create_todo_file(dir: &std::path::Path, session_id: &str, suffix: &str, content: &str) {
        let filename = format!("{}-agent-{}.json", session_id, suffix);
        let path = dir.join(&filename);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn returns_active_form_of_in_progress_item() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        let json = r#"[
            {"content": "Fix the bug", "status": "in_progress", "activeForm": "Fixing the bug"},
            {"content": "Add tests", "status": "completed", "activeForm": "Adding tests"}
        ]"#;
        create_todo_file(&todos_dir, "sess-123", "001", json);

        let result = super::get_current_task_from(&todos_dir, "sess-123");
        assert_eq!(result, Some("Fixing the bug".to_string()));
    }

    #[test]
    fn returns_none_when_all_items_completed() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        let json = r#"[
            {"content": "Fix the bug", "status": "completed", "activeForm": "Fixing the bug"},
            {"content": "Add tests", "status": "completed", "activeForm": "Adding tests"}
        ]"#;
        create_todo_file(&todos_dir, "sess-456", "001", json);

        let result = super::get_current_task_from(&todos_dir, "sess-456");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_empty_array() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        create_todo_file(&todos_dir, "sess-789", "001", "[]");

        let result = super::get_current_task_from(&todos_dir, "sess-789");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_nonexistent_directory() {
        let tmp = tempdir().unwrap();
        let nonexistent = tmp.path().join("does-not-exist");

        let result = super::get_current_task_from(&nonexistent, "sess-123");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_empty_session_id() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        let result = super::get_current_task_from(&todos_dir, "");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_invalid_json_in_file() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        create_todo_file(&todos_dir, "sess-bad", "001", "not valid json");

        let result = super::get_current_task_from(&todos_dir, "sess-bad");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_from_most_recent_file_by_mtime() {
        let tmp = tempdir().unwrap();
        let todos_dir = tmp.path().join(".claude/todos");
        fs::create_dir_all(&todos_dir).unwrap();

        // Create an older file with one in_progress task
        let old_json = r#"[
            {"content": "Old task", "status": "in_progress", "activeForm": "Old task active"}
        ]"#;
        create_todo_file(&todos_dir, "sess-multi", "001", old_json);

        // Sleep to ensure different mtime
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Create a newer file with a different in_progress task
        let new_json = r#"[
            {"content": "New task", "status": "in_progress", "activeForm": "New task active"}
        ]"#;
        create_todo_file(&todos_dir, "sess-multi", "002", new_json);

        let result = super::get_current_task_from(&todos_dir, "sess-multi");
        assert_eq!(result, Some("New task active".to_string()));
    }
}

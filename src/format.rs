use crate::bridge;
use crate::context;
use crate::path_format;
use crate::todos;
use crate::types::StatusInput;

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Box drawing vertical line used as segment separator.
const SEPARATOR: &str = " \u{2502} ";

/// Wrap text in dim ANSI codes. Returns text unchanged when `no_color` is true.
pub fn dim(text: &str, no_color: bool) -> String {
    if no_color {
        text.to_string()
    } else {
        format!("{}{}{}", DIM, text, RESET)
    }
}

/// Wrap text in bold ANSI codes. Returns text unchanged when `no_color` is true.
pub fn bold(text: &str, no_color: bool) -> String {
    if no_color {
        text.to_string()
    } else {
        format!("{}{}{}", BOLD, text, RESET)
    }
}

/// Assemble the full statusline from parsed Claude Code JSON input.
pub fn build_statusline(input: &StatusInput, no_color: bool) -> String {
    // Extract model name
    let model_name = input
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("Claude");

    // Extract directory: prefer workspace.current_dir, fall back to cwd
    let directory = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref())
        .unwrap_or("");

    let formatted_dir = path_format::format_directory(directory);

    // Extract session_id
    let session_id = input.session_id.as_deref().unwrap_or("");

    // Get current task from todo files
    let current_task = todos::get_current_task(session_id);

    // Compute context usage
    let (remaining_pct, used_pct) = match &input.context_window {
        Some(cw) => (cw.remaining_percentage, cw.used_percentage),
        None => (None, None),
    };
    let usage = context::compute_usage(remaining_pct, used_pct);

    // Format token count
    let token_display = context::format_token_count(&input.context_window);

    // Render context bar (if usage data available)
    let context_bar = match &usage {
        Some(u) => context::render_bar(u.scaled_used, u.raw_used, &token_display, no_color),
        None => String::new(),
    };

    // Write bridge file (best-effort, only if session and remaining exist)
    if !session_id.is_empty() {
        if let Some(remaining) = remaining_pct {
            let scaled = usage.as_ref().map(|u| u.scaled_used).unwrap_or(0);
            bridge::write_bridge(session_id, remaining, scaled);
        }
    }

    // Assemble output
    let model_segment = dim(model_name, no_color);
    let dir_segment = format!("{}{}", dim(&formatted_dir, no_color), context_bar);

    match current_task {
        Some(task) => {
            format!(
                "{}{}{}{}{}",
                model_segment,
                SEPARATOR,
                bold(&task, no_color),
                SEPARATOR,
                dir_segment
            )
        }
        None => {
            format!("{}{}{}", model_segment, SEPARATOR, dir_segment)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{ContextWindow, ModelInfo, StatusInput, WorkspaceInfo};

    // --- dim tests ---

    #[test]
    fn dim_wraps_text_in_dim_ansi_codes() {
        let result = super::dim("text", false);
        assert_eq!(result, "\x1b[2mtext\x1b[0m");
    }

    #[test]
    fn dim_returns_text_unchanged_when_no_color() {
        let result = super::dim("text", true);
        assert_eq!(result, "text");
    }

    // --- bold tests ---

    #[test]
    fn bold_wraps_text_in_bold_ansi_codes() {
        let result = super::bold("text", false);
        assert_eq!(result, "\x1b[1mtext\x1b[0m");
    }

    #[test]
    fn bold_returns_text_unchanged_when_no_color() {
        let result = super::bold("text", true);
        assert_eq!(result, "text");
    }

    // --- build_statusline tests ---

    #[test]
    fn build_statusline_with_full_input_contains_model_directory_and_bar() {
        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Claude Opus 4".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some("/Users/jamescarr/Git/jamesacarr/claude-statusline".to_string()),
                ..Default::default()
            }),
            session_id: Some("test-session".to_string()),
            context_window: Some(ContextWindow {
                remaining_percentage: Some(92.0),
                used_percentage: Some(8.0),
                total_input_tokens: Some(15234),
                total_output_tokens: Some(4521),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input, false);

        assert!(
            result.contains("Claude Opus 4"),
            "should contain model name"
        );
        assert!(
            result.contains(".../Git/jamesacarr/claude-statusline"),
            "should contain truncated directory"
        );
        assert!(
            result.contains("\u{2588}"),
            "should contain bar graph filled blocks"
        );
        assert!(result.contains("8%"), "should contain raw percentage");
        assert!(result.contains("(19.8k)"), "should contain token count");
    }

    #[test]
    fn build_statusline_without_task_has_two_segments() {
        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some("/tmp".to_string()),
                ..Default::default()
            }),
            context_window: Some(ContextWindow {
                used_percentage: Some(10.0),
                total_input_tokens: Some(1000),
                total_output_tokens: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input, false);

        // The separator is \u{2502} (box drawing vertical)
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        // Without task: model | dir+context = 2 segments
        assert_eq!(
            segment_count, 2,
            "expected 2 segments without task, got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_without_context_omits_bar() {
        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some("/tmp".to_string()),
                ..Default::default()
            }),
            context_window: Some(ContextWindow {
                // Both percentages are None -- no usage info
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input, false);

        assert!(
            !result.contains("\u{2588}"),
            "should not contain filled bar blocks"
        );
        assert!(
            !result.contains("\u{2591}"),
            "should not contain empty bar blocks"
        );
    }

    #[test]
    fn build_statusline_with_minimal_input_does_not_panic() {
        let input = StatusInput::default();
        let result = super::build_statusline(&input, false);
        // Should produce some output (at minimum the model fallback "Claude")
        assert!(
            result.contains("Claude"),
            "should contain default model name"
        );
    }

    #[test]
    fn build_statusline_uses_cwd_fallback_when_workspace_is_none() {
        let input = StatusInput {
            cwd: Some("/Users/jamescarr/projects/myapp".to_string()),
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input, false);
        assert!(
            result.contains(".../jamescarr/projects/myapp"),
            "should use cwd when workspace is missing: {}",
            result
        );
    }

    #[test]
    fn build_statusline_uses_separator_with_box_drawing_character() {
        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some("/tmp".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input, false);
        assert!(
            result.contains("\u{2502}"),
            "should use box drawing vertical separator"
        );
    }
}

use crate::context;
use crate::git_branch;
use crate::path_format;
use crate::types::StatusInput;

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Box drawing vertical line used as segment separator.
const SEPARATOR: &str = " \u{2502} ";

/// Wrap text in dim ANSI codes.
pub fn dim(text: &str) -> String {
    format!("{}{}{}", DIM, text, RESET)
}

/// Wrap text in bold ANSI codes.
pub fn bold(text: &str) -> String {
    format!("{}{}{}", BOLD, text, RESET)
}

/// Assemble the full statusline from parsed Claude Code JSON input.
pub fn build_statusline(input: &StatusInput) -> String {
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

    let branch = git_branch::get_branch(directory);

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
        Some(u) => context::render_bar(*u, &token_display),
        None => String::new(),
    };
    let context_bar = context_bar.trim_start();

    // Assemble output
    let model_segment = dim(model_name);

    let mut segments = vec![model_segment, dim(&formatted_dir)];

    if let Some(ref branch_name) = branch {
        segments.push(dim(branch_name));
    }

    if !context_bar.is_empty() {
        segments.push(context_bar.to_string());
    }

    segments.join(SEPARATOR)
}

#[cfg(test)]
mod tests {
    use crate::types::{ContextWindow, ModelInfo, StatusInput, WorkspaceInfo};

    // --- dim tests ---

    #[test]
    fn dim_wraps_text_in_dim_ansi_codes() {
        let result = super::dim("text");
        assert_eq!(result, "\x1b[2mtext\x1b[0m");
    }

    // --- bold tests ---

    #[test]
    fn bold_wraps_text_in_bold_ansi_codes() {
        let result = super::bold("text");
        assert_eq!(result, "\x1b[1mtext\x1b[0m");
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
                remaining_percentage: Some(70.0),
                used_percentage: Some(30.0),
                context_window_size: Some(200000),
                total_input_tokens: Some(55000),
                total_output_tokens: Some(15000),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input);

        assert!(
            result.contains("Claude Opus 4"),
            "should contain model name"
        );
        assert!(
            result.contains("…/Git/jamesacarr/claude-statusline"),
            "should contain truncated directory"
        );
        assert!(
            result.contains("\u{2588}"),
            "should contain bar graph filled blocks"
        );
        assert!(
            result.contains("\u{2591}"),
            "should contain bar graph empty blocks"
        );
        assert!(result.contains("30%"), "should contain usage percentage");
        // 30% of 200000 = 60000
        assert!(result.contains("(60.0k)"), "should contain token count");
    }

    #[test]
    fn build_statusline_with_context_has_three_segments() {
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

        let result = super::build_statusline(&input);

        // The separator is \u{2502} (box drawing vertical)
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        // With context, no git branch: model | dir | context_bar = 3 segments
        assert_eq!(
            segment_count, 3,
            "expected 3 segments with context, got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_without_context_has_two_segments() {
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

        let result = super::build_statusline(&input);

        // The separator is \u{2502} (box drawing vertical)
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        // Without context, no git branch: model | dir = 2 segments
        assert_eq!(
            segment_count, 2,
            "expected 2 segments without context, got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_no_double_space_between_dir_and_context_bar() {
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
                used_percentage: Some(50.0),
                total_input_tokens: Some(5000),
                total_output_tokens: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = super::build_statusline(&input);

        assert!(
            !result.contains("\u{2502}  "),
            "should not contain separator followed by two spaces, got: {}",
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

        let result = super::build_statusline(&input);

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
        let result = super::build_statusline(&input);
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

        let result = super::build_statusline(&input);
        assert!(
            result.contains("…/jamescarr/projects/myapp"),
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

        let result = super::build_statusline(&input);
        assert!(
            result.contains("\u{2502}"),
            "should use box drawing vertical separator"
        );
    }

    #[test]
    fn build_statusline_without_branch_and_with_context_has_three_segments() {
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
        let result = super::build_statusline(&input);
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        assert_eq!(
            segment_count, 3,
            "expected 3 segments without branch: model | dir | context_bar, got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_without_branch_and_without_context_has_two_segments() {
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
        let result = super::build_statusline(&input);
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        assert_eq!(
            segment_count, 2,
            "expected 2 segments without branch or context: model | dir, got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_with_branch_and_context_has_four_segments() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/test-branch\n").unwrap();

        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some(tmp.path().to_string_lossy().to_string()),
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
        let result = super::build_statusline(&input);
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        assert_eq!(
            segment_count, 4,
            "expected 4 segments with branch: model | dir | branch | context_bar, got: {}",
            result
        );
        assert!(
            result.contains("test-branch"),
            "expected output to contain branch name 'test-branch', got: {}",
            result
        );
    }

    #[test]
    fn build_statusline_with_branch_and_without_context_has_three_segments() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/my-feature\n").unwrap();

        let input = StatusInput {
            model: Some(ModelInfo {
                display_name: Some("Opus".to_string()),
                ..Default::default()
            }),
            workspace: Some(WorkspaceInfo {
                current_dir: Some(tmp.path().to_string_lossy().to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = super::build_statusline(&input);
        let separator = " \u{2502} ";
        let segment_count = result.split(separator).count();
        assert_eq!(
            segment_count, 3,
            "expected 3 segments with branch but no context: model | dir | branch, got: {}",
            result
        );
        assert!(
            result.contains("my-feature"),
            "expected output to contain branch name 'my-feature', got: {}",
            result
        );
    }
}

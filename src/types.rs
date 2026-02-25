use serde::Deserialize;

/// Top-level input from Claude Code JSON piped via stdin.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct StatusInput {
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub model: Option<ModelInfo>,
    pub workspace: Option<WorkspaceInfo>,
    pub version: Option<String>,
    pub output_style: Option<serde_json::Value>,
    pub cost: Option<CostInfo>,
    pub context_window: Option<ContextWindow>,
    pub exceeds_200k_tokens: Option<bool>,
    pub vim: Option<serde_json::Value>,
    pub agent: Option<AgentInfo>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct WorkspaceInfo {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct CostInfo {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct ContextWindow {
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub context_window_size: Option<u64>,
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct AgentInfo {
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_full_status_input() {
        let json = r#"{
            "cwd": "/Users/test/project",
            "session_id": "abc-123",
            "transcript_path": "/tmp/transcript.json",
            "model": {
                "id": "claude-opus-4-20250514",
                "display_name": "Claude Opus 4"
            },
            "workspace": {
                "current_dir": "/Users/test/project",
                "project_dir": "/Users/test/project"
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
                "total_input_tokens": 15234,
                "total_output_tokens": 4521,
                "context_window_size": 200000,
                "used_percentage": 8.0,
                "remaining_percentage": 92.0,
                "current_usage": {
                    "input_tokens": 12000,
                    "output_tokens": 4000,
                    "cache_creation_input_tokens": 500,
                    "cache_read_input_tokens": 200
                }
            },
            "exceeds_200k_tokens": false,
            "agent": {
                "name": "claude"
            }
        }"#;

        let input: StatusInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.cwd, Some("/Users/test/project".to_string()));
        assert_eq!(input.session_id, Some("abc-123".to_string()));
        assert_eq!(
            input.transcript_path,
            Some("/tmp/transcript.json".to_string())
        );
        assert_eq!(input.version, Some("1.0.0".to_string()));
        assert_eq!(input.exceeds_200k_tokens, Some(false));

        let model = input.model.unwrap();
        assert_eq!(model.id, Some("claude-opus-4-20250514".to_string()));
        assert_eq!(model.display_name, Some("Claude Opus 4".to_string()));

        let workspace = input.workspace.unwrap();
        assert_eq!(
            workspace.current_dir,
            Some("/Users/test/project".to_string())
        );
        assert_eq!(
            workspace.project_dir,
            Some("/Users/test/project".to_string())
        );

        let cost = input.cost.unwrap();
        assert_eq!(cost.total_cost_usd, Some(0.05));
        assert_eq!(cost.total_duration_ms, Some(1000));
        assert_eq!(cost.total_api_duration_ms, Some(500));
        assert_eq!(cost.total_lines_added, Some(10));
        assert_eq!(cost.total_lines_removed, Some(5));

        let ctx = input.context_window.unwrap();
        assert_eq!(ctx.total_input_tokens, Some(15234));
        assert_eq!(ctx.total_output_tokens, Some(4521));
        assert_eq!(ctx.context_window_size, Some(200000));
        assert_eq!(ctx.used_percentage, Some(8.0));
        assert_eq!(ctx.remaining_percentage, Some(92.0));

        let usage = ctx.current_usage.unwrap();
        assert_eq!(usage.input_tokens, Some(12000));
        assert_eq!(usage.output_tokens, Some(4000));
        assert_eq!(usage.cache_creation_input_tokens, Some(500));
        assert_eq!(usage.cache_read_input_tokens, Some(200));

        let agent = input.agent.unwrap();
        assert_eq!(agent.name, Some("claude".to_string()));
    }

    #[test]
    fn deserializes_minimal_input_with_all_defaults() {
        let json = r#"{}"#;
        let input: StatusInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.cwd, None);
        assert_eq!(input.session_id, None);
        assert_eq!(input.model, None);
        assert_eq!(input.workspace, None);
        assert_eq!(input.context_window, None);
        assert_eq!(input.agent, None);
        assert_eq!(input.cost, None);
        assert_eq!(input.vim, None);
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"unknown_field": "value", "another": 42}"#;
        let input: StatusInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.cwd, None);
    }

    #[test]
    fn deserializes_percentage_as_integer() {
        // used_percentage may come as integer from JSON
        let json = r#"{"context_window": {"used_percentage": 8, "remaining_percentage": 92}}"#;
        let input: StatusInput = serde_json::from_str(json).unwrap();
        let ctx = input.context_window.unwrap();
        assert_eq!(ctx.used_percentage, Some(8.0));
        assert_eq!(ctx.remaining_percentage, Some(92.0));
    }

    #[test]
    fn deserializes_null_optional_fields_as_none() {
        let json = r#"{"context_window": {"current_usage": null, "used_percentage": null}}"#;
        let input: StatusInput = serde_json::from_str(json).unwrap();
        let ctx = input.context_window.unwrap();
        assert_eq!(ctx.current_usage, None);
        assert_eq!(ctx.used_percentage, None);
    }
}

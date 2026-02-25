---
task_id: claude-statusline
title: Rust binary replacing Node.js Claude Code statusline
status: planning
created: 2026-02-25T00:19:10Z
updated: 2026-02-25T00:24:29Z
current_wave: null
current_task: null
pause_reason: null
---

# Rust binary replacing Node.js Claude Code statusline

## Goal

A compiled Rust binary `claude-statusline` reads Claude Code JSON from stdin and writes a formatted ANSI statusline to stdout. It replaces `~/.claude/hooks/gsd-statusline.js` with three behavioral changes: (1) no GSD update checking, (2) directory display shows last 3 path levels with `...` prefix when path exceeds 3 levels, (3) context usage displays raw percentage AND total token count (input + output) alongside the 10-segment bar graph, where the bar fill/color uses the 80%-scaled value but the text percentage shows the raw `used_percentage` for consistency with the token count. The binary maintains bridge file compatibility with `gsd-context-monitor.js`.

## Success Criteria

1. `cargo build --release` produces a binary at `target/release/claude-statusline` under 2MB
2. Piping valid Claude Code JSON via stdin produces correctly formatted output containing: dim model name, bold in-progress task (when present), truncated directory path, colored bar graph with raw percentage and total token count
3. Piping JSON with `null` or missing optional fields (`context_window.current_usage`, `vim`, `agent`, `used_percentage`) produces output without panicking -- fallback values used
4. Piping invalid JSON or empty stdin produces empty output and exit code 0
5. Bridge file written to `{tmpdir}/claude-ctx-{session_id}.json` with fields `session_id`, `remaining_percentage`, `used_pct`, `timestamp` -- compatible with `gsd-context-monitor.js`
6. Directory path `/Users/jamescarr/Git/jamesacarr/claude-statusline` displays as `.../Git/jamesacarr/claude-statusline`; path `/Users/jamescarr` displays unchanged
7. Context bar thresholds match existing behavior: green (<63% scaled), yellow (<81%), orange 256-color (<95%), blinking red (>=95%) -- thresholds use the 80%-scaled value
8. Context text display shows raw `used_percentage` from JSON paired with total token count (input + output tokens) formatted as `{n}k` with one decimal (e.g. `8% (19.8k)`) -- these form a consistent unscaled pair
9. `cargo test` passes all unit and integration tests
10. `cargo clippy -- -D warnings` produces zero warnings
11. No `unwrap()` or `expect()` on any user-input-derived code path in non-test code
12. Output contains no GSD update check or GSD-related content

## Non-Functional Requirements

1. **Startup time < 5ms** -- measured with `hyperfine -N --warmup 3` piping sample JSON. The binary replaces a Node.js script with ~30ms overhead; sub-5ms is achievable with synchronous I/O and minimal dependencies (research: quality-standards.md)
2. **Binary size < 2MB stripped** -- release profile uses `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` (research: quality-standards.md)
3. **Never crash on malformed input** -- all user-input code paths use `Result` with `?` propagation; top-level `main()` catches all errors and outputs empty string with exit 0. No `unwrap()`/`expect()` outside tests (research: quality-standards.md, risks-edge-cases.md)
4. **Bridge file backward compatibility** -- bridge file JSON schema (`session_id`, `remaining_percentage`, `used_pct`, `timestamp`) must match exactly what `gsd-context-monitor.js` reads at lines 42-57 (research: codebase-integration.md)
5. **Forward-compatible JSON parsing** -- all input struct fields use `Option<T>` with `#[serde(default)]`; unknown fields ignored (serde default behavior). Handles future Claude Code schema additions without breaking (research: risks-edge-cases.md)
6. **NO_COLOR support** -- when `NO_COLOR` environment variable is set (any value including empty), strip all ANSI escape codes from output (research: quality-standards.md, no-color.org convention)
7. **stdin size limit** -- cap stdin read at 1MB to prevent memory exhaustion from pathological input (research: quality-standards.md)
8. **No TTY detection for color output** -- Claude Code pipes stdin/stdout to this binary and renders ANSI codes itself; stdout will never be a TTY in normal operation. Do NOT use `is_terminal()` or `isatty()` to gate color output, or ANSI codes will never be emitted. Only respect `NO_COLOR` environment variable for disabling colors (research: risks-edge-cases.md, "ANSI / Terminal Output" section)
9. **Minimum Rust toolchain 1.85** -- required for edition 2024. Enforced via `rust-version = "1.85"` in Cargo.toml, which produces a clear error on older toolchains (research: approach.md)

## Wave 1: Project Scaffolding

Status: pending

### Task 1.1: Create Cargo.toml with dependencies and release profile

- **Status:** pending
- **Files affected:** `Cargo.toml`
- **Action:** Create `Cargo.toml` at project root with:
  - `[package]` section: name `claude-statusline`, version `0.1.0`, edition `2024`, `rust-version = "1.85"`, description, license
  - `[dependencies]`: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `dirs = "6"`
  - `[dev-dependencies]`: `assert_cmd = "2"`, `predicates = "3"`, `tempfile = "3"`
  - `[profile.release]`: `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`
- **Verification:** `cargo check 2>&1` exits 0 (after Task 1.2 completes the source files)
- **Done when:** `Cargo.toml` exists with all specified dependencies, `rust-version = "1.85"`, and release profile settings
- **Retries:** 0
- **Last failure:** null

### Task 1.2: Create project skeleton with type definitions and stub modules

- **Status:** pending
- **Files affected:** `src/main.rs`, `src/lib.rs`, `src/types.rs`, `src/context.rs`, `src/path_format.rs`, `src/todos.rs`, `src/bridge.rs`, `src/format.rs`
- **Action:** Create `src/` directory and all source files:

  **`src/types.rs`** -- Define all serde structs for the Claude Code JSON input. Every field must be `Option<T>` with `#[serde(default)]` for forward compatibility. Structs:
  - `StatusInput` (top-level): `cwd`, `session_id`, `transcript_path`, `model` (Option<ModelInfo>), `workspace` (Option<WorkspaceInfo>), `version`, `output_style` (Option<serde_json::Value>), `cost` (Option<CostInfo>), `context_window` (Option<ContextWindow>), `exceeds_200k_tokens` (Option<bool>), `vim` (Option<serde_json::Value>), `agent` (Option<AgentInfo>)
  - `ModelInfo`: `id`, `display_name` (both Option<String>)
  - `WorkspaceInfo`: `current_dir`, `project_dir` (both Option<String>)
  - `CostInfo`: `total_cost_usd` (Option<f64>), `total_duration_ms` (Option<u64>), `total_api_duration_ms` (Option<u64>), `total_lines_added` (Option<u64>), `total_lines_removed` (Option<u64>)
  - `ContextWindow`: `total_input_tokens` (Option<u64>), `total_output_tokens` (Option<u64>), `context_window_size` (Option<u64>), `used_percentage` (Option<f64>), `remaining_percentage` (Option<f64>), `current_usage` (Option<CurrentUsage>), note: deserialize percentage fields as `f64` per risks research (may be int or float)
  - `CurrentUsage`: `input_tokens` (Option<u64>), `output_tokens` (Option<u64>), `cache_creation_input_tokens` (Option<u64>), `cache_read_input_tokens` (Option<u64>)
  - `AgentInfo`: `name` (Option<String>)
  - `TodoItem`: `content` (Option<String>), `status` (Option<String>), `active_form` (String, with `#[serde(alias = "activeForm")]`)
  - `BridgeData` (for serialization): `session_id` (String), `remaining_percentage` (f64), `used_pct` (u32), `timestamp` (u64) -- derive `Serialize`

  **`src/lib.rs`** -- Declare all public modules:
  ```rust
  pub mod types;
  pub mod context;
  pub mod path_format;
  pub mod todos;
  pub mod bridge;
  pub mod format;
  ```

  **`src/context.rs`** -- Stub: empty file or `// TODO: implement`

  **`src/path_format.rs`** -- Stub: empty file or `// TODO: implement`

  **`src/todos.rs`** -- Stub: empty file or `// TODO: implement`

  **`src/bridge.rs`** -- Stub: empty file or `// TODO: implement`

  **`src/format.rs`** -- Stub: empty file or `// TODO: implement`

  **`src/main.rs`** -- Minimal compiling entry point:
  ```rust
  fn main() {
      // TODO: implement
  }
  ```
- **Verification:** `cargo check 2>&1` exits 0
- **Done when:** `cargo check` succeeds with zero errors -- project skeleton compiles
- **Retries:** 0
- **Last failure:** null

## Wave 2: Core Logic Modules

Status: pending

### Task 2.1: Implement context calculation, bar graph, and color thresholds

- **Status:** pending
- **Files affected:** `src/context.rs`
- **Action:** Replace the stub `src/context.rs` with full implementation. This module translates `gsd-statusline.js` lines 24-61 to Rust with the new token count display.

  **Design note -- percentage/token consistency:** The text display shows the raw `used_percentage` from the JSON paired with total token count (input + output). The bar graph fill and color thresholds use the 80%-scaled value. This keeps the displayed pair consistent: both are unscaled values reflecting actual context usage. The bar graph is a visual indicator that exaggerates usage to warn early.

  **Public functions:**

  `pub fn compute_usage(remaining_percentage: Option<f64>, used_percentage: Option<f64>) -> Option<UsageInfo>` where `UsageInfo` is a struct with `raw_used: u32` (0-100, from `used_percentage` or derived from `remaining_percentage`), `scaled_used: u32` (0-100, scaled to 80% ceiling for bar graph). Logic:
  - If both `remaining_percentage` and `used_percentage` are `None`, return `None`
  - Prefer `used_percentage` if available: `raw_used = clamp(round(used_percentage), 0, 100)`
  - Fall back to `remaining_percentage`: `raw_used = clamp(round(100 - remaining), 0, 100)`
  - `scaled_used = clamp(round((raw_used / 80) * 100), 0, 100)` -- matches JS line 28
  - Return `UsageInfo { raw_used, scaled_used }`

  `pub fn format_token_count(ctx: &Option<ContextWindow>) -> String` -- compute total context token usage (input + output) from `context_window` level fields. Logic:
  - Extract `total_input_tokens` and `total_output_tokens` from `ContextWindow`, defaulting each to 0 if None
  - `total = total_input_tokens + total_output_tokens`
  - Format: if total >= 1000, display as `{total/1000.0:.1}k` (e.g. `19.8k`); if < 1000, display raw number (e.g. `842`). If `ctx` is None, return `"0"`
  - Rationale: uses `total_input_tokens + total_output_tokens` because these represent the total context window consumption, consistent with what `context_window_size` limits. This differs from `used_percentage` which is calculated only from input-side tokens (per codebase-integration.md). The text display pairs raw `used_percentage` with total tokens -- both are actual usage values, giving users the full picture.

  `pub fn render_bar(scaled_used: u32, raw_used: u32, token_display: &str, no_color: bool) -> String` -- build the 10-segment bar graph. Logic:
  - `filled = scaled_used / 10`
  - `bar = "█".repeat(filled) + "░".repeat(10 - filled)`
  - Color selection based on **scaled_used** (matching JS lines 52-60):
    - `scaled_used < 63`: green `\x1b[32m`
    - `scaled_used < 81`: yellow `\x1b[33m`
    - `scaled_used < 95`: orange `\x1b[38;5;208m`
    - `scaled_used >= 95`: blinking red `\x1b[5;31m` with skull prefix
  - Text percentage uses **raw_used** (unscaled): ` {color}{skull_if_critical}{bar} {raw_used}% ({token_display})\x1b[0m`
  - If `no_color` is true, omit all `\x1b[...]` sequences and the skull emoji

  **Unit tests** (in `#[cfg(test)] mod tests`):
  - `compute_usage(Some(92.0), Some(8.0))` -> raw_used=8, scaled_used=10
  - `compute_usage(Some(0.0), Some(100.0))` -> raw_used=100, scaled_used=100 (clamped)
  - `compute_usage(Some(100.0), Some(0.0))` -> raw_used=0, scaled_used=0
  - `compute_usage(Some(20.0), Some(80.0))` -> raw_used=80, scaled_used=100
  - `compute_usage(None, None)` -> None
  - `compute_usage(Some(92.0), None)` -> raw_used=8 (derived from remaining), scaled_used=10
  - `compute_usage(None, Some(8.0))` -> raw_used=8, scaled_used=10
  - `compute_usage(Some(-5.0), None)` -> clamp raw_used to 100
  - `compute_usage(Some(150.0), None)` -> clamp raw_used to 0
  - `format_token_count` with total_input=15234, total_output=4521 -> `"19.8k"` (total 19755)
  - `format_token_count` with total_input=500, total_output=342 -> `"842"`
  - `format_token_count` with None -> `"0"`
  - `format_token_count` with total_input=1000, total_output=0 -> `"1.0k"`
  - `format_token_count` with total_input=0, total_output=0 -> `"0"`
  - `render_bar` at scaled=50, raw=40 -> green, 5 filled blocks, text shows `40%`
  - `render_bar` at scaled=70, raw=56 -> yellow, text shows `56%`
  - `render_bar` at scaled=90, raw=72 -> orange 256-color, text shows `72%`
  - `render_bar` at scaled=100, raw=80 -> blinking red with skull, text shows `80%`
  - `render_bar` at scaled=0, raw=0 -> green, 0 filled blocks, text shows `0%`
  - `render_bar` with `no_color=true` -> no ANSI sequences in output
- **Verification:** `cargo test --lib context 2>&1`
- **Done when:** All context unit tests pass; `cargo test --lib context` exits 0
- **Retries:** 0
- **Last failure:** null

### Task 2.2: Implement directory path truncation

- **Status:** pending
- **Files affected:** `src/path_format.rs`
- **Action:** Replace the stub `src/path_format.rs` with implementation for the new directory display behavior.

  **Public function:**

  `pub fn format_directory(path: &str) -> String` -- truncate path to last 3 levels with `...` prefix when the path has more than 3 components (excluding the root `/`). Use `std::path::Path::components()` to split, filter out `RootDir` component for counting. Logic:
  - Split into components using `Path::components()`
  - Count non-root components
  - If count > 3: take last 3 non-root components, join with `/`, prepend `.../`
  - If count <= 3: return the original path unchanged
  - Examples from task spec:
    - `/Users/jamescarr/Git/jamesacarr/claude-statusline` (5 components) -> `.../Git/jamesacarr/claude-statusline`
    - `/Users/jamescarr` (2 components) -> `/Users/jamescarr` (unchanged)
    - `/tmp` (1 component) -> `/tmp` (unchanged)
    - `/` -> `/` (unchanged)

  **Unit tests** (in `#[cfg(test)] mod tests`):
  - 5-component path -> `.../` + last 3
  - 4-component path -> `.../` + last 3
  - 3-component path -> original unchanged
  - 2-component path -> original unchanged
  - 1-component path `/tmp` -> `/tmp`
  - Root path `/` -> `/`
  - Path with trailing slash `/Users/jamescarr/project/` -> same as without trailing slash
  - Path with spaces `/Users/james carr/My Project/foo/bar` -> `.../My Project/foo/bar`
  - Empty string -> empty string
- **Verification:** `cargo test --lib path_format 2>&1`
- **Done when:** All path_format unit tests pass; `cargo test --lib path_format` exits 0
- **Retries:** 0
- **Last failure:** null

### Task 2.3: Implement todo file reading

- **Status:** pending
- **Files affected:** `src/todos.rs`
- **Action:** Replace the stub `src/todos.rs` with implementation translating `gsd-statusline.js` lines 64-84 to Rust.

  **Public function:**

  `pub fn get_current_task(session_id: &str) -> Option<String>` -- read the current in-progress task. Logic:
  - If `session_id` is empty, return `None`
  - Resolve home dir with `dirs::home_dir()`; if None, return None
  - Construct path: `{home}/.claude/todos/`
  - If directory does not exist (`!path.exists()`), return `None`
  - Read directory entries with `std::fs::read_dir()`; on error, return `None`
  - Filter entries: filename starts with `session_id`, contains `-agent-`, ends with `.json`
  - For each matching entry, get metadata mtime; sort by mtime descending
  - Read the most recent file; parse as `Vec<TodoItem>` (from types.rs); on parse error, skip
  - Find first item with `status == Some("in_progress".to_string())`
  - Return its `active_form` field (the `activeForm` JSON field via serde alias)
  - All file I/O errors return `None` silently -- never panic

  **Unit tests** (in `#[cfg(test)] mod tests`):
  - Use `tempfile::tempdir()` to create test fixtures
  - Test with valid todo file containing an in_progress item -> returns activeForm
  - Test with todo file where all items are completed -> returns None
  - Test with empty array `[]` -> returns None
  - Test with non-existent directory -> returns None
  - Test with empty session_id -> returns None
  - Test with invalid JSON in file -> returns None (skipped)
  - Test with multiple matching files -> returns from most recent by mtime

  Note: The public function signature needs to accept a custom base path for testing. Either:
  (a) Add an internal function `get_current_task_from(base_dir: &Path, session_id: &str)` that the public function calls with the real home dir, OR
  (b) Make the public function accept an optional base path override.
  Option (a) is cleaner -- expose `get_current_task_from` as `pub(crate)` for testing and have `get_current_task` call it.
- **Verification:** `cargo test --lib todos 2>&1`
- **Done when:** All todos unit tests pass; `cargo test --lib todos` exits 0
- **Retries:** 0
- **Last failure:** null

### Task 2.4: Implement bridge file writing

- **Status:** pending
- **Files affected:** `src/bridge.rs`
- **Action:** Replace the stub `src/bridge.rs` with implementation translating `gsd-statusline.js` lines 32-44 to Rust.

  **Public function:**

  `pub fn write_bridge(session_id: &str, remaining_percentage: f64, scaled_used: u32)` -- write bridge file for context monitor. Logic:
  - If `session_id` is empty, return silently
  - Construct path: `{std::env::temp_dir()}/claude-ctx-{session_id}.json`
  - Create `BridgeData` struct (from types.rs): `session_id`, `remaining_percentage`, `used_pct: scaled_used`, `timestamp: SystemTime::now().duration_since(UNIX_EPOCH).as_secs()`
  - Serialize to JSON with `serde_json::to_string()`
  - Write using atomic pattern: write to `{path}.tmp`, then `std::fs::rename()` to final path
  - On any error (serialize, write, rename), return silently -- bridge is best-effort, matching JS behavior
  - Sanitize `session_id` before path construction: reject if it contains `/`, `..`, or null bytes (return silently)

  **Unit tests** (in `#[cfg(test)] mod tests`):
  - Use `tempfile::tempdir()` and override the temp dir path for testing. Similar to todos, use an internal function `write_bridge_to(dir: &Path, ...)` that the public function calls.
  - Test valid write -> file exists with correct JSON fields
  - Test bridge file content matches expected schema: `session_id` (string), `remaining_percentage` (number), `used_pct` (number), `timestamp` (number)
  - Test empty session_id -> no file written
  - Test session_id with path traversal chars (`../etc`) -> no file written
  - Test `timestamp` is a reasonable unix epoch (> 1700000000)
- **Verification:** `cargo test --lib bridge 2>&1`
- **Done when:** All bridge unit tests pass; `cargo test --lib bridge` exits 0
- **Retries:** 0
- **Last failure:** null

## Wave 3: Statusline Assembly and Entry Point

Status: pending

### Task 3.1: Implement ANSI formatting helpers and statusline assembly

- **Status:** pending
- **Files affected:** `src/format.rs`
- **Action:** Replace the stub `src/format.rs` with ANSI helper functions and the top-level statusline assembly function.

  **Constants:**
  ```rust
  const DIM: &str = "\x1b[2m";
  const BOLD: &str = "\x1b[1m";
  const RESET: &str = "\x1b[0m";
  ```

  **Public functions:**

  `pub fn dim(text: &str, no_color: bool) -> String` -- wrap text in dim ANSI codes. If `no_color`, return text unchanged.

  `pub fn bold(text: &str, no_color: bool) -> String` -- wrap text in bold ANSI codes. If `no_color`, return text unchanged.

  `pub fn build_statusline(input: &StatusInput, no_color: bool) -> String` -- assemble the full statusline. Logic:
  - Extract model name: `input.model.as_ref().and_then(|m| m.display_name.as_deref()).unwrap_or("Claude")`
  - Extract directory: `input.workspace.as_ref().and_then(|w| w.current_dir.as_deref()).or(input.cwd.as_deref()).unwrap_or("")`
  - Format directory with `path_format::format_directory()`
  - Extract session_id: `input.session_id.as_deref().unwrap_or("")`
  - Get current task: `todos::get_current_task(session_id)`
  - Compute context usage: `context::compute_usage(remaining_percentage, used_percentage)` where both come from `input.context_window`
  - Format token count: `context::format_token_count(&input.context_window)` -- uses `total_input_tokens + total_output_tokens` from `context_window` level
  - Render bar: `context::render_bar(scaled_used, raw_used, &token_display, no_color)` if usage is Some
  - Write bridge file: `bridge::write_bridge(session_id, remaining_pct, scaled_used)` if session and remaining exist
  - Assemble output matching the format from `gsd-statusline.js` lines 100-103:
    - With task: `{dim model} | {bold task} | {dim formatted_dir}{context_bar}`
    - Without task: `{dim model} | {dim formatted_dir}{context_bar}`
    - Separator is ` | ` (space-pipe-space) using literal `\u{2502}` (box drawing vertical) matching the JS `│`

  **Unit tests:**
  - `dim("text", false)` -> `"\x1b[2mtext\x1b[0m"`
  - `dim("text", true)` -> `"text"`
  - `bold("text", false)` -> `"\x1b[1mtext\x1b[0m"`
  - `build_statusline` with full input -> contains model, task, directory, bar
  - `build_statusline` with no task -> output has two segments (model | dir+context), not three
  - `build_statusline` with no context -> no bar rendered
  - `build_statusline` with minimal input (missing most fields) -> does not panic, produces some output
- **Verification:** `cargo test --lib format 2>&1`
- **Done when:** All format unit tests pass; `cargo test --lib format` exits 0
- **Retries:** 0
- **Last failure:** null

### Task 3.2: Implement main entry point with stdin reading and error handling

- **Status:** pending
- **Files affected:** `src/main.rs`
- **Action:** Replace the stub `src/main.rs` with the full entry point. This is the orchestration layer translating `gsd-statusline.js` lines 9-13 and 105-107 to Rust.

  **IMPORTANT: Do NOT add TTY detection.** Claude Code pipes stdin/stdout to this binary and renders ANSI codes itself. `stdout.is_terminal()` will always return `false` in normal operation. Adding TTY detection to gate color output would cause the binary to never emit ANSI codes. Only respect the `NO_COLOR` environment variable for disabling colors.

  **Implementation:**
  ```rust
  use std::io::Read;

  fn main() {
      let output = match run() {
          Ok(line) => line,
          Err(_) => String::new(),
      };
      print!("{}", output);
  }

  fn run() -> Result<String, Box<dyn std::error::Error>> {
      // Read stdin with 1MB cap
      let mut input = String::new();
      std::io::stdin().lock().take(1_048_576).read_to_string(&mut input)?;

      // Parse JSON
      let data: claude_statusline::types::StatusInput = serde_json::from_str(&input)?;

      // Check NO_COLOR -- presence of the variable (any value including empty) disables color
      // Do NOT check is_terminal() -- see note above
      let no_color = std::env::var("NO_COLOR").is_ok();

      // Build statusline
      Ok(claude_statusline::format::build_statusline(&data, no_color))
  }
  ```

  Key requirements:
  - Use `stdin().lock().take(1_048_576)` to cap at 1MB
  - Top-level `main()` catches ALL errors (JSON parse, IO, etc.) and outputs empty string
  - Exit code is always 0 (Rust's `main()` returning `()` always exits 0)
  - No `unwrap()` or `expect()` -- use `?` operator throughout `run()`
  - The `run()` function returns `Result<String, Box<dyn std::error::Error>>`
  - No TTY detection -- only `NO_COLOR` env var
- **Verification:** `cargo build 2>&1`
- **Done when:** `cargo build` succeeds with zero errors; the binary is produced at `target/debug/claude-statusline`
- **Retries:** 0
- **Last failure:** null

## Wave 4: Integration Tests

Status: pending

### Task 4.1: Add integration tests for full binary pipeline

- **Status:** pending
- **Files affected:** `tests/integration.rs`
- **Action:** Create `tests/integration.rs` using `assert_cmd` and `predicates` crates (already in dev-dependencies from Task 1.1).

  **Test cases:**

  1. **Valid full input** -- pipe complete JSON (matching the schema from codebase-integration.md) via `.write_stdin()`. Use JSON with `remaining_percentage: 92`, `used_percentage: 8`, `total_input_tokens: 15234`, `total_output_tokens: 4521`. Assert:
     - Exit code 0
     - stdout contains model display_name
     - stdout contains formatted directory (`.../` prefix for long paths)
     - stdout contains bar graph characters (`█`, `░`)
     - stdout contains `8%` (raw used_percentage) and `(19.8k)` (total input+output tokens: 15234+4521=19755)

  2. **Valid input with null optionals** -- pipe JSON where `context_window.current_usage` is null, `vim` absent, `agent` absent. Assert:
     - Exit code 0
     - stdout contains model name and directory
     - No panic or error output

  3. **Valid input with no context** -- pipe JSON where `context_window.remaining_percentage` is null and `context_window.used_percentage` is null. Assert:
     - Exit code 0
     - No bar graph in output

  4. **Invalid JSON** -- pipe `"not json"`. Assert:
     - Exit code 0
     - stdout is empty (no output, not crash)

  5. **Empty stdin** -- pipe empty string. Assert:
     - Exit code 0
     - stdout is empty

  6. **NO_COLOR environment variable** -- set `NO_COLOR=1`, pipe valid JSON. Assert:
     - Exit code 0
     - stdout does NOT contain `\x1b[` (no ANSI escape sequences)
     - stdout still contains bar graph characters and percentage

  7. **Directory truncation in output** -- pipe JSON with `workspace.current_dir` set to a deep path (e.g. `/Users/jamescarr/Git/jamesacarr/claude-statusline`). Assert:
     - stdout contains `.../Git/jamesacarr/claude-statusline`

  8. **Short directory unchanged** -- pipe JSON with `workspace.current_dir` set to `/tmp`. Assert:
     - stdout contains `/tmp` without `.../` prefix

  9. **Context threshold: green** -- pipe JSON with `remaining_percentage: 92`, `used_percentage: 8`. Math: raw_used=8, scaled=`round((8/80)*100)`=10. Assert:
     - stdout contains `\x1b[32m` (green)
     - stdout contains `8%` (raw percentage in text)

  10. **Context threshold: orange** -- pipe JSON with `remaining_percentage: 30`, `used_percentage: 70`. Math: raw_used=70, scaled=`round((70/80)*100)`=88. Assert:
      - stdout contains `\x1b[38;5;208m` (orange 256-color, since 88 >= 81 and < 95)
      - stdout contains `70%` (raw percentage in text)

  11. **Context threshold: blinking red** -- pipe JSON with `remaining_percentage: 4`, `used_percentage: 96`. Math: raw_used=96, scaled=`round((96/80)*100)`=120, clamped to 100. Assert:
      - stdout contains `\x1b[5;31m` (blinking red, since 100 >= 95)
      - stdout contains `96%` (raw percentage in text)

  Use `assert_cmd::Command::cargo_bin("claude-statusline")` to locate the binary.
- **Verification:** `cargo test --test integration 2>&1`
- **Done when:** All integration tests pass; `cargo test --test integration` exits 0
- **Retries:** 0
- **Last failure:** null

## Wave 5: Build Tooling and CI

Status: pending

### Task 5.1: Create Makefile with build, test, and lint targets

- **Status:** pending
- **Files affected:** `Makefile`
- **Action:** Create `Makefile` at project root with the following targets:

  - `help` (default): list all targets with descriptions
  - `build`: `cargo build`
  - `release`: `cargo build --release`
  - `test`: `cargo test`
  - `lint`: `cargo clippy -- -D warnings`
  - `fmt`: `cargo fmt`
  - `fmt-check`: `cargo fmt --check`
  - `check`: `cargo check`
  - `clean`: `cargo clean`
  - `bench`: `hyperfine -N --warmup 3 'echo '\''{"model":{"display_name":"Opus"},"session_id":"test","workspace":{"current_dir":"/a/b/c/d"},"context_window":{"remaining_percentage":80}}'\'' | ./target/release/claude-statusline'` (requires `release` target first)
  - `install`: `cargo install --path .`
  - `all`: `fmt-check lint test release`

  Use `.PHONY` for all targets. Use single-quotes in bash commands per user convention.
- **Verification:** `make help 2>&1`
- **Done when:** `make help` outputs a list of available targets
- **Retries:** 0
- **Last failure:** null

### Task 5.2: Create GitHub Actions CI workflow

- **Status:** pending
- **Files affected:** `.github/workflows/ci.yml`
- **Action:** Create `.github/workflows/ci.yml` for CI pipeline.

  **Triggers:** push to `main`, pull requests to `main`

  **Minimum toolchain note:** The project requires Rust 1.85+ (edition 2024). The `dtolnay/rust-toolchain@stable` action installs the latest stable toolchain, which is >= 1.85 as of February 2025. If pinning a specific version is desired, use `dtolnay/rust-toolchain@1.85.0` as a minimum.

  **Jobs:**

  1. `check` -- runs on `ubuntu-latest`:
     - `dtolnay/rust-toolchain@stable` with `components: clippy, rustfmt`
     - `actions/cache@v4` for `~/.cargo` and `target/`
     - `cargo fmt --check`
     - `cargo clippy -- -D warnings`
     - `cargo test`

  2. `build` -- runs on matrix `[ubuntu-latest, macos-latest]`, needs `check`:
     - `dtolnay/rust-toolchain@stable`
     - `actions/cache@v4`
     - `cargo build --release`
     - Upload binary artifact with `actions/upload-artifact@v4`

  3. `release` -- runs on tag `v*`, needs `build`:
     - Download artifacts from build job
     - Create GitHub Release with `softprops/action-gh-release`
     - Attach binaries for each platform

  Cache key should include `runner.os`, `hashFiles('Cargo.lock')`.
- **Verification:** `cat .github/workflows/ci.yml | head -5` shows valid YAML (visual check -- CI runs on push)
- **Done when:** `.github/workflows/ci.yml` exists with all three jobs and valid YAML syntax
- **Retries:** 0
- **Last failure:** null

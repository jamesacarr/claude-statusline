# Risks & Edge Cases Research

> Task: Create a Claude Code statusline binary in Rust that reads JSON from stdin and outputs a formatted terminal statusline. It reads JSON from stdin, computes context usage, reads todo files, writes bridge files, and outputs ANSI-colored text.
> Last researched: 2026-02-25T00:06:24Z

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| JSON schema changes in future Claude Code versions | high | high | Use `serde(default)` on all fields; use `Option<T>` for every field; never panic on missing data. The schema has already grown (e.g., `vim`, `agent` added as optional). Design for additive changes. |
| `used_percentage` / `remaining_percentage` is `null` early in session | high | medium | Documented in official docs: these fields "may be `null` early in the session". Default to 0% used when null. |
| `context_window.current_usage` is `null` before first API call | high | medium | Documented behavior. Fall back to showing 0 tokens / 0% when null. |
| Cumulative token bug (issue #13783) causes >100% values | high | medium | `total_input_tokens` / `total_output_tokens` are cumulative, not current context. The `used_percentage` field is pre-calculated by Claude Code and more reliable. Prefer `used_percentage` over manual calculation. Clamp any computed percentage to 0..100. |
| `~/.claude/todos/` directory missing or empty | medium | low | Directory may not exist on fresh installs. Check with `Path::exists()` before reading. Return empty task list on missing dir. Most files contain `[]` (empty array) -- handle gracefully. |
| Todo file contains invalid JSON | low | medium | Files are written by Claude Code itself, so corruption is rare. Use `serde_json::from_str` with error handling; skip corrupt files rather than aborting. |
| Bridge file tmpdir not writable | low | medium | `/tmp` can fill up or have permission issues in restricted environments. Use `std::env::temp_dir()`, attempt write, log warning to stderr on failure, continue with statusline output. Never block stdout output on bridge file failure. |
| Bridge file race condition (multiple sessions) | medium | low | Multiple Claude Code sessions write bridge files simultaneously. Use session_id in filename for isolation (e.g., `/tmp/claude-statusline-{session_id}.json`). Use atomic write pattern: write to temp file then rename. |
| stdin is empty (no data piped) | medium | medium | Binary invoked without piping. `serde_json::from_reader(stdin)` will block waiting for EOF. Use `stdin().lock()` and handle `UnexpectedEof` error. Output a sensible default (blank or error indicator) rather than hanging. |
| stdin contains malformed JSON | low | high | If Claude Code malfunctions or the binary is invoked manually with bad input. `serde_json::from_reader` returns `Err` -- catch it, output a fallback statusline (e.g., `[error: invalid input]`), exit 0 so Claude Code does not show blank. |
| Very large stdin payload | low | low | The JSON payload is small (< 2KB based on official schema). However, defend against pathological input: use `serde_json::from_reader` (streaming) rather than reading entire stdin to String first. serde_json has a built-in 128-level recursion limit. |
| Terminal does not support ANSI escape codes | low | medium | `TERM=dumb`, `NO_COLOR` env var, or piped output. Check `NO_COLOR` env var (any value = disable colors). Check `TERM=dumb`. Consider not emitting ANSI when stdout is not a TTY (though Claude Code typically pipes this, so TTY check may be counterproductive -- Claude Code renders the ANSI itself). |
| Unicode width miscalculation in bar/text display | medium | low | Emoji, CJK, and combining characters have variable display widths. Use `unicode-width` crate if doing column alignment. For the 10-segment bar graph using block characters, width is fixed and predictable. Risk is mainly in model names and directory paths. |
| serde_json stack overflow on deeply nested input | very low | high | serde_json has a default 128-level recursion limit. The statusline JSON schema is flat (max 3 levels deep). No action needed beyond default protection. |

## Edge Cases

### Stdin / JSON Parsing
- **Empty stdin (0 bytes)**: `serde_json::from_reader` returns `Err(Error { io: UnexpectedEof })`. Output fallback statusline.
- **Partial JSON (truncated mid-stream)**: Same error handling as empty stdin. Claude Code cancels in-flight scripts on new updates, so this can happen.
- **Extra whitespace / trailing newlines**: serde_json handles this natively; no issue.
- **JSON with unknown fields**: Use `#[serde(deny_unknown_fields)]` sparingly -- prefer ignoring unknowns to be forward-compatible with new Claude Code versions.
- **Numeric fields as floats vs ints**: `used_percentage` is documented as integer in examples (e.g., `8`, `92`) but could potentially be a float. Deserialize as `f64` and truncate/round for display.

### Context Window Values
- **`used_percentage` is 0**: Valid early state. Show empty bar.
- **`used_percentage` is negative**: Should not happen per schema, but clamp to 0.
- **`used_percentage` exceeds 100**: Can happen if cumulative tokens are used (bug #13783). Clamp to 100 for bar display, but show actual value in text.
- **`used_percentage` is NaN/Infinity**: Not possible in JSON (NaN/Infinity are not valid JSON numbers). serde_json rejects them. No risk.
- **`context_window_size` is 0**: Would cause division-by-zero if computing percentage manually. Guard with `if size > 0` or rely on pre-calculated `used_percentage`.
- **`context_window_size` is 1,000,000 (extended context)**: Documented possibility. Bar graph scales the same way (percentage-based), but token count display should handle 7-digit numbers.

### Path Handling
- **Root path `/`**: Truncation to "last 3 levels" yields `/`. Handle gracefully -- display as-is.
- **Home directory `~` or `/Users/jamescarr`**: Common case. Truncation works normally.
- **Single-component path like `/tmp`**: Display as-is; fewer than 3 components means no truncation needed.
- **Paths with spaces**: `/Users/james carr/My Project`. No shell escaping needed since this is internal string processing, not shell invocation.
- **Paths with unicode characters**: `/Users/jamescarr/proyecto/espanol`. Use `std::path::Path` components, which handle UTF-8 natively on macOS/Linux.
- **Very long path components**: `/Users/jamescarr/a]really-extremely-long-directory-name-that-goes-on/...`. Even truncated to 3 levels, individual components can be long. Consider max-width truncation of individual components if needed.
- **Trailing slashes**: `/Users/jamescarr/project/` -- `Path::components()` ignores trailing slashes.
- **Windows paths (future)**: `C:\Users\...` uses different separators. Not a concern for initial macOS/Linux target, but `std::path::Path` handles both if cross-platform is desired later.
- **Symlinks in path**: `cwd` from Claude Code is the resolved path. No special handling needed.

### Todo File Reading
- **Todo file is `[]` (empty array)**: Most common case (observed in `~/.claude/todos/`). Display "no current task" or omit task section.
- **Todo file is `{}` (wrong type)**: Unexpected but possible. Deserialize as `Vec<TodoItem>` -- serde will return error. Skip file.
- **Todo file has no `in_progress` items**: All items are `completed` or `pending`. Display first pending item, or nothing.
- **Todo file has unicode in task content**: Observed in real files (e.g., task descriptions with special chars). No issue with Rust's native UTF-8 strings.
- **Hundreds of todo files in directory**: Observed: 345 files in `~/.claude/todos/`. Must filter by session_id to find the right file, not iterate all. Filename pattern is `{session_id}-agent-{agent_id}.json`.
- **Todo file being written while we read**: Claude Code may be updating the file. Open with read-only, handle partial read errors gracefully.
- **Permission denied on todo directory**: Unlikely on user's own home dir, but handle `io::Error` and skip.

### Bridge File Writing
- **Bridge file directory does not exist**: `std::env::temp_dir()` should always exist. Create parent dirs with `create_dir_all` if using a subdirectory.
- **Disk full**: `write_all` returns `Err`. Log to stderr, continue with stdout output.
- **Concurrent writes from multiple sessions**: Use session_id in filename. Each session writes its own file.
- **Stale bridge files from dead sessions**: Not this binary's responsibility to clean up, but consider TTL-based cleanup or document the accumulation risk.
- **Bridge file read by external tool while being written**: Use atomic write (write temp + rename) to prevent partial reads.

### ANSI / Terminal Output
- **`NO_COLOR` environment variable set**: Spec says presence (any value, including empty) disables color. Check `std::env::var("NO_COLOR").is_ok()`.
- **`TERM=dumb`**: No ANSI support. Strip escape codes.
- **Output piped (not a TTY)**: Claude Code itself pipes to this binary and renders the ANSI codes. So `isatty(stdout)` will be false in normal operation -- do NOT use TTY detection to disable colors, or colors will never work.
- **Terminal with limited color support**: 16-color terminals may not render 256-color or truecolor codes. Stick to basic 16-color ANSI codes (30-37, 90-97 foreground) for maximum compatibility.
- **ANSI codes in model names or task descriptions**: If upstream data contains escape codes, they could corrupt output. Sanitize by stripping `\x1b[...m` sequences from input data before embedding in output.

### Numeric Display
- **Token counts with thousands separators**: 150234 tokens is harder to read than 150,234 or 150K. Consider human-friendly formatting.
- **Cost as very small float**: `0.00001` USD. Format with appropriate precision.
- **Duration in milliseconds overflow**: `total_duration_ms` as very large number (multi-day session). Handle gracefully with hours display.

## Backward Compatibility

No breaking changes concern since this is a greenfield project. However, **forward compatibility with Claude Code's JSON schema is critical**:

- The schema has already evolved (fields like `vim`, `agent`, `output_style` were added over time).
- Fields documented as "may be absent" (`vim`, `agent`) vs "may be null" (`current_usage`, `used_percentage`) require different handling in serde: `Option<T>` with `#[serde(default)]` for absent fields, `Option<T>` for nullable fields.
- New top-level or nested fields will be added. Use `#[serde(flatten)] other: serde_json::Value` or simply allow unknown fields (serde's default behavior with structs).
- The `version` field in the JSON can be used to detect Claude Code version if behavior needs to diverge.

## Fragile Areas

- **Percentage clamping logic** -- The interplay between `used_percentage` (pre-calculated, sometimes null), `current_usage` (detailed but null before first API call), and cumulative totals (misleadingly named) creates multiple code paths. The "80% scaling" adds another layer. This area needs thorough unit tests with boundary values.
- **Todo file session matching** -- Matching the current `session_id` to todo filenames requires parsing the filename pattern `{session_id}-agent-{agent_id}.json`. If Claude Code changes this naming convention, todo lookup breaks silently (returns no tasks rather than crashing). The filename pattern is not documented in the official statusline API.
- **ANSI escape code string building** -- Manual string concatenation with escape codes is error-prone. A missed reset (`\033[0m`) causes color bleed into subsequent terminal content. Use a helper function or crate that ensures reset codes are always appended.
- **Bridge file path construction** -- Constructing paths from session_id without sanitization could allow path traversal if session_id ever contains `/` or `..`. Session IDs appear to be UUIDs (safe), but validate format before using in path construction.

## Unknowns

- **Bridge file format and consumers**: The task mentions "bridge file writing for context monitoring" but the specific format and downstream consumers are not defined. The Planner needs to specify what data goes in bridge files and who reads them.
- **80% scaling rationale**: The task says "context usage scaled to 80% limit" but it is unclear whether this means the bar fills at 80% usage (treating 80% as the effective limit) or the percentage itself is multiplied by 0.8. The Planner should clarify the exact formula.
- **Todo file naming convention stability**: The `{session_id}-agent-{agent_id}.json` pattern in `~/.claude/todos/` is observed empirically but not documented. It could change without notice.
- **Performance budget**: Claude Code cancels in-flight statusline scripts if a new update triggers. The binary should complete well under 300ms (the debounce interval). Reading hundreds of todo files or doing expensive I/O could exceed this. The Planner should decide whether to scan all todo files or use a targeted lookup.
- **`NO_COLOR` interaction with Claude Code's own rendering**: Since Claude Code pipes the output and renders it, it is unclear whether Claude Code strips ANSI codes itself when appropriate, or whether the statusline binary is responsible. If Claude Code handles it, respecting `NO_COLOR` in the binary may be unnecessary or even counterproductive.

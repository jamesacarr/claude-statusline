# Codebase Integration Research

> Task: Removing all compatibility with gsd-context-monitor.js from the claude-statusline Rust binary. Identify every file, function, struct, enum variant, conditional branch, format string, test, and comment that references or supports gsd-context-monitor.js. Map the full dependency graph of this compatibility layer within the codebase.
> Last researched: 2026-02-24T00:00:00Z (manual timestamp -- MCP time unavailable)

## Affected Code

| File/Module | Role | Change Type |
|------------|------|-------------|
| `src/bridge.rs` | Entire module -- writes bridge JSON file for gsd-context-monitor.js | **delete** |
| `src/types.rs` (lines 80-87) | `BridgeData` struct + doc comment referencing gsd-context-monitor.js | **modify** (remove struct + comment) |
| `src/types.rs` (lines 234-246) | `serializes_bridge_data` test | **modify** (remove test) |
| `src/format.rs` (line 1) | `use crate::bridge;` import | **modify** (remove import) |
| `src/format.rs` (lines 73-79) | Bridge-writing conditional block inside `build_statusline()` | **modify** (remove block) |
| `src/lib.rs` (line 1) | `pub mod bridge;` module declaration | **modify** (remove line) |

## Detailed Dependency Map

### 1. `src/bridge.rs` -- DELETE ENTIRE FILE

The entire file exists solely for gsd-context-monitor.js compatibility. Contains:

- **Line 6**: Doc comment `/// Write a bridge file for gsd-context-monitor.js compatibility.`
- **Line 11**: `pub fn write_bridge(session_id, remaining_percentage, scaled_used)` -- public API
- **Line 17**: `pub(crate) fn write_bridge_to(dir, session_id, remaining_percentage, scaled_used)` -- testable internal variant
- **Lines 23-29**: Session ID validation guard (path traversal prevention)
- **Lines 31-57**: Bridge file construction, atomic write logic
- **Lines 60-174**: 7 unit tests:
  - `writes_bridge_file_with_correct_json_fields` (line 66)
  - `bridge_file_schema_matches_expected_types` (line 88)
  - `does_not_write_file_for_empty_session_id` (line 110)
  - `does_not_write_file_for_session_id_with_path_traversal` (line 123)
  - `does_not_write_file_for_session_id_with_slash` (line 136)
  - `does_not_write_file_for_session_id_with_null_byte` (line 149)
  - `timestamp_is_a_reasonable_unix_epoch` (line 162)

### 2. `src/types.rs` -- REMOVE `BridgeData` struct and its test

- **Lines 80-87**: `BridgeData` struct with `#[derive(Debug, Serialize)]` and 4 fields (`session_id`, `remaining_percentage`, `used_pct`, `timestamp`). Doc comment on line 80 explicitly references gsd-context-monitor.js.
- **Lines 234-246**: Test `serializes_bridge_data` which constructs a `BridgeData` and verifies JSON serialization.

After removal, check whether `Serialize` from serde is still needed by any other struct. Currently **no other struct** in `types.rs` derives `Serialize` -- all others only derive `Deserialize`. If `BridgeData` is removed, the `Serialize` import in line 1 (`use serde::{Deserialize, Serialize}`) can be simplified to `use serde::Deserialize`.

### 3. `src/format.rs` -- REMOVE bridge call site

- **Line 1**: `use crate::bridge;` -- remove this import
- **Lines 73-79**: The bridge-writing block inside `build_statusline()`:
  ```rust
  // Write bridge file (best-effort, only if session and remaining exist)
  if !session_id.is_empty() {
      if let Some(remaining) = remaining_pct {
          let scaled = usage.as_ref().map(|u| u.scaled_used).unwrap_or(0);
          bridge::write_bridge(session_id, remaining, scaled);
      }
  }
  ```
  This is the **only call site** for `bridge::write_bridge`. After removal, the `remaining_pct` variable (line 59) is only used by `context::compute_usage` (line 62), which still needs it. The `scaled` local variable disappears entirely. The `session_id` variable (line 52) is still used by `todos::get_current_task` (line 55).

### 4. `src/lib.rs` -- REMOVE module declaration

- **Line 1**: `pub mod bridge;` -- remove this line. The remaining modules (`context`, `format`, `path_format`, `todos`, `types`) are unaffected.

## Entry Points

The bridge compatibility layer has one entry point into the rest of the system:

- `format::build_statusline()` at `src/format.rs:33` calls `bridge::write_bridge()` at line 77
- `bridge::write_bridge()` uses `types::BridgeData` (imported at `src/bridge.rs:4`)

There are **no other entry points**. No external binary, CLI flag, or environment variable controls bridge behavior. The bridge write happens unconditionally when session_id and remaining_percentage are both present.

## Existing Patterns to Follow

- Module removal pattern: delete the `.rs` file, remove the `pub mod` line from `src/lib.rs`, remove all `use crate::bridge` imports from consumer files. The codebase has no `mod.rs` pattern -- it uses the flat module style.
- Unused import cleanup: `cargo clippy -- -D warnings` (run in CI at `.github/workflows/ci.yml:35`) will catch any leftover dead imports after removal.

## Shared Code to Reuse

No shared utilities need to be introduced. This is purely a removal task.

## Dependencies

### Crate dependencies affected

- **`serde` `Serialize` derive**: Currently used only by `BridgeData`. After removing `BridgeData`, the `Serialize` derive is unused. The `serde` crate itself remains needed for `Deserialize`. The `features = ["derive"]` in `Cargo.toml` stays since `Deserialize` still uses it.
- **`tempfile` dev-dependency**: Used by both `bridge.rs` tests and `todos.rs` tests. After deleting `bridge.rs`, `tempfile` is still needed by `todos.rs` tests. No change to `Cargo.toml`.
- **`serde_json`**: Still needed by `bridge.rs` for serialization and by `types.rs`/`todos.rs` for deserialization. After removing `bridge.rs`, `serde_json` is still used elsewhere. No change.

### No new dependencies needed

## Data Flow

### Before (current)

```
stdin JSON
  |
  v
main.rs::run() --> serde_json::from_str --> StatusInput
  |
  v
format::build_statusline(&data, no_color)
  |
  +-- extract session_id, remaining_pct, used_pct
  +-- todos::get_current_task(session_id)
  +-- context::compute_usage(remaining_pct, used_pct) --> UsageInfo
  +-- context::format_token_count(&context_window)
  +-- context::render_bar(scaled_used, raw_used, token_display, no_color)
  +-- bridge::write_bridge(session_id, remaining, scaled)  <-- SIDE EFFECT: writes file to $TMPDIR
  |       |
  |       v
  |   types::BridgeData --> serde_json::to_string --> fs::write + fs::rename
  |
  v
  assemble string --> stdout
```

### After (target)

```
stdin JSON
  |
  v
main.rs::run() --> serde_json::from_str --> StatusInput
  |
  v
format::build_statusline(&data, no_color)
  |
  +-- extract session_id, remaining_pct, used_pct
  +-- todos::get_current_task(session_id)
  +-- context::compute_usage(remaining_pct, used_pct) --> UsageInfo
  +-- context::format_token_count(&context_window)
  +-- context::render_bar(scaled_used, raw_used, token_display, no_color)
  |
  v
  assemble string --> stdout
```

The file-system side effect (`$TMPDIR/claude-ctx-{session_id}.json` writes) is completely eliminated. The binary becomes a pure stdin-to-stdout transformer with only the read-only `~/.claude/todos/` access remaining.

## Summary of Changes by Scope

| Scope | Count | Details |
|-------|-------|---------|
| Files to delete | 1 | `src/bridge.rs` |
| Files to modify | 3 | `src/types.rs`, `src/format.rs`, `src/lib.rs` |
| Structs to remove | 1 | `BridgeData` |
| Functions to remove | 2 | `write_bridge`, `write_bridge_to` |
| Tests to remove | 8 | 7 in `bridge.rs`, 1 in `types.rs` (`serializes_bridge_data`) |
| Comments to remove | 3 | doc comments on `BridgeData`, `write_bridge`, bridge block in `format.rs` |
| Import lines to remove | 3 | `use crate::bridge` in `format.rs`, `pub mod bridge` in `lib.rs`, `Serialize` from `types.rs` line 1 |
| Lines of code removed (approx) | ~190 | 175 lines in `bridge.rs` + ~15 lines across other files |
| Files unchanged | 5 | `src/main.rs`, `src/context.rs`, `src/todos.rs`, `src/path_format.rs`, `tests/integration.rs` |

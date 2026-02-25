# Codebase Integration Research

> Task: Remove the task/todo segment completely from the statusline and add a separator between the dir_segment and the context_bar
> Last researched: 2026-02-24T00:00:00Z (approximate -- time MCP unavailable)

## Affected Code

| File/Module | Role | Change Type |
|------------|------|-------------|
| `src/format.rs` | Assembles the full statusline output | modify |
| `src/todos.rs` | Reads in-progress task from `~/.claude/todos/` | delete |
| `src/lib.rs` | Module declarations | modify |
| `src/types.rs` | Contains `TodoItem` struct | modify |
| `tests/integration.rs` | End-to-end binary tests | modify |

## Entry Points

The statusline is assembled in `src/format.rs:build_statusline()` (line 32). This is the single function that:
1. Calls `todos::get_current_task(session_id)` (line 54) to get the current task
2. Builds `dir_segment` by concatenating `formatted_dir` and `context_bar` with no separator (line 74)
3. Uses a `match current_task` block (lines 76-90) to decide between a 3-segment layout (model | task | dir+context) and a 2-segment layout (model | dir+context)

Both changes happen entirely within `build_statusline()`.

## Existing Patterns to Follow

- **ANSI wrapping helpers:** `dim()` and `bold()` in `src/format.rs` (lines 14-29) wrap text in ANSI codes respecting `no_color`. Any new separator styling should use these or follow the same pattern.
- **SEPARATOR constant:** `src/format.rs:11` defines `const SEPARATOR: &str = " \u{2502} ";` (box-drawing vertical with spaces). Reuse this for the new separator between `dir_segment` and `context_bar`.
- **Module removal pattern:** Recent commits (see `7bc57eb`, `c9f1770`, `60b62cd`, `5e54b0b`) show the established pattern for removing a module: delete the file, remove `pub mod` from `lib.rs`, remove `use` imports from consuming modules, and remove related type definitions.

## Shared Code to Reuse

- `SEPARATOR` constant at `src/format.rs:11` -- reuse for the new dir/context separator
- `dim()` at `src/format.rs:14` -- if the separator should be dimmed to match the existing visual style

## Dependencies

- The `dirs` crate (in `Cargo.toml` line 12) is used only by `src/todos.rs:11` (`dirs::home_dir()`). After removing `todos.rs`, the `dirs` dependency can be removed from `Cargo.toml`.
- No new dependencies are needed.

## Data Flow

### Before (current)

```
stdin JSON -> StatusInput
  -> model.display_name -> model_segment (dim)
  -> workspace.current_dir / cwd -> formatted_dir -> dir_segment = dim(formatted_dir) + context_bar
  -> session_id -> todos::get_current_task() -> current_task
  -> context_window -> compute_usage() + render_bar() -> context_bar

Assembly (with task):  model_segment | bold(task) | dir_segment
Assembly (no task):    model_segment | dir_segment

Where dir_segment = dim(formatted_dir) + context_bar  (no separator between them)
```

### After (proposed)

```
stdin JSON -> StatusInput
  -> model.display_name -> model_segment (dim)
  -> workspace.current_dir / cwd -> formatted_dir
  -> context_window -> compute_usage() + render_bar() -> context_bar

Assembly (always):  model_segment | dim(formatted_dir) | context_bar

Where | is the SEPARATOR constant (" \u{2502} ")
Note: context_bar may be empty when no usage data is available
```

## Detailed Change Map

### `src/format.rs` -- modify

1. **Remove** `use crate::todos;` import (line 3)
2. **Remove** the `session_id` extraction (line 51) and `current_task` lookup (line 54)
3. **Change** `dir_segment` construction (line 74): currently `format!("{}{}", dim(&formatted_dir, no_color), context_bar)` concatenates dir and context_bar directly. Instead, keep them as separate segments with SEPARATOR between them (when context_bar is non-empty).
4. **Replace** the `match current_task` block (lines 76-90) with a single assembly path:
   - When `context_bar` is non-empty: `model_segment + SEPARATOR + dim(formatted_dir) + SEPARATOR + context_bar`
   - When `context_bar` is empty: `model_segment + SEPARATOR + dim(formatted_dir)`
5. **Update tests:** Remove/update `build_statusline_without_task_has_two_segments` (line 168) -- the segment count assertion will change. The test `build_statusline_with_full_input_contains_model_directory_and_bar` (line 128) should still pass since it only checks for presence of substrings, but verify segment count expectations.

### `src/todos.rs` -- delete

Entire file is removed. Contains `get_current_task()`, `get_current_task_from()`, and all associated tests.

### `src/lib.rs` -- modify

Remove `pub mod todos;` (line 4).

### `src/types.rs` -- modify

Remove the `TodoItem` struct (lines 71-78) and its two test functions:
- `deserializes_todo_item_with_active_form_alias` (line 207)
- `deserializes_todo_item_with_snake_case_active_form` (line 217)

Note: `session_id` field on `StatusInput` (line 8) could be kept or removed. It is still deserialized from JSON input. Removing it is optional -- keeping it avoids a breaking change to the JSON contract. Recommend keeping it for forward compatibility.

### `Cargo.toml` -- modify

Remove `dirs = "6"` from `[dependencies]` (line 12) since it is only used by `todos.rs`. Also remove `tempfile = "3"` from `[dev-dependencies]` (line 17) if no other test code uses it -- currently only `todos.rs` tests use `tempfile`.

### `tests/integration.rs` -- modify

No integration tests currently assert on task content, so changes are minimal. However, the segment count in the output will change when `context_bar` is present (3 separator instances instead of 1 if context_bar gets its own separator). Tests that count segments or split on SEPARATOR may need updating. Specifically, check that `valid_full_input_exits_zero_and_contains_expected_output` (line 56) still passes -- it checks for presence of substrings, not structure, so it should be fine.

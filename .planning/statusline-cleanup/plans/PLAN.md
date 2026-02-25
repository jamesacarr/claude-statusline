---
task_id: statusline-cleanup
title: Remove task/todo segment and add dir-context separator
status: planning
created: 2026-02-24T17:45:00Z
updated: 2026-02-24T18:15:00Z
current_wave: null
current_task: null
pause_reason: null
---

# Remove task/todo segment and add dir-context separator

## Goal

When this plan is complete, the task/todo feature is fully removed from the codebase (no dead code, no unused dependencies), and the statusline always renders as `model | dir | context_bar` when context data exists or `model | dir` when it does not. All tests pass, clippy and fmt are clean, and the `dirs` and `tempfile` crates are no longer in the dependency tree.

## Success Criteria

1. `src/todos.rs` does not exist
2. `pub mod todos;` is not present in `src/lib.rs`
3. `TodoItem` struct is not present in `src/types.rs` and its two deserialization tests are removed
4. `dirs` is not in `[dependencies]` and `tempfile` is not in `[dev-dependencies]` in `Cargo.toml`
5. `build_statusline()` in `src/format.rs` has no reference to `todos`, `session_id`, or `current_task`
6. When context data is present, the statusline output contains exactly two `SEPARATOR` instances producing three segments: model, dir, and context_bar
7. When context data is absent, the statusline output contains exactly one `SEPARATOR` instance producing two segments: model and dir
8. The output of `build_statusline` with context data does not contain `\u{2502}  ` (separator followed by two spaces). Verified by: `build_statusline_no_double_space_between_dir_and_context_bar` test passes
9. `cargo fmt --check` passes with no formatting issues
10. `cargo clippy -- -D warnings` passes with no warnings
11. `cargo test` passes -- all unit and integration tests reflect the new layout

## Non-Functional Requirements

1. **Performance (minor improvement):** Removing `todos::get_current_task()` eliminates a filesystem scan (`read_dir` + `read_to_string` + JSON parse) from every invocation. Verified by: the call no longer exists in `build_statusline()`.
2. **Binary size (minor improvement):** Removing the `dirs` crate shrinks the dependency tree. Verified by: `dirs` absent from `Cargo.toml` and `Cargo.lock` after `cargo update`.

## Wave 1: Delete todo module and remove TodoItem type
Status: pending

### Task 1.1: Delete src/todos.rs
- **Status:** pending
- **Files affected:** `src/todos.rs`
- **Action:** Delete the file `src/todos.rs` entirely. This file contains `get_current_task()`, `get_current_task_from()`, and all associated tests. It is the sole consumer of the `dirs` crate and `tempfile` dev-dependency.
- **Verification:** `test ! -f src/todos.rs`
- **Done when:** `src/todos.rs` does not exist on the filesystem
- **Retries:** 0
- **Last failure:** null

### Task 1.2: Remove TodoItem struct and its tests from src/types.rs
- **Status:** pending
- **Files affected:** `src/types.rs`
- **Action:** Remove the `TodoItem` struct definition (lines ~71-78) and its two test functions `deserializes_todo_item_with_active_form_alias` (lines ~206-214) and `deserializes_todo_item_with_snake_case_active_form` (lines ~216-222) from `src/types.rs`. Keep the `use serde::Deserialize;` import (still needed by remaining structs). Do not remove the `session_id` field from `StatusInput` -- it may be used by other consumers and its presence is harmless.
- **Verification:** `grep -c 'TodoItem' src/types.rs` returns 0, then `cargo check` succeeds
- **Done when:** `grep 'TodoItem' src/types.rs` returns no matches and `cargo check` succeeds
- **Retries:** 0
- **Last failure:** null

### Task 1.3: Remove dirs and tempfile dependencies from Cargo.toml
- **Status:** pending
- **Files affected:** `Cargo.toml`
- **Action:** Remove `dirs = "6"` from `[dependencies]`. Remove `tempfile = "3"` from `[dev-dependencies]`. `tempfile` is only used by `src/todos.rs` tests (confirmed via grep -- no other test files import it). Keep `serde`, `serde_json`, `assert_cmd`, and `predicates` unchanged.
- **Verification:** `grep -Ec 'dirs|tempfile' Cargo.toml` returns 0
- **Done when:** Neither `dirs` nor `tempfile` appear in `Cargo.toml`
- **Retries:** 0
- **Last failure:** null

## Wave 2: Update lib.rs and format.rs
Status: pending

### Task 2.1: Remove todos module declaration from src/lib.rs
- **Status:** pending
- **Files affected:** `src/lib.rs`
- **Action:** Remove the line `pub mod todos;` from `src/lib.rs`. The remaining module declarations (`context`, `format`, `path_format`, `types`) stay unchanged. This follows the same module removal pattern used in recent commits (e.g., `c9f1770` which removed `pub mod bridge;`).
- **Verification:** `grep -c 'todos' src/lib.rs` returns 0
- **Done when:** `src/lib.rs` does not contain `todos` and `cargo check` succeeds
- **Retries:** 0
- **Last failure:** null

### Task 2.2: Simplify build_statusline() in src/format.rs
- **Status:** pending
- **Files affected:** `src/format.rs`
- **Action:** Modify `src/format.rs` as follows:
  1. **Remove** the `use crate::todos;` import (line 3).
  2. **Remove** the `session_id` extraction (line ~51: `let session_id = ...`) and the `current_task` lookup (line ~54: `let current_task = ...`).
  3. **Trim the leading space** from `context_bar`: after the existing `context_bar` match block (lines ~63-71), add `let context_bar = context_bar.trim_start();`. This returns a `&str` (no allocation needed). The `render_bar` function in `src/context.rs` prepends a leading space to its output, but `SEPARATOR` already ends with a space, so the trim prevents a double-space.
  4. **Replace** lines ~72-90 (the `dir_segment` construction and `match current_task` block) with a single assembly:
     - When `context_bar` is non-empty (after trimming): `format!("{}{}{}{}{}", model_segment, SEPARATOR, dim(&formatted_dir, no_color), SEPARATOR, context_bar)`
     - When `context_bar` is empty: `format!("{}{}{}", model_segment, SEPARATOR, dim(&formatted_dir, no_color))`
  5. The `bold()` function is `pub` in a library crate, so clippy will not warn about dead code for it. Leave it in place.
- **Verification:** `cargo check` succeeds. Then verify no todo references remain: `grep -c 'todos' src/format.rs` returns 0, `grep -c 'current_task' src/format.rs` returns 0 (excluding test code), `grep -Ec 'session_id' src/format.rs` returns 0 (excluding test code)
- **Done when:** `build_statusline()` has no references to todos, session_id, or current_task, and produces the new layout
- **Retries:** 0
- **Last failure:** null

## Wave 3: Update all tests
Status: pending

### Task 3.1: Update unit tests in src/format.rs
- **Status:** pending
- **Files affected:** `src/format.rs` (test module only)
- **Action:** Update the `#[cfg(test)] mod tests` section in `src/format.rs`:
  1. **Update** `build_statusline_without_task_has_two_segments` (line ~168): Rename to `build_statusline_with_context_has_three_segments`. Change the assertion from `segment_count == 2` to `segment_count == 3` because the layout is now `model | dir | context_bar` (3 segments when split on SEPARATOR). The test input already includes `context_window` with `used_percentage: Some(10.0)`, so it will produce a non-empty context bar.
  2. **Add** a new test `build_statusline_without_context_has_two_segments` that constructs a `StatusInput` with no `context_window` data (or with both percentages as `None`) and asserts `segment_count == 2` (model | dir).
  3. **Verify** `build_statusline_with_full_input_contains_model_directory_and_bar` (line ~128) still passes -- it checks for substring presence, not structure. The separator between dir and context_bar is now explicit, which should not break these assertions.
  4. **Verify** `build_statusline_without_context_omits_bar` (line ~201) still passes -- no context bar means no trailing separator. Confirm no dangling separator in output.
  5. **Verify** `build_statusline_with_minimal_input_does_not_panic` (line ~231) still passes -- default input has no context data, so output should be just the model with no trailing separator.
  6. **Remove** any test-only references to `TodoItem` if present (none currently exist in format.rs tests).
  7. **Add** a test `build_statusline_no_double_space_between_dir_and_context_bar` that constructs a `StatusInput` with context data and asserts the output does not contain `"\u{2502}  "` (separator character followed by two spaces) when context_bar is present.
- **Verification:** `cargo test --lib format` passes all tests
- **Done when:** All format.rs unit tests pass and segment count assertions match the new layout
- **Retries:** 0
- **Last failure:** null

### Task 3.2: Update integration tests in tests/integration.rs
- **Status:** pending
- **Files affected:** `tests/integration.rs`
- **Action:** Review and update `tests/integration.rs`:
  1. **Verify** `valid_full_input_exits_zero_and_contains_expected_output` (line ~56) -- this test checks for substring presence (`contains("Claude Opus 4")`, `contains(".../Git/...")`, `contains("\u{2588}")`, etc.). It does not assert segment counts. It should pass without changes, but verify by running.
  2. **Verify** all other integration tests (Tests 2-11) -- none assert on task content or segment count. They check for substring presence, ANSI codes, and NO_COLOR behavior. These should pass without changes.
  3. **If any test fails**, fix the assertion. The most likely candidate is if any test implicitly depends on the old `dir+context_bar` concatenation (no separator) producing a specific substring. With the new `SEPARATOR` between dir and context_bar, substrings that span the boundary will no longer match.
  4. **Optional enhancement**: Add a new integration test `full_input_has_separator_between_dir_and_context_bar` that pipes `full_json()` and asserts the output contains the separator character between the directory and the bar graph. Use the `output()` approach to inspect the full stdout string.
- **Verification:** `cargo test --test integration` passes all tests
- **Done when:** All integration tests pass
- **Retries:** 0
- **Last failure:** null

## Wave 4: Final validation
Status: pending

### Task 4.1: Run full CI checks
- **Status:** pending
- **Files affected:** (none -- validation only)
- **Action:** Run the full CI validation suite from the project root:
  1. `cargo fmt --check` -- verify no formatting issues
  2. `cargo clippy -- -D warnings` -- verify no clippy warnings (pay attention to: unused imports, dead code from removed todo references)
  3. `cargo test` -- verify all unit and integration tests pass
  4. `cargo build --release` -- verify the release build succeeds (confirms no linking issues from removed `dirs` crate)
  If `bold()` triggers a dead-code warning from clippy (unlikely since it is `pub` in a library crate), remove it and its test.
- **Verification:** All four commands exit 0 with no warnings or errors
- **Done when:** `cargo fmt --check` exits 0, `cargo clippy -- -D warnings` exits 0, `cargo test` exits 0, `cargo build --release` exits 0
- **Retries:** 0
- **Last failure:** null

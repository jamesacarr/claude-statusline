# Quality & Standards Research

> Task: Remove the task/todo segment completely from the statusline; add a separator between the dir_segment and the context_bar
> Last researched: 2026-02-24T00:00:00Z (approximate)

## Security

Not applicable. This change is purely cosmetic -- it removes a feature (todo display) and adds a visual separator character. No new inputs, no network calls, no file reads introduced. The existing `todos.rs` file-reading code is being removed, which marginally reduces attack surface (no more filesystem reads from `~/.claude/todos/`).

## Performance

Not applicable. Removing the todo lookup (`todos::get_current_task`) eliminates a filesystem scan (`read_dir` + `read_to_string` + JSON parse) from every invocation, which is a minor performance improvement. The new separator is a static string concatenation -- zero overhead.

## Accessibility

Not applicable (no UI changes in the traditional sense). The output is a terminal statusline rendered by Claude Code. However, note:

- The separator character `\u{2502}` (box drawing vertical) is already used between other segments. Adding it between dir and context bar maintains visual consistency.
- Ensure the separator is visible in `NO_COLOR` mode -- the existing `SEPARATOR` constant is plain text (no ANSI wrapping), so it works correctly regardless of color mode.

## Testing Strategy

- **Test types needed:** Unit tests (in `src/format.rs`), integration tests (in `tests/integration.rs`)
- **Key test cases:**

  1. **Statusline no longer contains task segment** -- with any input, the output should never contain 3 separator-delimited segments (was: model | task | dir+context; now: model | dir+context)
  2. **Separator between dir and context bar** -- when context data is present, the dir_segment and context_bar should be separated by the `SEPARATOR` constant instead of being concatenated directly
  3. **No separator when context bar is empty** -- when no context data is available, no trailing separator should appear after the dir
  4. **NO_COLOR mode** -- separator appears correctly without ANSI codes
  5. **Minimal input** -- default/empty StatusInput still produces valid output without panics
  6. **Integration: full JSON** -- update `tests/integration.rs` to reflect the new 2-segment layout (model | dir separator context) since the task segment is gone

- **Mocking approach:** No mocking needed. The `todos::get_current_task` call is being removed entirely, so no mock is required. Unit tests use in-memory `StatusInput` structs. Integration tests pipe JSON via stdin.

- **Edge cases to cover:**
  - Context bar is empty (no `context_window` or both percentages null) -- no dangling separator
  - Context bar is present -- separator appears between dir text and bar
  - `NO_COLOR=1` with context bar -- separator renders without ANSI artifacts

- **Existing test patterns:**
  - Unit tests in each module under `#[cfg(test)] mod tests` -- see `src/format.rs:93-280`, `src/context.rs:91-275`, `src/todos.rs:70-183`, `src/path_format.rs:30-93`, `src/types.rs:80-232`
  - Integration tests in `tests/integration.rs` using `assert_cmd` + `predicates` crate pattern: build a `Command`, pipe JSON via `write_stdin`, assert on stdout content
  - Tests use `StatusInput::default()` and `..Default::default()` for partial construction
  - Tests validate both presence and absence of specific Unicode characters and ANSI escape sequences

## Standards Checklist

1. `cargo fmt --check` passes with no formatting issues
2. `cargo clippy -- -D warnings` passes with no warnings
3. `cargo test` passes -- all existing tests updated or removed to reflect the new layout
4. No dead code: `src/todos.rs` module removed (or its `use` in `format.rs` removed), `TodoItem` type in `src/types.rs` removed if unused, `pub mod todos` removed from `src/lib.rs`
5. Existing `build_statusline_without_task_has_two_segments` test in `src/format.rs:168-198` still passes (this test already validates the 2-segment no-task layout, which becomes the only layout)
6. Tests that referenced task/todo behavior are removed or updated (no tests for `build_statusline` with `current_task = Some(...)` since that path no longer exists)
7. Integration test `valid_full_input_exits_zero_and_contains_expected_output` in `tests/integration.rs:56-69` updated to not expect 3 segments
8. The `SEPARATOR` constant is used (not a raw string) for the new dir-to-context-bar separator to maintain consistency
9. Separator only appears between dir and context bar when context bar is non-empty
10. `NO_COLOR` mode produces correct output -- no ANSI leaks around the new separator
11. The `session_id` field can remain in `StatusInput` (it may be used by other consumers) -- only the todo lookup call and `todos` module should be removed

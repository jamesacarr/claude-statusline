# Risks & Edge Cases Research

> Task: Remove the task/todo feature completely from the statusline and add a separator between the dir_segment and the context_bar.
> Last researched: 2026-02-24T17:00:00Z

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tests that assert on segment count or output structure break | high | low | Several unit tests (`src/format.rs:168-198` — `build_statusline_without_task_has_two_segments`) and integration tests explicitly count separators or match the output layout. These must be updated to match the new 3-segment format (model, dir, context_bar). Straightforward but must not be missed. |
| Dead code warnings from leftover `TodoItem` type | high | low | Removing `src/todos.rs` and its call in `src/format.rs:54` will leave `TodoItem` in `src/types.rs:71-78` unreferenced. The compiler will emit dead-code warnings. Must also remove `TodoItem` and its tests (`src/types.rs:207-221`). |
| `dirs` crate becomes an unused dependency | high | low | `dirs` is only used in `src/todos.rs:11`. After deletion, `Cargo.toml` line `dirs = "6"` becomes dead weight. Remove it. Similarly, `tempfile` dev-dependency is only used by `src/todos.rs` tests (bridge was already removed). Check whether any other test file still needs it before removing. |
| Separator placement when context_bar is empty | medium | medium | Currently `context_bar` is an empty string when no usage data exists (`src/format.rs:69`). If the new format is `{dir} {sep} {context_bar}`, an empty `context_bar` would render a trailing separator: `model | dir | `. Must conditionally omit the separator when `context_bar` is empty, or the output looks broken. |
| NO_COLOR mode output mismatch | low | low | The separator addition changes the no-color output format. The integration test `no_color_env_strips_ansi_escape_sequences` (`tests/integration.rs:159-192`) checks for bar characters and percentages but not segment layout. Should still pass, but verify. |
| Planning docs and architecture docs become stale | medium | low | Multiple files in `.planning/codebase/` reference `todos.rs`, `TodoItem`, and the todo-related data flow (e.g., `ARCHITECTURE.md:37`, `CONVENTIONS.md:32`, `INTEGRATIONS.md:12`, `STACK.md:22`, `TESTING.md:55`). These will be inaccurate post-change. Update or note as out-of-date. |

## Edge Cases

- **Context window data is None** -- `context_bar` is empty string (`src/format.rs:69`). The new separator between `dir_segment` and `context_bar` must be suppressed. Expected: output is `{model} | {dir}` with no trailing separator.
- **Context window present but both percentages are None** -- Same result as above: `compute_usage` returns `None` (`src/context.rs:26`), so `context_bar` is empty. Expected: no separator before empty bar.
- **Context window present with valid data** -- `context_bar` is a non-empty string. Expected: `{model} | {dir} | {context_bar}` with the new separator visible.
- **Empty/default `StatusInput`** -- `build_statusline_with_minimal_input_does_not_panic` test (`src/format.rs:231-239`). No context data, no dir, model falls back to "Claude". Expected: `Claude` with no trailing separators or bars.
- **Very long directory path combined with long context bar** -- The dir is already truncated to 3 components (`src/path_format.rs:19-21`), but adding a separator pushes the total width further. Terminal width is not under this binary's control (Claude Code handles it), so this is cosmetic only.
- **SEPARATOR constant reuse** -- The existing `SEPARATOR` constant (`src/format.rs:11`) is ` \u{2502} ` (space-box_vertical-space). Decide whether the dir-to-context_bar separator should use the same constant or a different visual treatment. Using the same constant is consistent; using something different (e.g., just a space) may be more visually appropriate since the context bar already has a leading space (`src/context.rs:72,86`).
- **Double space between dir and context_bar** -- Currently `context_bar` output from `render_bar` starts with a leading space (`src/context.rs:72`: `format!(" {} ..."`). If the new separator also includes spaces (as `SEPARATOR` does), the result would be `dir | context_bar` with the leading space in `context_bar` creating `dir |  bar` (double space). Either strip the leading space from `context_bar` or adjust the separator.

## Backward Compatibility

No breaking changes to external consumers. This binary reads stdin JSON and writes to stdout. The output format changes, but:
- The format is consumed exclusively by Claude Code's statusline renderer.
- There is no documented/stable output contract -- the output is visual ANSI text.
- Removing the task segment is the explicit goal, so the layout change is intentional.

**Dependency removals:**
- `dirs` crate can be removed from `[dependencies]` in `Cargo.toml`.
- `tempfile` dev-dependency: verify no other test files use it before removing. Currently only `src/todos.rs` imports it. Integration tests (`tests/integration.rs`) use `assert_cmd` and `predicates`, not `tempfile`.

## Fragile Areas

- `src/format.rs:74` -- The `dir_segment` is assembled by concatenating `formatted_dir` and `context_bar` directly. Splitting these into separate segments with a conditional separator requires restructuring the assembly logic at lines 73-89. The `match current_task` block (lines 76-90) will be removed entirely, simplifying the function, but the new conditional separator logic must handle the "no context data" case cleanly.
- `src/format.rs:168-198` -- The test `build_statusline_without_task_has_two_segments` explicitly counts segments by splitting on the separator character. This test logic must be updated to expect the new segment count (2 when no context bar, 3 when context bar is present).
- `src/types.rs:71-78` -- `TodoItem` struct. If not removed, the compiler will warn about dead code. Its tests at lines 207-221 will also need removal. However, the `#[serde(alias = "activeForm")]` attribute means it was specifically designed for the todo JSON format -- it has no other use.

## Unknowns

- **Desired separator style between dir and context_bar**: The task says "add a separator" but does not specify whether it should be the same `SEPARATOR` constant (` | ` using box-drawing vertical) or something else. The Planner should decide. Using the existing `SEPARATOR` is the simplest and most consistent choice.
- **Leading space in context_bar output**: `render_bar` (`src/context.rs:72,86`) prepends a space to its output. If the new separator already provides padding, this creates a visual double-space. The Planner should decide whether to strip the leading space from `render_bar` output or adjust the separator.

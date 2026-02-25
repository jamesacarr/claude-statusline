# Approach Research

> Task: Remove the task/todo segment completely from the statusline and add a separator between the dir_segment and the context_bar.
> Last researched: 2026-02-24T00:00:00Z (approximate)

## Current State

The statusline is assembled in `build_statusline()` at `src/format.rs:32-91`.

**Current layout (with task):** `dim(model) │ bold(task) │ dim(dir)context_bar`
**Current layout (without task):** `dim(model) │ dim(dir)context_bar`

Key observations:
- The task segment comes from `todos::get_current_task(session_id)` (line 54)
- The `dir_segment` concatenates `dim(formatted_dir)` and `context_bar` directly (line 74): `format!("{}{}", dim(&formatted_dir, no_color), context_bar)`
- The `context_bar` already has a leading space (from `render_bar` in `src/context.rs:72` and `85-87`)
- The `SEPARATOR` constant is `" \u{2502} "` (space, box-drawing vertical, space) at line 11

## Viable Approaches

### Approach 1: Minimal In-Place Edit

- **What:** Modify `build_statusline()` to remove the task branch and insert a separator between dir and context bar, all within `src/format.rs`. Keep `todos.rs` and `TodoItem` type intact but unused.
- **How:**
  1. Remove the `use crate::todos;` import and the `current_task` variable (lines 3, 54)
  2. Remove the `match current_task` block (lines 76-90), replace with a single `format!` that always produces `model | dir | context_bar`
  3. Change line 74 from concatenating dir+context_bar directly to separating them with the `SEPARATOR` constant (or a new separator character)
  4. Handle the case where `context_bar` is empty (no context data) -- omit the separator
- **Pros:**
  - Smallest diff, fewest files touched
  - No risk of breaking anything outside `format.rs`
  - Fast to implement and review
- **Cons:**
  - Leaves dead code (`todos.rs`, `TodoItem` struct, `dirs` dependency usage for todos)
  - Technical debt accumulates
- **Best when:** You want a quick change and plan a separate cleanup pass later
- **Sources:** `src/format.rs:32-91`, `src/todos.rs`, `src/context.rs:66-89`

### Approach 2: Full Cleanup (Recommended)

- **What:** Remove the task/todo functionality entirely (code, types, tests, dependency if unused), and add the separator between dir and context bar.
- **How:**
  1. Delete `src/todos.rs`
  2. Remove `pub mod todos;` from `src/lib.rs` (line 4)
  3. Remove `TodoItem` struct from `src/types.rs` (lines 71-78) and its tests (lines 206-231)
  4. Remove `use crate::todos;` from `src/format.rs` (line 3)
  5. Remove `session_id` extraction and `current_task` lookup from `build_statusline()` (lines 51-54)
  6. Simplify `build_statusline()` to always produce: `model_segment + SEPARATOR + dir_segment + separator + context_bar`
  7. Handle empty `context_bar`: when no context data exists, omit the trailing separator+bar entirely
  8. Update all tests in `src/format.rs` that assert segment counts or task-related behavior
  9. Check if `dirs` crate is used elsewhere -- if only by `todos.rs`, remove from `Cargo.toml`
- **Pros:**
  - Clean removal with no dead code
  - Reduces binary size (removes `dirs` crate if unused elsewhere, removes filesystem I/O)
  - Tests accurately reflect the new behavior
  - `dirs` crate is only used in `todos.rs` (confirmed via grep), so it can be removed
- **Cons:**
  - Larger diff across more files
  - More test updates required
- **Best when:** You want a clean codebase without dead code (standard practice for owned projects)
- **Sources:** `src/todos.rs`, `src/lib.rs:4`, `src/types.rs:71-78`, `Cargo.toml:12`

### Approach 3: Configurable Separator

- **What:** Instead of hardcoding the separator between dir and context bar, make the separator configurable or use a different visual style (e.g., a thinner separator, a dimmed pipe, etc.).
- **How:**
  1. Same as Approach 2 for the todo removal
  2. Introduce a second separator constant (e.g., `DIR_CTX_SEPARATOR`) distinct from the segment `SEPARATOR` to visually distinguish the dir-to-context-bar boundary from the model-to-dir boundary
  3. Options: same `│` character, a dimmed `│`, a thinner `|`, or just extra spacing
- **Pros:**
  - Allows visual hierarchy (major segments vs sub-segments)
  - Could use a dimmer/thinner separator to indicate the context bar is metadata about the dir, not a separate segment
- **Cons:**
  - Over-engineering for a simple statusline
  - No user-facing config mechanism exists yet
- **Best when:** You want visual distinction between the primary separator (model | dir) and the secondary one (dir | context)
- **Sources:** `src/format.rs:10-11`

## Recommendation

**Approach 2 (Full Cleanup)** is the right choice. This is a personal project with a clean git history (recent commits already show bridge removal cleanup), so leaving dead code is inconsistent with the established pattern. The `dirs` dependency is only used by `todos.rs`, so removing it also shrinks the dependency tree and binary.

For the separator between dir and context bar, reuse the existing `SEPARATOR` constant (`" │ "`) for consistency. The statusline becomes:

- **With context:** `dim(model) │ dim(dir) │ context_bar`
- **Without context:** `dim(model) │ dim(dir)`

Note: `render_bar` in `src/context.rs` currently prefixes its output with a space (lines 72 and 85-87). When adding the `SEPARATOR` between dir and context bar, this leading space should be removed or the separator should account for it to avoid double-spacing.

## Open Questions

1. **Separator style for dir-to-context boundary:** Should it use the same `" │ "` as the model-to-dir separator, or a visually distinct one? The task description says "a separator" without specifying the character -- defaulting to the existing `SEPARATOR` constant seems reasonable for consistency.
2. **Leading space in `render_bar`:** The bar currently starts with `" "` (e.g., `" ████░░░░░░ 8% (19.8k)"`). If using `SEPARATOR` (which ends with a space), the leading space in `render_bar` should be removed to avoid `"│  ████..."` (double space). This is a minor formatting detail the Planner should decide.

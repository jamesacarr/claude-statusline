# Plan Critique

> Task: statusline-cleanup
> Reviewed: 2026-02-24T18:42:00Z
> Verdict: has objections

## Objections

### Objection 1: Wave 1 verification commands will fail because the project cannot compile mid-wave

- **Category:** internal-consistency
- **Severity:** high
- **Affected tasks:** Task 1.2
- **Evidence:** Task 1.1 deletes `src/todos.rs`. Task 1.2's verification includes `cargo check`. But after Wave 1 completes, `src/lib.rs` still contains `pub mod todos;` (not removed until Task 2.1) and `src/format.rs` still contains `use crate::todos;` (not removed until Task 2.2). The compiler cannot resolve the deleted module.
- **Problem:** The executor will run Task 1.2's verification (`cargo check`), it will fail with a module-not-found error, and the task will be marked as failed even though the actual edit to `types.rs` was correct. This blocks the entire plan at Wave 1. Tasks within a wave may run in parallel and complete before the next wave, so there is no guarantee that `lib.rs` or `format.rs` changes happen first.
- **Suggestion:** Two options: (a) Remove `cargo check` from Task 1.2's verification -- use only the grep check (`grep -c 'TodoItem' src/types.rs` returns 0). The compile check can wait for Wave 2. Or (b) collapse Waves 1 and 2 into a single wave, since all files are distinct (`src/todos.rs`, `src/types.rs`, `Cargo.toml`, `src/lib.rs`, `src/format.rs`). This allows `cargo check` as a Wave-end verification. Option (b) is preferred because it also validates the compile state earlier.

### Objection 2: No task updates Cargo.lock, contradicting NFR 2 verification claim

- **Category:** internal-consistency
- **Severity:** medium
- **Affected tasks:** Task 1.3, Task 4.1
- **Evidence:** NFR 2 states: "Verified by: `dirs` absent from `Cargo.toml` and `Cargo.lock` after `cargo update`." No task in any wave runs `cargo update`. Task 1.3 removes `dirs` and `tempfile` from `Cargo.toml` but does not update `Cargo.lock`. Task 4.1 runs `cargo build --release` which implicitly updates the lockfile, but the NFR claims verification via `cargo update` specifically. The `Cargo.lock` will still list `dirs` (and its transitive deps) until something regenerates it.
- **Problem:** An executor or verifier checking NFR 2 literally would look for a `cargo update` step and a `Cargo.lock` check, finding neither. The lockfile cleanup happens as a side effect of `cargo build` in Task 4.1 but is never explicitly verified.
- **Suggestion:** Add `Cargo.lock` to Task 1.3's files-affected list and add a step in its action: "Run `cargo update` to regenerate `Cargo.lock` without the removed dependencies." Add `grep -c '"dirs"' Cargo.lock` returning 0 to its verification. Alternatively, revise NFR 2's verification text to match reality: "Verified by: `dirs` absent from `Cargo.toml`; lockfile updated implicitly by subsequent `cargo build --release` in Task 4.1."

### Objection 3: Task 3.2 action is too vague for an executor to act without interpretation

- **Category:** internal-consistency
- **Severity:** medium
- **Affected tasks:** Task 3.2
- **Evidence:** Task 3.2's action says: "If any test fails, fix the assertion. The most likely candidate is if any test implicitly depends on the old `dir+context_bar` concatenation (no separator) producing a specific substring." The plan schema (Constraints item 4) requires actions specific enough that "the executor does not need to interpret intent."
- **Problem:** "If any test fails, fix the assertion" requires the executor to diagnose an unspecified failure and decide on a fix with no guidance. After reviewing the actual integration tests in `tests/integration.rs`, none assert on segment count or task content -- they use only `predicate::str::contains` for substring checks and `String::from_utf8_lossy` for negative assertions. All 11 tests should pass without modification.
- **Suggestion:** Replace the conditional repair language with a definitive statement: "All 11 integration tests should pass without modification. None assert on segment count, task content, or cross-boundary substrings (verified: Tests 1-11 use `predicate::str::contains` and substring checks only). Run `cargo test --test integration` as verification. If any test fails unexpectedly, escalate -- do not modify integration tests in this task."

## Observations

- The previous critique (reviewed 2026-02-24T18:02:00Z) raised four objections: missing wave status lines, macOS grep alternation syntax, ambiguous leading-space handling in Task 2.2, and untestable SC8. The current plan revision addressed the wave status lines (now present), improved the grep syntax, clarified the `trim_start()` approach in Task 2.2 step 3, and rewrote SC8 with a concrete test reference. These are resolved.
- The `bold()` function becomes dead code after this change (only call site was `bold(&task, no_color)`). Since it is `pub` in a library crate, clippy will not warn. Acceptable to leave, but could be noted as a follow-up cleanup.
- The ARCHITECTURE.md and CONVENTIONS.md in `.planning/codebase/` still reference `bridge.rs` and `todos.rs` as active modules. These become further out of date after this change. Not actionable within this plan.
- Task 3.1 step 1 renames `build_statusline_without_task_has_two_segments` and changes its assertion from 2 to 3 segments. This is correct -- the test input includes `used_percentage: Some(10.0)` which produces a non-empty context_bar, so the new layout `model | dir | context_bar` yields 3 segments when split on SEPARATOR.
- The plan correctly preserves `session_id` on `StatusInput` per research guidance and CONCERNS.md principles about additive schema evolution.

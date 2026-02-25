# Plan Critique

> Task: statusline-git-branch
> Reviewed: 2026-02-25T18:31:20+00:00
> Verdict: has objections

## Previous Objections Assessment

### Objection 1 (Wave 1 dependency violation between Task 1.3 and Task 1.2): RESOLVED
The revised plan merged module registration into Task 1.2. Task 1.3 no longer exists. The dependency is eliminated.

### Objection 2 (Test assertions assume hardcoded path exists on disk): RESOLVED
All branch-present tests now use `tempfile::tempdir()` with synthetic `.git/HEAD` files. The existing `full_input_has_separator_between_dir_and_context_bar` integration test now uses a flexible assertion (`segments.len() == 3 || segments.len() == 4`) and checks the last segment for bar graph characters. The `build_statusline_with_full_input_contains_model_directory_and_bar` unit test no longer adds segment count assertions.

### Objection 3 (No deterministic unit test for 4-segment case in format.rs): RESOLVED
Task 2.1 now includes four new unit tests using `tempfile::tempdir()`: two for branch-present cases (with/without context, asserting 4 and 3 segments respectively) and two for branch-absent cases (with/without context, asserting 3 and 2 segments). All are deterministic.

## Objections

### Objection 1: Task 1.2 depends on Task 1.1 but both are in Wave 1
- **Category:** internal-consistency
- **Severity:** medium
- **Affected tasks:** Task 1.1, Task 1.2
- **Evidence:** Task 1.2 creates `src/git_branch.rs` with unit tests that use `tempfile::tempdir()` (line 65-73 of the plan). The verification command is `cargo test --lib git_branch`. Task 1.1 adds `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml`. Both tasks are in Wave 1, which permits parallel execution.
- **Problem:** If an executor runs Task 1.2 before Task 1.1 completes (parallel execution within a wave), `cargo test --lib git_branch` will fail to compile because `tempfile` is not yet in `Cargo.toml`. The executor gets stuck on Task 1.2 verification.
- **Suggestion:** Merge Task 1.1 into Task 1.2. Adding a single line to `Cargo.toml` is trivially part of creating the module. Task 1.2's action already describes everything needed -- just prepend "Add `tempfile = \"3\"` to the `[dev-dependencies]` section of `Cargo.toml` after the existing `predicates = \"3\"` line." to the action, add `Cargo.toml` to the files affected list, and remove Task 1.1 entirely. This eliminates the dependency with no other consequences since no other task touches `Cargo.toml`.

## Observations

- The previous critique's observation about long branch name truncation remains valid and is acceptable to defer.
- The plan's line number references to `src/format.rs` are accurate against the current source (verified lines 47, 50, 66-85, 169, 202). This is good for executor precision.
- The worktree test (`follows_gitdir_file_for_worktrees`) uses an absolute path for the `gitdir:` value. Real worktrees sometimes use relative paths. The plan's implementation description (Task 1.2, step 2) correctly says to resolve relative paths, but the test only covers the absolute path case. This is not an objection -- the absolute path case covers the common scenario and the relative path handling code is simple -- but a relative-path worktree test would strengthen coverage in future work.

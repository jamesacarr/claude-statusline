---
task_id: statusline-git-branch
title: Show current git branch in statusline
status: planning
created: 2026-02-25T18:21:21+00:00
updated: 2026-02-25T18:28:42+00:00
current_wave: null
current_task: null
pause_reason: null
---

# Show current git branch in statusline

## Goal
The statusline displays the current git branch name between the directory and context usage bar segments, separated by the existing box-drawing separator. When the working directory is not inside a git repository or HEAD is detached, the branch segment is omitted and the output is identical to the current behaviour. The implementation reads `.git/HEAD` directly using `std::fs` with zero new runtime dependencies.

## Success Criteria
1. Running the binary with `current_dir` pointing to a git repo directory produces output in the format `model | dir | branch | context_bar` (4 segments when branch and context are present)
2. Running the binary with `current_dir` pointing to `/tmp` (not a git repo) produces the same output as before: `model | dir | context_bar` (3 segments) or `model | dir` (2 segments)
3. The branch segment shows the bare branch name (e.g., `main`, `feature/my-branch`) with no icon or prefix
4. The branch name is styled with `dim()` matching the directory segment's visual treatment
5. Detached HEAD (raw SHA in `.git/HEAD`) causes the branch segment to be omitted
6. `.git` as a file (worktrees/submodules with `gitdir:` indirection) is handled -- the function follows the pointer to read `HEAD` from the correct location
7. `cargo test` passes with all existing and new tests
8. `cargo clippy -- -D warnings` produces zero warnings
9. `cargo fmt --check` produces no formatting drift
10. No new runtime dependencies added to `Cargo.toml` `[dependencies]`

## Non-Functional Requirements
1. **Performance:** Branch detection must complete in under 1ms. Reading `.git/HEAD` is a single filesystem syscall on a ~40-byte file, well within budget. The statusline runs on every prompt render; process spawn overhead (5-20ms) is avoided by reading the file directly.
2. **Security:** The path to `.git/HEAD` is constructed by walking up from the directory already present in the JSON input. No user-supplied branch names are used in path construction. Branch names cannot contain ANSI control characters per git-check-ref-format, but the existing `dim()` helper wraps and resets ANSI codes safely regardless.
3. **Reliability:** All filesystem errors (missing `.git`, permission denied, non-UTF-8 content, non-existent directory) result in `None` return and silent omission of the branch segment. The statusline never fails due to git state.

## Wave 1: Create git branch module
Status: pending

### Task 1.1: Add tempfile dev-dependency to Cargo.toml
- **Status:** pending
- **Files affected:** `Cargo.toml`
- **Action:** Add `tempfile = "3"` to the `[dev-dependencies]` section of `Cargo.toml`, after the existing `predicates = "3"` line. This is needed for unit tests that create temporary directories with synthetic `.git/HEAD` files. Do not add any runtime dependencies.
- **Verification:** `cargo check --tests`
- **Done when:** `cargo check --tests` succeeds and `Cargo.toml` `[dev-dependencies]` contains `tempfile = "3"` while `[dependencies]` remains unchanged (only `serde` and `serde_json`).
- **Retries:** 0
- **Last failure:** null

### Task 1.2: Create src/git_branch.rs module and register in src/lib.rs
- **Status:** pending
- **Files affected:** `src/git_branch.rs`, `src/lib.rs`
- **Action:** Create a new file `src/git_branch.rs` following the single-responsibility module pattern (like `src/path_format.rs` and `src/context.rs`). Then register the module in `src/lib.rs`.

  **Part A -- Create `src/git_branch.rs`:**

  1. `pub fn get_branch(dir: &str) -> Option<String>` -- public entry point. Converts `dir` to a `Path` and delegates to `get_branch_from`. Returns `None` if `dir` is empty.

  2. `pub(crate) fn get_branch_from(dir: &Path) -> Option<String>` -- testable variant accepting an injected path. This function:
     - Walks up the directory tree from `dir` looking for a `.git` entry at each level (check `dir/.git`, then `dir/../.git`, etc., stopping at the filesystem root).
     - If `.git` is a **directory**, reads `<dir>/.git/HEAD`.
     - If `.git` is a **file** (worktree/submodule), reads its content, extracts the path after `gitdir: `, trims whitespace, resolves the path (relative paths are relative to the directory containing the `.git` file), then reads `HEAD` from that resolved git directory.
     - Parses the HEAD content: if it starts with `ref: refs/heads/`, strips that prefix and returns the remainder (trimmed) as `Some(branch_name)`. This handles branch names with slashes like `feature/foo/bar`.
     - If HEAD content does not start with `ref: refs/heads/` (detached HEAD -- raw SHA, or `ref:` pointing outside `refs/heads/`), returns `None`.
     - On any error (`Err` from `fs::read_to_string`, `fs::metadata`, missing prefix, empty content), returns `None` immediately. Never panics.

  3. Add a `/// ` doc comment on both public and `pub(crate)` functions explaining purpose and return semantics.

  4. Add a `#[cfg(test)] mod tests` block at the bottom with the following unit tests using `tempfile::tempdir()`:
     - `returns_branch_name_from_standard_git_head` -- create tempdir, mkdir `.git`, write `.git/HEAD` with `ref: refs/heads/main\n`, assert returns `Some("main")`
     - `returns_branch_name_with_slashes` -- write `.git/HEAD` with `ref: refs/heads/feature/my-branch\n`, assert returns `Some("feature/my-branch")`
     - `returns_none_for_detached_head` -- write `.git/HEAD` with a 40-char hex SHA, assert returns `None`
     - `returns_none_when_no_git_directory` -- create tempdir with no `.git`, assert returns `None`
     - `returns_none_for_empty_head_file` -- write `.git/HEAD` with empty string, assert returns `None`
     - `returns_none_for_empty_dir_string` -- call `get_branch("")`, assert returns `None`
     - `finds_git_dir_in_parent_directory` -- create tempdir, mkdir `sub`, write `.git/HEAD` at root level, call `get_branch_from` with `sub` path, assert returns the branch name
     - `follows_gitdir_file_for_worktrees` -- create tempdir, create a separate dir with `HEAD` file containing `ref: refs/heads/wt-branch\n`, write `.git` as a file containing `gitdir: <path-to-separate-dir>\n`, assert returns `Some("wt-branch")`

  Follow test naming convention: descriptive `snake_case` sentences.

  **Part B -- Register module in `src/lib.rs`:**

  Add `pub mod git_branch;` to `src/lib.rs`. Place it alphabetically among the existing module declarations (after `pub mod format;` and before `pub mod path_format;`). The file should read:
  ```
  pub mod context;
  pub mod format;
  pub mod git_branch;
  pub mod path_format;
  pub mod types;
  ```

- **Verification:** `cargo test --lib git_branch`
- **Done when:** All 8 unit tests pass. `cargo check` succeeds (module registered correctly). `cargo clippy -- -D warnings` reports no warnings for the new module.
- **Retries:** 0
- **Last failure:** null

## Wave 2: Integrate branch into statusline assembly
Status: pending

### Task 2.1: Modify build_statusline in src/format.rs to include git branch segment
- **Status:** pending
- **Files affected:** `src/format.rs`
- **Action:** Modify `src/format.rs` to integrate the git branch segment. Changes:

  1. Add `use crate::git_branch;` to the imports at the top (after `use crate::context;`, before `use crate::path_format;`).

  2. In `build_statusline()`, after line 47 (`let formatted_dir = ...`) and before line 50 (`let (remaining_pct, used_pct) = ...`), add:
     ```rust
     let branch = git_branch::get_branch(directory);
     ```
     Note: pass the raw `directory` string (before `format_directory` truncates it) so the filesystem walk uses the actual path.

  3. Replace the assembly block (lines 66-85, from `let model_segment = ...` to the closing brace) with a Vec-based segment assembly:
     ```rust
     let model_segment = dim(model_name, no_color);

     let mut segments = vec![model_segment, dim(&formatted_dir, no_color)];

     if let Some(ref branch_name) = branch {
         segments.push(dim(branch_name, no_color));
     }

     if !context_bar.is_empty() {
         segments.push(context_bar.to_string());
     }

     segments.join(SEPARATOR)
     ```
     This replaces the if/else conditional with a linear segment collector that naturally handles all 4 states (branch x context_bar presence).

  4. The existing unit test `build_statusline_with_context_has_three_segments` (line 169) uses `current_dir: "/tmp"` which is not a git repo, so `get_branch("/tmp")` returns `None`. The segment count remains 3. **No change needed to the assertion.** Update the test comment to clarify: `// With context, no git branch: model | dir | context_bar = 3 segments`.

  5. The existing unit test `build_statusline_without_context_has_two_segments` (line 202) uses `/tmp` which has no `.git`. **No change to assertion.** Update comment: `// Without context, no git branch: model | dir = 2 segments`.

  6. The existing unit test `build_statusline_with_full_input_contains_model_directory_and_bar` (line 123) uses `current_dir` pointing to the project directory (`/Users/jamescarr/Git/jamesacarr/claude-statusline`). This path may or may not exist on different machines (does not exist on CI runners) and the branch name varies. **Do not add segment count assertions to this test.** The existing `contains` assertions will continue to pass regardless of whether a branch segment is present. No changes needed.

  7. Add a new unit test `build_statusline_without_branch_and_with_context_has_three_segments` that explicitly tests the no-branch case with context:
     ```rust
     #[test]
     fn build_statusline_without_branch_and_with_context_has_three_segments() {
         let input = StatusInput {
             model: Some(ModelInfo {
                 display_name: Some("Opus".to_string()),
                 ..Default::default()
             }),
             workspace: Some(WorkspaceInfo {
                 current_dir: Some("/tmp".to_string()),
                 ..Default::default()
             }),
             context_window: Some(ContextWindow {
                 used_percentage: Some(10.0),
                 total_input_tokens: Some(1000),
                 total_output_tokens: Some(0),
                 ..Default::default()
             }),
             ..Default::default()
         };
         let result = super::build_statusline(&input, false);
         let separator = " \u{2502} ";
         let segment_count = result.split(separator).count();
         assert_eq!(
             segment_count, 3,
             "expected 3 segments without branch: model | dir | context_bar, got: {}",
             result
         );
     }
     ```

  8. Add a new unit test `build_statusline_without_branch_and_without_context_has_two_segments`:
     ```rust
     #[test]
     fn build_statusline_without_branch_and_without_context_has_two_segments() {
         let input = StatusInput {
             model: Some(ModelInfo {
                 display_name: Some("Opus".to_string()),
                 ..Default::default()
             }),
             workspace: Some(WorkspaceInfo {
                 current_dir: Some("/tmp".to_string()),
                 ..Default::default()
             }),
             ..Default::default()
         };
         let result = super::build_statusline(&input, false);
         let separator = " \u{2502} ";
         let segment_count = result.split(separator).count();
         assert_eq!(
             segment_count, 2,
             "expected 2 segments without branch or context: model | dir, got: {}",
             result
         );
     }
     ```

  9. Add a new unit test `build_statusline_with_branch_and_context_has_four_segments` using `tempfile::tempdir()` to deterministically test the branch-present case:
     ```rust
     #[test]
     fn build_statusline_with_branch_and_context_has_four_segments() {
         let tmp = tempfile::tempdir().unwrap();
         let git_dir = tmp.path().join(".git");
         std::fs::create_dir(&git_dir).unwrap();
         std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/test-branch\n").unwrap();

         let input = StatusInput {
             model: Some(ModelInfo {
                 display_name: Some("Opus".to_string()),
                 ..Default::default()
             }),
             workspace: Some(WorkspaceInfo {
                 current_dir: Some(tmp.path().to_string_lossy().to_string()),
                 ..Default::default()
             }),
             context_window: Some(ContextWindow {
                 used_percentage: Some(10.0),
                 total_input_tokens: Some(1000),
                 total_output_tokens: Some(0),
                 ..Default::default()
             }),
             ..Default::default()
         };
         let result = super::build_statusline(&input, false);
         let separator = " \u{2502} ";
         let segment_count = result.split(separator).count();
         assert_eq!(
             segment_count, 4,
             "expected 4 segments with branch: model | dir | branch | context_bar, got: {}",
             result
         );
         assert!(
             result.contains("test-branch"),
             "expected output to contain branch name 'test-branch', got: {}",
             result
         );
     }
     ```

  10. Add a new unit test `build_statusline_with_branch_and_without_context_has_three_segments` using `tempfile::tempdir()`:
      ```rust
      #[test]
      fn build_statusline_with_branch_and_without_context_has_three_segments() {
          let tmp = tempfile::tempdir().unwrap();
          let git_dir = tmp.path().join(".git");
          std::fs::create_dir(&git_dir).unwrap();
          std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/my-feature\n").unwrap();

          let input = StatusInput {
              model: Some(ModelInfo {
                  display_name: Some("Opus".to_string()),
                  ..Default::default()
              }),
              workspace: Some(WorkspaceInfo {
                  current_dir: Some(tmp.path().to_string_lossy().to_string()),
                  ..Default::default()
              }),
              ..Default::default()
          };
          let result = super::build_statusline(&input, false);
          let separator = " \u{2502} ";
          let segment_count = result.split(separator).count();
          assert_eq!(
              segment_count, 3,
              "expected 3 segments with branch but no context: model | dir | branch, got: {}",
              result
          );
          assert!(
              result.contains("my-feature"),
              "expected output to contain branch name 'my-feature', got: {}",
              result
          );
      }
      ```

- **Verification:** `cargo test --lib format`
- **Done when:** All existing format tests pass (including the updated comments) and the four new tests pass. `cargo clippy -- -D warnings` reports no warnings.
- **Retries:** 0
- **Last failure:** null

## Wave 3: Update integration tests
Status: pending

### Task 3.1: Update integration tests for git branch segment
- **Status:** pending
- **Files affected:** `tests/integration.rs`
- **Action:** Modify `tests/integration.rs` to account for the new branch segment. All new tests that assert branch-present behaviour must use deterministic tempdir fixtures, not the hardcoded project path.

  1. The existing test `full_input_has_separator_between_dir_and_context_bar` (line 319) uses `full_json()` which references the project directory. On CI, this path does not exist, so `get_branch` returns `None` and the segment count stays at 3. Locally, it may be a git repo and produce 4 segments. **Update the assertion from `assert_eq!(segments.len(), 3, ...)` to `assert!(segments.len() == 3 || segments.len() == 4, ...)`** with a comment explaining that the count depends on whether the binary's JSON `current_dir` path exists as a git repo on the runner. Update the bar graph assertion to check the **last** segment (`segments[segments.len() - 1]`) instead of `segments[2]`, since the context bar is always the final segment regardless of branch presence.

  2. Add a new integration test `non_git_dir_omits_branch_segment` using `/tmp` as the directory:
     ```rust
     #[test]
     fn non_git_dir_omits_branch_segment() {
         let json = r#"{
             "model": { "display_name": "Opus" },
             "workspace": { "current_dir": "/tmp" },
             "context_window": {
                 "total_input_tokens": 1000,
                 "total_output_tokens": 0,
                 "used_percentage": 10.0,
                 "remaining_percentage": 90.0
             }
         }"#;

         let output = cmd()
             .write_stdin(json)
             .output()
             .expect("command should execute");

         assert!(output.status.success());
         let stdout = String::from_utf8_lossy(&output.stdout);
         let separator = " \u{2502} ";
         let segments: Vec<&str> = stdout.trim_end().split(separator).collect();
         assert_eq!(
             segments.len(), 3,
             "expected 3 segments (model | dir | context_bar) for non-git dir, got {}: {:?}",
             segments.len(),
             stdout
         );
     }
     ```

  3. Add a new integration test `git_repo_tempdir_includes_branch_segment` that creates a tempdir with a synthetic `.git/HEAD` file and passes it as `current_dir` in the JSON input to the binary. This deterministically tests the branch-present case:
     ```rust
     #[test]
     fn git_repo_tempdir_includes_branch_segment() {
         let tmp = tempfile::tempdir().unwrap();
         let git_dir = tmp.path().join(".git");
         std::fs::create_dir(&git_dir).unwrap();
         std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/integration-test-branch\n").unwrap();

         let json = format!(
             r#"{{
                 "model": {{ "display_name": "Opus" }},
                 "workspace": {{ "current_dir": "{}" }},
                 "context_window": {{
                     "total_input_tokens": 1000,
                     "total_output_tokens": 0,
                     "used_percentage": 10.0,
                     "remaining_percentage": 90.0
                 }}
             }}"#,
             tmp.path().to_string_lossy().replace('\\', "\\\\")
         );

         let output = cmd()
             .write_stdin(json)
             .output()
             .expect("command should execute");

         assert!(output.status.success());
         let stdout = String::from_utf8_lossy(&output.stdout);
         let separator = " \u{2502} ";
         let segments: Vec<&str> = stdout.trim_end().split(separator).collect();
         assert_eq!(
             segments.len(), 4,
             "expected 4 segments (model | dir | branch | context_bar) for git repo tempdir, got {}: {:?}",
             segments.len(),
             stdout
         );
         assert!(
             stdout.contains("integration-test-branch"),
             "expected output to contain branch name 'integration-test-branch', got: {}",
             stdout
         );
     }
     ```

  4. Add a new integration test `branch_segment_respects_no_color` using a tempdir for deterministic branch-present assertion:
     ```rust
     #[test]
     fn branch_segment_respects_no_color() {
         let tmp = tempfile::tempdir().unwrap();
         let git_dir = tmp.path().join(".git");
         std::fs::create_dir(&git_dir).unwrap();
         std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/no-color-branch\n").unwrap();

         let json = format!(
             r#"{{
                 "model": {{ "display_name": "Opus" }},
                 "workspace": {{ "current_dir": "{}" }},
                 "context_window": {{
                     "total_input_tokens": 1000,
                     "total_output_tokens": 0,
                     "used_percentage": 10.0,
                     "remaining_percentage": 90.0
                 }}
             }}"#,
             tmp.path().to_string_lossy().replace('\\', "\\\\")
         );

         let output = cmd()
             .env("NO_COLOR", "1")
             .write_stdin(json)
             .output()
             .expect("command should execute");

         assert!(output.status.success());
         let stdout = String::from_utf8_lossy(&output.stdout);
         // With NO_COLOR, no ANSI escape codes should be present anywhere
         assert!(
             !stdout.contains("\x1b["),
             "should not contain ANSI escape sequences when NO_COLOR is set, got: {}",
             stdout
         );
         // Branch should be present (we created a synthetic git repo)
         let separator = " \u{2502} ";
         let segments: Vec<&str> = stdout.trim_end().split(separator).collect();
         assert_eq!(
             segments.len(), 4,
             "expected 4 segments in tempdir git repo with NO_COLOR, got {}: {:?}",
             segments.len(),
             stdout
         );
     }
     ```

- **Verification:** `cargo test --test integration`
- **Done when:** All integration tests pass, including the updated and new tests.
- **Retries:** 0
- **Last failure:** null

## Wave 4: Final validation
Status: pending

### Task 4.1: Run full CI validation suite
- **Status:** pending
- **Files affected:** (none -- read-only validation)
- **Action:** Run the complete CI validation pipeline to ensure all checks pass:
  1. `cargo fmt --check` -- verify no formatting drift
  2. `cargo clippy -- -D warnings` -- verify zero warnings
  3. `cargo test` -- verify all unit and integration tests pass
  4. `cargo build --release` -- verify the release build compiles successfully (confirms no binary size regression from new dependencies)
- **Verification:** All four commands exit with code 0
- **Done when:** `cargo fmt --check` exits 0, `cargo clippy -- -D warnings` exits 0, `cargo test` exits 0, and `cargo build --release` exits 0.
- **Retries:** 0
- **Last failure:** null

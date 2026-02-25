# Quality & Standards Research

> Task: Show the current git branch after the current directory and before the context usage bar in the claude-statusline tool. Should have a separator between the directory and branch, as well as between the branch and the context usage bar.
> Last researched: 2026-02-25T14:44:06Z

## Security

### Command Injection (if using `std::process::Command`)

The git branch name is not user-supplied in this context -- it comes from the local `.git/HEAD` file or `git rev-parse`. However, since the binary reads `cwd`/`workspace.current_dir` from untrusted JSON input (Claude Code schema), the directory used as the working directory for the git command must be validated.

**Requirements:**
1. **Do not shell out via `sh -c`** -- use `Command::new("git").arg(...)` with separated arguments, never string interpolation. This avoids [OWASP command injection](https://cheatsheetseries.owasp.org/cheatsheets/OS_Command_Injection_Defense_Cheat_Sheet.html) entirely since `std::process::Command` does not invoke a shell.
2. **Set the working directory explicitly** via `.current_dir()` using the same directory already extracted in `build_statusline()` (`src/format.rs:40-45`). Do not construct paths from the branch name output.
3. **Do not embed raw git output into ANSI strings without consideration** -- the existing codebase concern at `src/format.rs` (CONCERNS.md: "No sanitisation of ANSI in input data") already notes that model names are embedded directly. Branch names follow the same low-risk pattern (git branch names cannot contain control characters per `git-check-ref-format`), but stripping ANSI escapes from the output would be a defense-in-depth measure.

### Reading `.git/HEAD` Directly (if using file-based approach)

- The path to `.git/HEAD` must be constructed from the resolved working directory, not from any user-supplied branch name
- Symlink traversal is not a concern -- `std::fs::read_to_string` follows symlinks, but `.git/HEAD` is always within the repo root
- No credential exposure risk -- `.git/HEAD` contains only a ref pointer (e.g., `ref: refs/heads/main`), never secrets

### No New Attack Surface from JSON Input

The feature does not add new fields to `StatusInput` (`src/types.rs`). The git branch is derived from the filesystem based on the directory already present in the input. No new deserialization vectors are introduced.

## Performance

This is a statusline tool invoked on every prompt render. Latency directly impacts perceived shell responsiveness.

**Budget:** The entire binary should complete in under 10ms. Current execution is sub-millisecond for formatting; git branch detection is the primary new cost.

### Approach Performance Characteristics

| Approach | Expected Latency | Notes |
|----------|-----------------|-------|
| Read `.git/HEAD` file | <1ms | Single syscall, no process spawn. Fastest option. |
| `git rev-parse --abbrev-ref HEAD` | 5-20ms | Process spawn overhead dominates. Acceptable but noticeable at scale. |
| `git2` / `gix` crate (libgit2) | 1-5ms | No process spawn, but library init has overhead. [libgit2 is slower than git CLI for status ops](https://github.com/libgit2/libgit2/issues/4230) but branch lookup is trivial. |

**Key performance requirements:**
1. **No blocking on missing git** -- if `git` is not installed or the directory is not a git repo, the feature must return immediately (no timeout waiting for a process). The statusline must still render without the branch segment.
2. **No network I/O** -- `git rev-parse` does not trigger network ops, but ensure no flags like `--verify` with remote refs are used
3. **Binary size impact** -- the project uses aggressive release optimization (`opt-level = "z"`, `lto = "fat"`, `strip = "symbols"` in `Cargo.toml:17-22`). Adding `git2` crate would add ~2-4MB to the binary due to static libgit2 linking. Reading `.git/HEAD` or shelling out to `git` adds zero dependency weight.

### Caching

Not needed for this use case. The binary is invoked fresh each time (single-invocation CLI per `ARCHITECTURE.md`), so there is no opportunity for caching across calls.

## Accessibility

Not applicable (no UI changes). This is a CLI tool that outputs text to stdout. The existing `NO_COLOR` compliance (`src/main.rs:21`) covers the relevant accessibility concern for terminal output. The git branch segment should respect the same `no_color` flag -- dimming the branch text when color is enabled, plain text when disabled, following the existing pattern in `src/format.rs:13-18`.

## Testing Strategy

### Test types needed

- **Unit tests** in the new git module (e.g., `src/git.rs`) -- test branch name extraction logic
- **Unit tests** in `src/format.rs` -- test `build_statusline` with and without git branch data
- **Integration tests** in `tests/integration.rs` -- test full binary pipeline with git branch visible in output

### Key test cases

**Git module unit tests:**
1. Parse `.git/HEAD` content `ref: refs/heads/main` -> returns `"main"`
2. Parse `.git/HEAD` content `ref: refs/heads/feature/my-branch` -> returns `"feature/my-branch"`
3. Parse `.git/HEAD` with detached HEAD (raw SHA) -> returns abbreviated SHA or `"HEAD"`
4. Handle missing `.git/HEAD` file -> returns `None`
5. Handle empty `.git/HEAD` content -> returns `None`
6. Handle `.git/HEAD` with unexpected format -> returns `None`

**Format module unit tests (extend existing tests in `src/format.rs:88-337`):**
7. `build_statusline` with branch present and context bar -> 4 segments (model | dir | branch | context)
8. `build_statusline` with branch present but no context -> 3 segments (model | dir | branch)
9. `build_statusline` with no branch (not a git repo) and context -> 3 segments (model | dir | context) -- same as current
10. `build_statusline` with no branch and no context -> 2 segments (model | dir) -- same as current
11. Branch segment uses separator character `\u{2502}` consistently
12. Branch segment respects `no_color` flag

**Integration tests (extend `tests/integration.rs`):**
13. Full input in a git repo directory produces output containing a branch name
14. Full input in a non-git directory omits branch segment

### Mocking approach

- **For git module**: follow the existing testability pattern (`TESTING.md` prescriptive guidance). Create a `pub(crate) fn get_branch_from(dir: &Path) -> Option<String>` that accepts a directory, and a public `get_branch(dir: &str) -> Option<String>` that delegates to it. Tests can pass a tempdir with a synthetic `.git/HEAD` file.
- **For format module**: pass `None` or `Some("main")` as the branch parameter to `build_statusline` (or a new internal function that accepts branch data). No filesystem access needed in format tests.
- **For integration tests**: the test runs from within this git repo, so `workspace.current_dir` pointing to the repo root will naturally produce a branch name. For the non-git case, use `/tmp` as the directory.

### Edge cases to cover

- Detached HEAD state (e.g., during rebase or checkout of a specific commit)
- Branch names with `/` (e.g., `feature/foo/bar`)
- Branch names with special characters allowed by git (e.g., `fix-#123`, `release-v1.0`)
- Worktree-linked repos where `.git` is a file (containing `gitdir: ...`) rather than a directory
- Bare repositories (no working tree)
- Submodules with their own `.git` structure
- Very long branch names (should not break layout -- consider truncation)

### Existing test patterns

- Unit test naming: `verb_noun_condition` -- e.g., `returns_none_for_empty_session_id` (`src/context.rs`)
- Unit test structure: `#[cfg(test)] mod tests` at bottom of each file (`src/format.rs:88`, `src/context.rs:84`, `src/path_format.rs:30`)
- Integration tests: `cmd()` helper + fluent `assert_cmd` API (`tests/integration.rs:6-8`)
- Struct construction: `..Default::default()` for partial inputs (`src/format.rs:124-143`)
- Tempdir fixtures for filesystem tests: `tempfile::tempdir()` (referenced in `TESTING.md`)

## Standards Checklist

1. Git branch detection must not panic -- all errors return `None` and the statusline renders without the branch segment
2. Branch segment must use the existing `SEPARATOR` constant (`src/format.rs:10`) between directory and branch, and between branch and context bar
3. Branch text must be wrapped in `dim()` when color is enabled, matching the directory segment style (`src/format.rs:74,81`)
4. `no_color` flag must suppress all ANSI codes in the branch segment, consistent with existing `dim()`/`bold()` behaviour (`src/format.rs:13-28`)
5. New git module must follow single-responsibility pattern -- one file, one concern (`ARCHITECTURE.md` prescriptive guidance)
6. New module must be registered in `src/lib.rs` as `pub mod git;`
7. Must not add heavyweight dependencies (`git2`, `gix`) -- prefer `.git/HEAD` file read or `std::process::Command` to keep binary size minimal per release profile goals (`Cargo.toml:17-22`)
8. Process spawn (if used) must not inherit stdin -- use `.stdin(Stdio::null())` to prevent hanging
9. Process spawn (if used) must set a short timeout or use `.output()` (which waits for completion) -- do not use `.spawn()` without consuming the child
10. Non-git directories must produce the same output as the current implementation -- no regressions in segment count or separator placement
11. Existing unit tests in `src/format.rs` that assert segment counts (lines 169-226) must be updated to account for the new branch segment
12. CI pipeline (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`) must pass -- `.github/workflows/ci.yml:33-37`
13. All new code must pass `cargo clippy` with `-D warnings` (zero warnings policy per CI)
14. Branch segment must gracefully handle the case where the working directory from JSON input does not exist on disk

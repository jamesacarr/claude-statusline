# Risks & Edge Cases Research

> Task: Show the current git branch after the current directory and before the context usage bar in the claude-statusline tool. Should have a separator between the directory and branch, as well as between the branch and the context usage bar.
> Last researched: 2026-02-25T15:10:00Z

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Detached HEAD state produces confusing or empty output | high | low | Return `None` from `get_branch()` when `.git/HEAD` contains a raw SHA instead of `ref: refs/heads/...`. The segment is simply omitted. This occurs during rebase, bisect, cherry-pick, and direct commit checkout. |
| `.git` is a file (worktree/submodule), not a directory | medium | medium | When walking up the directory tree, check if `.git` is a file. If so, read its `gitdir: <path>` line and follow the indirection to the actual git directory, then read `HEAD` from there. Worktrees store HEAD at `<gitdir>/worktrees/<name>/HEAD`. Submodules store it at `<superproject>/.git/modules/<name>/HEAD`. |
| Directory from JSON input does not exist on disk | medium | low | `std::fs::read_to_string` on a non-existent path returns `Err`. The function returns `None`, omitting the branch segment. No panic, no user-visible error. Matches existing pattern (`CONCERNS.md`: "treat failure as None/skip"). |
| Existing segment-count tests break | high | high | Tests at `src/format.rs:169` (`build_statusline_with_context_has_three_segments`) and `src/format.rs:202` (`build_statusline_without_context_has_two_segments`) assert exact segment counts. Integration test at `tests/integration.rs:319` asserts 3 segments. All will break if the git branch is present. Must update these tests or restructure `build_statusline` to accept an injected branch value for testability. |
| ANSI colour bleed from branch segment | medium | medium | Per `CONCERNS.md` fragile area: any new ANSI segment must end with `\x1b[0m`. Using the existing `dim()` helper (`src/format.rs:13`) handles this automatically. Do not manually construct ANSI sequences for the branch text. |
| `format.rs` assembly logic becomes unwieldy | medium | low | Currently two cases (with/without context bar) at `src/format.rs:69-85`. Adding branch creates 4 combinations (branch x context). Codebase integration research recommends refactoring to `Vec<String>` segments joined by `SEPARATOR`. This is a minor refactor risk but keeps the code maintainable. |
| `$GIT_DIR` environment variable overrides repo location | low | low | Users can set `$GIT_DIR` to point to a git directory elsewhere. The `.git/HEAD` file approach ignores this env var. Acceptable trade-off: `$GIT_DIR` is rare in interactive use and the fallback (omitting branch) is harmless. |
| Bare repository has no working tree | low | low | Bare repos have `HEAD` at the repo root (no `.git` subdirectory). The directory walk looking for `.git` will not find one, so `get_branch()` returns `None`. Correct behaviour -- bare repos are not interactive working directories. |
| Very long branch names overflow statusline width | low | low | Git allows branch names up to 255 bytes. Names like `feature/JIRA-12345/implement-very-long-description-of-feature` could push the statusline past terminal width. Consider truncating branch display at ~30 characters with ellipsis. Not critical for v1 -- the statusline already has no width management for long directory paths or model names. |
| Process spawn overhead (if CLI approach chosen) | medium | medium | `git rev-parse --abbrev-ref HEAD` takes 5-20ms per invocation. The statusline runs after every assistant message. Over a session this adds noticeable latency. The recommended approach (read `.git/HEAD`) avoids this entirely at <1ms. |
| Symlinked `.git` directory | low | low | Some setups symlink `.git` to an external location. `std::fs::read_to_string` follows symlinks by default, so this is handled transparently. |

## Edge Cases

- **Detached HEAD (raw SHA in `.git/HEAD`)** -- expected behaviour: `get_branch()` returns `None`, branch segment omitted. Occurs during `git rebase`, `git bisect`, `git cherry-pick`, or `git checkout <sha>`. The `.git/HEAD` file contains a 40-character hex SHA instead of `ref: refs/heads/<branch>`.
- **Branch name with slashes** (e.g., `feature/my-branch`, `fix/area/thing`) -- expected behaviour: return the full ref name after `refs/heads/`. The `/` characters are valid in branch names per [git-check-ref-format](https://git-scm.com/docs/git-check-ref-format).
- **Branch name with special characters** (e.g., `fix-#123`, `release-v1.0`, `user@work`) -- expected behaviour: display as-is. Git branch names cannot contain ASCII control characters, spaces, `~`, `^`, `:`, `?`, `*`, `[`, `\`, or `..`. All allowed characters are safe for terminal display.
- **Empty `.git/HEAD` file** -- expected behaviour: `get_branch()` returns `None`. Could occur if the file is corrupted or being written to concurrently.
- **`.git/HEAD` with trailing newline/whitespace** -- expected behaviour: trim before parsing. The file always has a trailing newline (`ref: refs/heads/main\n`). Use `.trim()`.
- **`.git/HEAD` with unexpected format** (neither `ref:` prefix nor valid SHA) -- expected behaviour: return `None`. Do not panic on unexpected content.
- **Nested git repos** (repo inside repo) -- expected behaviour: the directory walk finds the innermost `.git` first (walking up from `workspace.current_dir`). This is correct -- it matches what `git` itself does.
- **Non-UTF-8 `.git/HEAD` content** -- expected behaviour: `std::fs::read_to_string` returns `Err` for non-UTF-8 content. The function returns `None`. Branch names are always ASCII per git-check-ref-format rules, so this is a corruption case.
- **Race condition: branch changes between statusline invocations** -- expected behaviour: display whatever `.git/HEAD` says at read time. The statusline is a snapshot, not a live view. No consistency guarantees needed.
- **Directory is `/` (root)** -- expected behaviour: walk up finds no `.git`, return `None`. No branch segment.
- **`.git` is a file (worktree)** -- file contains `gitdir: /path/to/main/.git/worktrees/<name>`. Must read `HEAD` from the referenced path. For linked worktrees, HEAD is at `<gitdir>/HEAD` (the worktree-specific path), not the main repo HEAD.
- **`.git` is a file (submodule)** -- file contains `gitdir: ../.git/modules/<name>`. Same resolution: follow the path and read `HEAD` from there. Submodules are typically in detached HEAD, so `get_branch()` will usually return `None` -- correct behaviour.
- **Permission denied reading `.git/HEAD`** -- expected behaviour: `std::fs::read_to_string` returns `Err`, function returns `None`. Branch segment omitted silently.
- **`workspace.current_dir` and `cwd` are both `None`** -- expected behaviour: `directory` is already `""` in this case (`src/format.rs:45`). `get_branch("")` should return `None` immediately (no path to search).

## Backward Compatibility

No breaking changes to the external interface:

1. **Stdin JSON schema unchanged** -- no new fields added to `StatusInput` (`src/types.rs`). Git branch is derived from the filesystem, not from input data.
2. **Stdout format change is additive** -- existing output gains a new optional segment. When not in a git repo, output is identical to current behaviour.
3. **Exit code unchanged** -- binary always exits 0 (`src/main.rs:4`, `unwrap_or_default()`).
4. **NO_COLOR behaviour preserved** -- branch segment uses `dim()` which respects the flag.

**Caveat for downstream consumers:** Any tooling that parses the statusline output by counting separator-delimited segments will see a different count when a git branch is present. The segment structure changes from `model | dir [| context]` to `model | dir [| branch] [| context]`. This is unlikely to affect real-world usage since Claude Code renders the string as-is.

## Fragile Areas

- **`src/format.rs:69-85` (segment assembly)** -- The current `if/else` for context bar presence is the exact code being modified. Adding branch creates a 4-way conditional. A refactor to segment-vector approach is safer but must preserve the existing behaviour for all current test cases. The ANSI reset concern from `CONCERNS.md` (fragile area: "missing reset could bleed colour") applies here -- each segment must be independently reset.

- **`src/format.rs:169-226` (segment count tests)** -- Three unit tests assert exact segment counts (2 or 3). These tests construct `StatusInput` with paths like `/tmp` that do not have a `.git` directory, so they would not gain a branch segment -- unless the test runner's CWD leaks into the function. The current code passes the `directory` string from JSON, not the process CWD, so these tests are safe *if* `get_branch()` receives the same directory string and `/tmp` has no `.git`. However, `tests/integration.rs:319` runs from the project root (a git repo) and asserts exactly 3 segments -- this will break and must be updated to expect 4.

- **Architecture docs reference non-existent modules** -- `ARCHITECTURE.md` references `src/todos.rs` and `src/bridge.rs` which do not exist on disk. The architecture doc appears to describe a planned or removed state. New code should follow the patterns described there but should not assume those modules exist.

## Unknowns

1. **Worktree HEAD resolution path** -- The exact file layout for worktree-linked repos (`<gitdir>/worktrees/<name>/HEAD`) needs verification with an actual worktree setup. The approach doc recommends deferring worktree support if it proves complex, falling back to the git CLI approach if needed.

2. **CI environment git state** -- GitHub Actions `actions/checkout@v4` creates a shallow clone. The `.git/HEAD` file exists and contains `ref: refs/heads/<branch>` for PR builds, but the exact branch name may differ from expectations (e.g., `refs/pull/N/merge`). Integration tests that assert a specific branch name in CI could be flaky. Tests should assert branch *presence* (non-empty, no-slash-prefix) rather than a specific value.

3. **Claude Code invocation context** -- It is unclear whether `workspace.current_dir` from Claude Code always points to a directory that exists on the machine running the statusline binary. In normal usage (local CLI), it does. In remote or container scenarios, the path might not exist locally. The function must handle this gracefully (return `None`).

4. **`ARCHITECTURE.md` accuracy** -- The architecture doc references modules (`todos.rs`, `bridge.rs`) that do not exist. The `format.rs` module dependency list in the doc includes these non-existent modules. The Planner should verify the architecture doc is up to date before relying on it for integration decisions.

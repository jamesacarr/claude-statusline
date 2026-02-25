# Approach Research

> Task: Show the current git branch after the current directory and before the context usage bar in the claude-statusline tool. Should have a separator between the directory and branch, as well as between the branch and the context usage bar.
> Last researched: 2026-02-25T00:00:00Z

## Context

The Claude Code statusline JSON schema does **not** include git branch data ([confirmed via official docs](https://code.claude.com/docs/en/statusline)). The tool must resolve the branch name itself using the directory path already available in the input (`workspace.current_dir` or `cwd`).

Current statusline layout in `src/format.rs:69-85`:
```
model | directory | context_bar
```

Target layout:
```
model | directory | branch | context_bar
```

When there is no git branch (not a repo, detached HEAD with no name, or error), the segment should be omitted and the layout should fall back to the current behaviour.

## Viable Approaches

### 1. Read `.git/HEAD` Directly (std::fs only)

- **What:** Parse the `.git/HEAD` file from the filesystem to extract the branch name. Walk up from `workspace.current_dir` looking for a `.git` directory, then read and parse the `HEAD` file.
- **How:** New module `src/git_branch.rs` with a function like `get_branch(dir: &str) -> Option<String>`. Reads `<repo>/.git/HEAD`, strips the `ref: refs/heads/` prefix, trims whitespace. Handles detached HEAD (raw SHA) by returning a short hash or `None`. Called from `format::build_statusline()`.
- **Pros:**
  - Zero new dependencies -- keeps Cargo.toml unchanged
  - Minimal binary size impact (a few hundred bytes of code)
  - No C library compilation (no libgit2/openssl build chain)
  - Fast: single file read, no process spawn
  - Follows project pattern of minimal dependencies (`serde` + `serde_json` only)
  - Follows testability pattern: expose `get_branch_from(base: &Path)` for tests
- **Cons:**
  - Must handle `.git` file (not directory) for worktrees and submodules -- the file contains `gitdir: <path>` pointing elsewhere
  - Must walk up directory tree to discover repo root (not just check `dir/.git`)
  - Does not handle exotic git configurations (e.g., `$GIT_DIR` env override, bare repos)
  - Slightly fragile if git internals change format (unlikely -- format stable since git 1.0)
- **Best when:** Binary size and dependency count are priorities, and only branch name display is needed (no other git features planned)
- **Sources:** [Git Internals - Git References](https://git-scm.com/book/en/v2/Git-Internals-Git-References), [Baeldung - git HEAD format](https://www.baeldung.com/ops/git-current-branch-name)

### 2. Shell Out to `git` CLI via `std::process::Command`

- **What:** Spawn `git branch --show-current` (or `git rev-parse --abbrev-ref HEAD`) as a child process to get the branch name.
- **How:** New module `src/git_branch.rs`. Use `std::process::Command::new("git").args(["branch", "--show-current"]).current_dir(dir).output()`. Parse stdout, trim, return `Option<String>`. Called from `format::build_statusline()`.
- **Pros:**
  - Zero new dependencies
  - Handles all git edge cases correctly (worktrees, submodules, bare repos, `$GIT_DIR`)
  - `git branch --show-current` returns empty string for detached HEAD -- clean handling
  - Leverages user's installed git, which already works for their repos
  - Trivial implementation (< 15 lines)
- **Cons:**
  - Process spawn overhead: ~2-5ms per invocation (fork + exec + wait). Statusline runs after each assistant message, so this is acceptable but measurable
  - Requires `git` to be in `$PATH` -- fails silently if not installed (acceptable: if no git, no branch to show)
  - Harder to unit test without actual git repos or mocking
  - `git branch --show-current` requires Git 2.22+ (June 2019); can fall back to `rev-parse --abbrev-ref HEAD` for older versions
- **Best when:** Correctness across all git configurations is more important than avoiding process spawn. This is the approach used by all shell-based statusline examples in the [official Claude Code docs](https://code.claude.com/docs/en/statusline)
- **Sources:** [Claude Code statusline docs - git examples](https://code.claude.com/docs/en/statusline), [git-branch docs](https://git-scm.com/docs/git-branch)

### 3. Use `git2` Crate (libgit2 bindings)

- **What:** Add the `git2` crate as a dependency and use its Rust API to open the repository and read HEAD.
- **How:** `git2::Repository::discover(dir)` to find repo, then `repo.head()` to get reference, extract branch name from shorthand. New module delegates to `git2` API.
- **Pros:**
  - Rich, well-tested API for all git operations
  - No process spawn -- in-process library call
  - Handles worktrees, submodules, bare repos via `discover()`
  - Would enable future git features (dirty status, ahead/behind counts) without shelling out
- **Cons:**
  - Significant dependency: `git2` pulls in `libgit2-sys` (C compilation), `openssl-sys` on some platforms
  - Binary size increase: ~100-150 KiB of text section from libgit2 alone ([source](https://github.com/johnthagen/min-sized-rust))
  - Compile time increase: libgit2 C compilation adds 10-30s to clean builds
  - Overkill for reading a single ref -- using < 0.1% of the library's surface area
  - Conflicts with project's minimal dependency philosophy (currently only `serde` + `serde_json`)
  - Cross-compilation complexity: libgit2 C build requires platform toolchains
- **Best when:** Multiple git features are planned (dirty indicators, ahead/behind, stash count) and the dependency cost is justified
- **Sources:** [git2 docs](https://docs.rs/git2), [git2-rs GitHub](https://github.com/rust-lang/git2-rs)

## Recommendation

**Approach 1 (Read `.git/HEAD` directly)** is the best fit for this project.

Rationale:
1. **Consistency with project philosophy.** The codebase has exactly two dependencies (`serde`, `serde_json`). Adding `git2` with its C compilation chain for a single file read is disproportionate. Reading `.git/HEAD` adds zero dependencies.
2. **Binary size.** The project's `Cargo.toml` is aggressively optimised for minimal binary size (`opt-level = "z"`, `lto = "fat"`, `strip = "symbols"`). Adding libgit2 would undermine this.
3. **Performance.** A single `fs::read_to_string` on a ~40-byte file is faster than spawning a process (~2-5ms) or initialising libgit2. The statusline runs frequently.
4. **Testability.** Follows the existing `_from(base: &Path)` pattern (see `src/bridge.rs:17`, architecture doc). Tests can create temp dirs with `.git/HEAD` files.
5. **Sufficient for scope.** The task only needs the branch name. The `.git/HEAD` format (`ref: refs/heads/<branch>`) has been stable for 20+ years.

The main gap -- worktree/submodule support where `.git` is a file, not a directory -- is addressable with a small check: if `.git` is a file, read its `gitdir:` line and follow the indirection. The directory-walk to find the repo root is straightforward (walk up checking for `.git` at each level, like `git2::Repository::discover` does).

If Approach 1 proves insufficient (e.g., worktree edge cases become problematic), **Approach 2** is the natural fallback -- it handles everything correctly with minimal code, at the cost of a process spawn.

## Open Questions

1. **Detached HEAD display:** When HEAD points directly at a SHA (not a branch), should the segment show a short hash (e.g., `a1b2c3d`), a fixed label like `(detached)`, or be omitted entirely? Recommend: omit the segment (return `None`), keeping the statusline clean.
2. **Git icon/prefix:** Should the branch name have a visual prefix (e.g., a git branch unicode symbol like `\u{E0A0}` from powerline fonts, or plain text like `git:`)? The official Claude Code docs examples use emoji `\u{1F33F}`, but this project currently avoids emoji. Recommend: display bare branch name with no icon, matching the project's minimal aesthetic.
3. **Worktree/submodule priority:** How important is worktree and submodule support? If low priority, Approach 1 can start with the simple case (`.git` is a directory) and add file-based indirection later.

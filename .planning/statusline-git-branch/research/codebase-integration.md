# Codebase Integration Research

> Task: Show the current git branch after the current directory and before the context usage bar in the claude-statusline tool. Should have a separator between the directory and branch, as well as between the branch and the context usage bar.
> Last researched: 2026-02-25T19:45:00Z

## Affected Code

| File/Module | Role | Change Type |
|------------|------|-------------|
| `src/git_branch.rs` | New module -- discovers git repo root and reads branch name from `.git/HEAD` | create |
| `src/lib.rs` | Crate root -- re-exports modules | modify (add `pub mod git_branch;`) |
| `src/format.rs` | Statusline assembler -- insert branch segment between directory and context bar | modify |
| `tests/integration.rs` | End-to-end binary tests | modify (update segment count assertions, add branch-specific tests) |

## Entry Points

The new code hooks into a single location: `format::build_statusline()` at `src/format.rs:31`. This function is the sole orchestration point for assembling the statusline. The new `git_branch` module will be called between directory formatting (line 47) and context computation (line 50), following the same call-then-format pattern used by other modules.

Call site insertion point in `src/format.rs`:
```
Line 47: let formatted_dir = path_format::format_directory(directory);
          // NEW: let branch = git_branch::get_branch(directory);
Line 50: let (remaining_pct, used_pct) = ...
```

The assembly block (lines 69-85) changes from handling 2 states (with/without context bar) to 4 states (branch x context bar presence).

## Existing Patterns to Follow

- **Single-responsibility modules** -- each `src/*.rs` file has one narrow job. Git branch detection belongs in its own module, not inline in `format.rs`. Pattern: `src/path_format.rs`, `src/context.rs`.

- **Module registration** -- add `pub mod git_branch;` to `src/lib.rs:1-4`. Flat-file convention, one module per line.

- **Testability via `_from()` variant** -- modules that touch the filesystem expose `pub(crate) fn <name>_from(base: &Path, ...)` alongside the public function. The public function resolves real paths; `_from` accepts injected paths for testing. Prescribed in `ARCHITECTURE.md:78`. Apply: `pub fn get_branch(dir: &str) -> Option<String>` wraps `pub(crate) fn get_branch_from(dir: &Path) -> Option<String>`.

- **Best-effort, never fail** -- all side-effect modules treat errors as `None`/skip. If `.git/HEAD` is missing, unreadable, or unparseable, return `None` and omit the segment. Prescribed in `CONCERNS.md:46`: "never propagate I/O errors to stdout".

- **ANSI styling** -- the directory segment uses `dim()` (`src/format.rs:74,81`). The branch name should also use `dim()` for visual consistency. Both `dim()` and `bold()` at `src/format.rs:13-28` accept a `no_color` flag and handle reset codes.

- **ANSI reset discipline** -- per `CONCERNS.md:47`, new ANSI colour segments must end with `\x1b[0m`. The existing `dim()` helper handles this already.

- **Separator constant** -- `SEPARATOR` at `src/format.rs:10` (`" \u{2502} "`) is the box-drawing vertical line with spaces. Reuse for both new separator positions (dir-to-branch, branch-to-context_bar).

- **Test naming** -- descriptive `snake_case` sentences. Examples: `src/format.rs:95` (`dim_wraps_text_in_dim_ansi_codes`), `src/path_format.rs:35` (`truncates_five_component_path_to_last_three_with_ellipsis_prefix`).

- **Integration test style** -- use `cmd()` helper with `assert_cmd` fluent API and `predicates`. See `tests/integration.rs:6-8`.

## Shared Code to Reuse

- `format::SEPARATOR` at `src/format.rs:10` -- box-drawing separator between all segments
- `format::dim()` at `src/format.rs:13-19` -- wrap branch name in dim ANSI codes, matching directory styling
- `format::bold()` at `src/format.rs:22-28` -- available if a distinct branch style is desired

Note: the raw `directory` string (before `path_format::format_directory()` truncates it) is what `git_branch::get_branch()` needs as input, since it must traverse the actual filesystem path to find `.git/HEAD`.

## Dependencies

**No new crate dependencies required.** The recommended approach (read `.git/HEAD` directly) uses only `std::fs` and `std::path` from the standard library.

`Cargo.toml` remains unchanged. This aligns with the project's minimal dependency philosophy (currently `serde` + `serde_json` only) and aggressive binary size optimization (`opt-level = "z"`, `lto = "fat"`, `strip = "symbols"` at `Cargo.toml:17-22`).

## Data Flow

### Before (current)

```
stdin JSON
  -> main::run()                                     [src/main.rs:8]
       -> serde_json::from_str -> StatusInput        [src/types.rs:6]
            -> format::build_statusline()            [src/format.rs:31]
                  |- path_format::format_directory()  -> formatted_dir
                  |- context::compute_usage()         -> usage
                  |- context::format_token_count()    -> token_display
                  |- context::render_bar()            -> context_bar
                  '- assemble: model | dir [| context_bar]
stdout
```

### After (proposed)

```
stdin JSON
  -> main::run()                                     [src/main.rs:8]
       -> serde_json::from_str -> StatusInput        [src/types.rs:6]
            -> format::build_statusline()            [src/format.rs:31]
                  |- path_format::format_directory()  -> formatted_dir
                  |- git_branch::get_branch(dir)     -> Option<branch>   ** NEW **
                  |    '- walks up from dir to find .git/
                  |       reads .git/HEAD
                  |       strips "ref: refs/heads/" prefix
                  |       returns None on any error
                  |- context::compute_usage()         -> usage
                  |- context::format_token_count()    -> token_display
                  |- context::render_bar()            -> context_bar
                  '- assemble: model | dir [| branch] [| context_bar]
stdout
```

### Segment Layout States

| Has Branch | Has Context Bar | Output |
|-----------|----------------|--------|
| no | no | `model \| dir` |
| no | yes | `model \| dir \| context_bar` |
| yes | no | `model \| dir \| branch` |
| yes | yes | `model \| dir \| branch \| context_bar` |

### Assembly Refactor Suggestion

The current if/else at `src/format.rs:69-85` handles 2 states. With 4 states, a cleaner pattern avoids nested conditionals:

```rust
let mut segments = vec![model_segment, dim(&formatted_dir, no_color)];
if let Some(ref branch) = branch_name {
    segments.push(dim(branch, no_color));
}
if !context_bar.is_empty() {
    segments.push(context_bar.to_string());
}
segments.join(SEPARATOR)
```

This naturally extends to future segments without combinatorial explosion.

### Test Impact Analysis

**Unit tests in `src/format.rs`** -- Tests construct `StatusInput` with arbitrary directory paths (e.g., `/tmp`). Since `git_branch::get_branch("/tmp")` will return `None` (no `.git` there), existing segment count assertions remain valid without modification:
- `build_statusline_with_context_has_three_segments` (line 169) -- still 3 segments (no branch for `/tmp`)
- `build_statusline_without_context_has_two_segments` (line 202) -- still 2 segments

**Integration tests in `tests/integration.rs`** -- The binary runs from the project root (a git repo), but `get_branch()` uses the `current_dir` from the JSON input, not the binary's cwd. Tests that use `current_dir: "/tmp"` will still produce no branch segment. However:
- `full_input_has_separator_between_dir_and_context_bar` (line 319) -- uses `current_dir` from `full_json()` which is the project path (`/Users/jamescarr/Git/jamesacarr/claude-statusline`). This IS a git repo, so the branch segment will appear, changing segment count from 3 to 4. **This test must be updated.**
- `valid_full_input_exits_zero_and_contains_expected_output` (line 56) -- uses same `full_json()` with the project path. Will now include a branch name in output. Existing assertions still pass (they check for contains, not exact match), but a new assertion for branch presence should be added.

**New tests needed:**
- Unit test: `build_statusline` with a path that is a git repo (using tempdir with `.git/HEAD` fixture)
- Unit tests for `git_branch::get_branch_from()`: valid branch, detached HEAD, missing `.git`, nested subdirectory, `.git` as file (worktrees)
- Integration test: verify branch name appears in output when `current_dir` points to a git repo

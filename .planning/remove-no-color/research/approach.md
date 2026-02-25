# Approach Research

> Task: Remove the NO_COLOR option from the claude-statusline Rust project. This means removing any CLI flag, environment variable handling, configuration, and related logic that supports NO_COLOR.
> Last researched: 2026-02-25

## Current State of NO_COLOR

The `no_color: bool` parameter threads through three layers of the codebase:

1. **Entry point** (`src/main.rs:19-21`): reads `NO_COLOR` env var, passes bool to `build_statusline`
2. **Orchestrator** (`src/format.rs:31`): `build_statusline` accepts `no_color` and passes it to `dim()`, `bold()`, and `context::render_bar()`
3. **Leaf functions**:
   - `format::dim()` (`src/format.rs:13`) -- returns plain text when `no_color` is true
   - `format::bold()` (`src/format.rs:22`) -- returns plain text when `no_color` is true
   - `context::render_bar()` (`src/context.rs:59`) -- skips ANSI color codes and skull emoji when `no_color` is true

Additionally:
- **Integration test** (`tests/integration.rs:157-193`): `no_color_env_strips_ansi_escape_sequences` test
- **Unit tests**: `render_bar_no_color_omits_ansi_sequences` (`src/context.rs:288`), `dim_returns_text_unchanged_when_no_color` (`src/format.rs:101`), `bold_returns_text_unchanged_when_no_color` (`src/format.rs:115`)
- **README** (`README.md:32-43`): "Disable ANSI colors" section documents `NO_COLOR=1` usage
- **Architecture docs** (`.planning/codebase/ARCHITECTURE.md:70`, `.planning/codebase/CONCERNS.md:21`, `.planning/codebase/INTEGRATIONS.md:21`)

## Viable Approaches

### Approach 1: Full Removal with Always-Color

- **What:** Remove the `no_color` parameter from all function signatures, delete all `if no_color` branches, delete the env var check in `main.rs`, delete all NO_COLOR-related tests, and update docs. Every function always emits ANSI codes.
- **How:**
  1. `src/main.rs`: delete lines 19-21 (`NO_COLOR` env var check), change `build_statusline(&data, no_color)` to `build_statusline(&data)`
  2. `src/format.rs`: remove `no_color: bool` from `dim()`, `bold()`, `build_statusline()` signatures; delete `if no_color` branches; always emit ANSI
  3. `src/context.rs`: remove `no_color: bool` from `render_bar()` signature; delete the early return on line 64-66; always emit colored bar
  4. `tests/integration.rs`: delete `no_color_env_strips_ansi_escape_sequences` test (lines 157-193)
  5. `src/format.rs` tests: delete `dim_returns_text_unchanged_when_no_color` and `bold_returns_text_unchanged_when_no_color`
  6. `src/context.rs` tests: delete `render_bar_no_color_omits_ansi_sequences`
  7. `README.md`: delete "Disable ANSI colors" section (lines 32-43)
  8. Update `.planning/codebase/` docs that reference NO_COLOR
- **Pros:**
  - Cleanest result -- eliminates all dead code paths
  - Simplifies every function signature (no threading a bool everywhere)
  - Reduces binary size marginally (fewer branches)
  - Easy to verify completeness via `grep -r no_color` and `grep -r NO_COLOR`
- **Cons:**
  - No way for users to disable ANSI if they ever need to (e.g., piping to a file, debugging)
  - Breaks the [no-color.org](https://no-color.org) convention, which is considered best practice for CLI tools
  - If re-added later, the `no_color` parameter must be re-threaded through all functions
- **Best when:** The project owner is certain NO_COLOR will never be needed again and color output is always consumed by ANSI-capable renderers (Claude Code's status bar)
- **Sources:** `src/main.rs:19-21`, `src/format.rs:13-28,31`, `src/context.rs:59-66`, `tests/integration.rs:157-193`

### Approach 2: Remove Env Var Check Only, Keep Internal Plumbing

- **What:** Remove only the `NO_COLOR` env var check in `main.rs` and always pass `false` to `build_statusline`. Keep the `no_color` parameter in function signatures for potential future use.
- **How:**
  1. `src/main.rs`: delete lines 19-21, hardcode `false` in the `build_statusline` call
  2. Delete the integration test for NO_COLOR
  3. Update README to remove the NO_COLOR section
  4. Optionally suppress dead-code warnings with `#[allow(dead_code)]` on the no_color branches or by keeping tests that exercise `true`
- **Pros:**
  - Minimal code change -- low risk of introducing bugs
  - Easy to re-enable later by restoring the env var check
  - Internal API still supports no-color if needed programmatically
- **Cons:**
  - Leaves dead code in the codebase (the `if no_color` branches are never reached in production)
  - Clippy may warn about unused parameters if the bool is always `false`
  - Unclear intent -- future developers may wonder why the parameter exists but is never `true`
  - Does not achieve the stated goal of "removing related logic"
- **Best when:** The removal is tentative and may be reversed soon
- **Sources:** `src/main.rs:19-21`

## Recommendation

**Approach 1: Full Removal** is the right choice. The task explicitly asks to remove "any CLI flag, environment variable handling, configuration, and related logic." Approach 2 leaves dead code that contradicts this goal.

The change is mechanical and low-risk:
- Remove `no_color: bool` from 4 function signatures (`dim`, `bold`, `build_statusline`, `render_bar`)
- Delete the `if no_color` early-return branches in those 3 functions
- Delete the env var check in `main.rs`
- Delete 4 tests (1 integration, 3 unit)
- Update README and planning docs

The scope is small (roughly 30-40 lines removed across 3 source files) and every change is independently verifiable. Running `cargo test` after the change will confirm nothing else depended on the parameter.

Key consideration from CONCERNS.md: the "Do Not Touch" section at `.planning/codebase/CONCERNS.md:39` warns against removing the NO_COLOR check. This was written as architectural guidance to preserve the feature. Since the task is explicitly to remove NO_COLOR, this warning should be acknowledged and overridden -- the CONCERNS.md entry itself should be updated to reflect the new reality (colors are always on, `is_terminal()` is still not used).

## Open Questions

1. **Should CONCERNS.md and ARCHITECTURE.md be updated as part of this task?** They contain multiple references to NO_COLOR as a key pattern. Leaving them stale would mislead future contributors.
2. **Should the "Do Not Touch" entry in CONCERNS.md be removed or rewritten?** It currently warns against removing NO_COLOR. After removal, it should at minimum note that `is_terminal()` must still not be used (that part remains valid).

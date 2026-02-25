---
task_id: remove-no-color
title: Remove NO_COLOR option from claude-statusline
status: completed
created: 2026-02-25T20:47:00Z
updated: 2026-02-25T16:29:54Z
current_wave: null
current_task: null
pause_reason: null
---

# Remove NO_COLOR option from claude-statusline

## Goal

All NO_COLOR logic -- the environment variable check, the `no_color: bool` parameter threaded through function signatures, conditional branches, related tests, and documentation -- is removed from the codebase. The binary always emits ANSI-colored output. The project compiles, all remaining tests pass, and clippy reports zero warnings.

## Success Criteria

1. `grep -r 'no_color' src/ tests/` returns zero matches
2. `grep -r 'NO_COLOR' src/ tests/ README.md` returns zero matches
3. `make test` passes with zero failures
4. `make lint` passes with zero warnings
5. Function signatures for `dim()`, `bold()`, `build_statusline()`, and `render_bar()` do not accept a `no_color` parameter
6. The README.md "Disable ANSI colors" section (lines 32-43) is removed
7. Planning docs (`.planning/codebase/`) no longer reference NO_COLOR as an active feature

## Non-Functional Requirements

1. **Preserve `is_terminal()` rationale** -- The comment at `src/main.rs:20` explaining why `is_terminal()` must not be used is load-bearing institutional knowledge (flagged in CONCERNS.md "Do Not Touch" section). After removing the NO_COLOR code, a standalone comment must remain in `src/main.rs` near the `build_statusline` call explaining that TTY detection must not be added because Claude Code pipes stdout. Verifiable by reading `src/main.rs` for a comment containing "is_terminal" after the change.
2. **No dead code** -- Removing `no_color` branches must not leave unused constants, unreachable code, or clippy warnings. Verifiable by `make lint` passing.

## Wave 1: Remove NO_COLOR from source files

Status: completed

### Task 1.1: Remove NO_COLOR from main.rs

- **Status:** passed
- **Files affected:** `src/main.rs`
- **Action:** In `src/main.rs`: (1) Delete line 19 (the comment `// Check NO_COLOR -- presence of the variable (any value including empty) disables color`). (2) Keep line 20 in place (`// Do NOT check is_terminal() -- Claude Code pipes stdin/stdout and renders ANSI itself`) -- this is the load-bearing rationale comment per CONCERNS.md "Do Not Touch" and NFR #1. (3) Delete line 21 (`let no_color = std::env::var("NO_COLOR").is_ok();`). (4) Update line 24: change `Ok(claude_statusline::format::build_statusline(&data, no_color))` to `Ok(claude_statusline::format::build_statusline(&data))`.
- **Verification:** `grep -c 'no_color\|NO_COLOR' src/main.rs` returns 0 AND `grep -c 'is_terminal' src/main.rs` returns 1 or more.
- **Done when:** `grep -c 'no_color\|NO_COLOR' src/main.rs` returns 0 AND `grep -c 'is_terminal' src/main.rs` returns 1 or more
- **Retries:** 0
- **Last failure:** null

### Task 1.2: Remove NO_COLOR from format.rs

- **Status:** passed
- **Files affected:** `src/format.rs`
- **Action:** In `src/format.rs`: (1) Update `dim()` (line 13): remove `no_color: bool` parameter from signature, delete the `if no_color` branch (lines 14-15), keep only the `else` body so the function unconditionally returns `format!("{}{}{}", DIM, text, RESET)`. Update the doc comment on line 12 to remove the "Returns text unchanged when `no_color` is true" clause -- change to `/// Wrap text in dim ANSI codes.` (2) Update `bold()` (line 22): same treatment -- remove `no_color: bool` parameter, delete `if no_color` branch (lines 23-24), keep only `format!("{}{}{}", BOLD, text, RESET)`. Update doc comment on line 21 similarly. (3) Update `build_statusline()` (line 31): remove `no_color: bool` parameter from signature. Update all internal call sites: `dim(model_name, no_color)` -> `dim(model_name)` (line 67), `dim(&formatted_dir, no_color)` -> `dim(&formatted_dir)` (lines 74, 81), `context::render_bar(*u, &token_display, no_color)` -> `context::render_bar(*u, &token_display)` (line 61). (4) In the `#[cfg(test)]` module: delete `dim_returns_text_unchanged_when_no_color` test (lines 100-104) and `bold_returns_text_unchanged_when_no_color` test (lines 114-118). Update remaining test calls: `super::dim("text", false)` -> `super::dim("text")` (line 96), `super::bold("text", false)` -> `super::bold("text")` (line 110), all `super::build_statusline(&input, false)` calls -> `super::build_statusline(&input)` (lines 145, 188, 215, 248, 275, 290, 309, 331).
- **Verification:** `grep -c 'no_color' src/format.rs` returns 0.
- **Done when:** `grep -c 'no_color' src/format.rs` returns 0
- **Retries:** 0
- **Last failure:** null

### Task 1.3: Remove NO_COLOR from context.rs

- **Status:** passed
- **Files affected:** `src/context.rs`
- **Action:** In `src/context.rs`: (1) Update `render_bar()` (line 59): remove `no_color: bool` parameter from signature. Delete the `if no_color` early-return block (lines 64-66). The function body should go directly from the `bar` construction (line 62) to the `let (color, skull) = ...` threshold logic (line 68 onwards). Update the doc comment on lines 55-58 to remove any reference to no_color. (2) In the `#[cfg(test)]` module: delete `render_bar_no_color_omits_ansi_sequences` test (lines 287-297). Update all remaining `render_bar` test calls to remove the third `false` argument: `render_bar(40, "5.0k", false)` -> `render_bar(40, "5.0k")` (lines 233, 248, 255, 265, 276).
- **Verification:** `grep -c 'no_color' src/context.rs` returns 0.
- **Done when:** `grep -c 'no_color' src/context.rs` returns 0
- **Retries:** 0
- **Last failure:** null

## Wave 2: Compile check, remove integration test, and lint

Status: completed

### Task 2.1: Verify compilation after signature changes

- **Status:** passed
- **Files affected:** (none -- verification only)
- **Action:** Run `cargo check` to confirm all three source files (`main.rs`, `format.rs`, `context.rs`) compile together after the `no_color` parameter removal across all function signatures. This verifies that all call sites were updated consistently in Wave 1.
- **Verification:** `cargo check` exits with code 0.
- **Done when:** `cargo check` exits 0
- **Retries:** 0
- **Last failure:** null

### Task 2.2: Remove NO_COLOR integration test

- **Status:** passed
- **Files affected:** `tests/integration.rs`
- **Action:** In `tests/integration.rs`: delete the entire Test 6 block -- the comment on line 157 (`// --- Test 6: NO_COLOR environment variable ---`) and the `no_color_env_strips_ansi_escape_sequences` test function (lines 159-193). No other integration tests reference NO_COLOR or need modification.
- **Verification:** `grep -c 'no_color\|NO_COLOR' tests/integration.rs` returns 0. `make test` passes with zero failures.
- **Done when:** `make test` passes AND `grep -c 'no_color\|NO_COLOR' tests/integration.rs` returns 0
- **Retries:** 0
- **Last failure:** null

### Task 2.3: Run full lint check

- **Status:** passed
- **Files affected:** (none -- verification only)
- **Action:** Run `make lint` to confirm clippy passes with zero warnings after all NO_COLOR code has been removed. This checks for dead code, unused variables, and any other issues introduced by the removal.
- **Verification:** `make lint` exits with code 0.
- **Done when:** `make lint` exits 0
- **Retries:** 0
- **Last failure:** null

## Wave 3: Update documentation

Status: completed

### Task 3.1: Remove NO_COLOR section from README.md

- **Status:** passed
- **Files affected:** `README.md`
- **Action:** In `README.md`: delete the "Disable ANSI colors" section, which spans lines 32-43 (the `### Disable ANSI colors` heading, the descriptive text, and the JSON code block). The line before (line 30, "Claude Code pipes JSON session data...") and the line after (line 45, `## Development`) should remain, with a single blank line between them.
- **Verification:** `grep -c 'NO_COLOR\|Disable ANSI' README.md` returns 0. Visual inspection confirms the README flows from the Usage section directly to the Development section.
- **Done when:** `grep -c 'NO_COLOR\|Disable ANSI' README.md` returns 0
- **Retries:** 0
- **Last failure:** null

### Task 3.2: Update planning docs to remove NO_COLOR references

- **Status:** passed
- **Files affected:** `.planning/codebase/CONCERNS.md`, `.planning/codebase/TESTING.md`, `.planning/codebase/INTEGRATIONS.md`, `.planning/codebase/ARCHITECTURE.md`
- **Action:** Update the four planning codebase map files to reflect that NO_COLOR is no longer a feature. Exact edits for each file:

  **(1) `.planning/codebase/CONCERNS.md`:**
  - In the "Tech Debt" table, delete the entire `TERM=dumb` row (line 10): `| \`TERM=dumb\` not handled | ... | \`src/main.rs:19-21\` | low |`
  - In "Known Pitfalls", replace the `is_terminal()` bullet (line 21) with: `- **\`is_terminal()\` must not be used for color detection**: Claude Code pipes stdout; \`is_terminal()\` will always return false in normal operation. Using it to gate ANSI output would permanently disable colours. The binary always emits ANSI -- do not add TTY detection. See comment at \`src/main.rs\`.`
  - In "Do Not Touch", replace the `src/main.rs:20` entry (line 39) with: `- \`src/main.rs\` (comment about \`is_terminal()\`) -- the decision to skip TTY detection is load-bearing. The binary always emits ANSI codes. Do not add \`is_terminal()\` checks; Claude Code pipes stdout and renders ANSI itself.`

  **(2) `.planning/codebase/TESTING.md`:**
  - In the "What Is Tested" table, delete the `NO_COLOR env variable` row (line 57): `| NO_COLOR env variable | \`tests/integration.rs\` | Integration, env injection |`

  **(3) `.planning/codebase/INTEGRATIONS.md`:**
  - In the "Environment Variables" table, delete the `NO_COLOR` row (lines 21-22): `| \`NO_COLOR\` | When present ... | \`src/main.rs\` |`
  - In "Prescriptive Guidance", replace line 33 with: `- New env var checks should be placed in \`src/main.rs\` and passed as parameters into library functions rather than read inside library modules`

  **(4) `.planning/codebase/ARCHITECTURE.md`:**
  - In "Key Patterns", delete the entire `NO_COLOR compliance` paragraph (line 70): `**\`NO_COLOR\` compliance.** The binary checks \`std::env::var("NO_COLOR").is_ok()\` rather than \`is_terminal()\` because Claude Code pipes stdout; presence of the variable (any value) disables ANSI codes (\`src/main.rs:21\`).`
  - In the data flow diagram (line 46), update `format::build_statusline()` -- no parameter change needed in the diagram as it does not show arguments.

- **Verification:** `grep -rc 'NO_COLOR' .planning/codebase/CONCERNS.md .planning/codebase/TESTING.md .planning/codebase/INTEGRATIONS.md .planning/codebase/ARCHITECTURE.md` shows 0 for all four files.
- **Done when:** `grep -rc 'NO_COLOR' .planning/codebase/CONCERNS.md .planning/codebase/TESTING.md .planning/codebase/INTEGRATIONS.md .planning/codebase/ARCHITECTURE.md` shows 0 for all four files
- **Retries:** 0
- **Last failure:** null

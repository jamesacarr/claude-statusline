# Codebase Integration Research

> Task: Remove the NO_COLOR option from the claude-statusline Rust project. This means removing any CLI flag, environment variable handling, configuration, and related logic that supports NO_COLOR.
> Last researched: 2026-02-25

## Affected Code

| File/Module | Role | Change Type |
|------------|------|-------------|
| `src/main.rs` | Reads `NO_COLOR` env var and passes `no_color: bool` to `build_statusline` | modify |
| `src/format.rs` | `dim()`, `bold()`, and `build_statusline()` all accept and thread `no_color: bool` | modify |
| `src/context.rs` | `render_bar()` accepts `no_color: bool` and branches on it | modify |
| `tests/integration.rs` | Test 6 (`no_color_env_strips_ansi_escape_sequences`) validates NO_COLOR behaviour | modify (delete test) |
| `README.md` | Documents "Disable ANSI colors" section with `NO_COLOR=1` usage example | modify |
| `.planning/codebase/CONCERNS.md` | References NO_COLOR in tech debt, known pitfalls, and "Do Not Touch" sections | modify |
| `.planning/codebase/TESTING.md` | Lists NO_COLOR as a tested integration concern | modify |
| `.planning/codebase/INTEGRATIONS.md` | Documents `NO_COLOR` as an external integration and prescriptive guidance | modify |
| `.planning/codebase/ARCHITECTURE.md` | Documents `NO_COLOR` compliance as a key pattern | modify |

## Entry Points

The sole entry point for NO_COLOR is `src/main.rs:21`:

```rust
let no_color = std::env::var("NO_COLOR").is_ok();
```

This boolean is passed into `format::build_statusline(&data, no_color)` at line 24. From there it threads through:

1. **`format::build_statusline()`** (line 31) -- passes `no_color` to `dim()`, `bold()`, and `context::render_bar()`
2. **`format::dim()`** (line 13) -- if `no_color`, returns plain text; otherwise wraps in `\x1b[2m...\x1b[0m`
3. **`format::bold()`** (line 22) -- if `no_color`, returns plain text; otherwise wraps in `\x1b[1m...\x1b[0m`
4. **`context::render_bar()`** (line 59) -- if `no_color`, returns plain bar without ANSI codes or skull emoji; otherwise applies color thresholds

## Existing Patterns to Follow

- **Parameter removal cascades** -- the `no_color` bool is passed as a function argument, not stored in a struct or global. Removal means deleting the parameter from each function signature and removing all `if no_color { ... } else { ... }` branches, keeping only the `else` (color-enabled) branch.
- **ANSI constants are already defined** -- `DIM`, `BOLD`, `RESET` constants in `src/format.rs:5-7` are used in the color-enabled branches. After removal, `dim()` and `bold()` will unconditionally use these constants.
- **`render_bar` early-return** -- `src/context.rs:64-66` has a separate early-return format string for no-color mode. After removal, delete the early return and keep only the color-enabled path (lines 68-81).

## Shared Code to Reuse

No new shared code is needed. The change is purely subtractive.

## Dependencies

No dependency changes. No crates are added or removed.

## Data Flow

### Before

```
main.rs: NO_COLOR env var check --> no_color: bool
  --> format::build_statusline(input, no_color)
        --> format::dim(text, no_color)        -- conditional ANSI wrapping
        --> format::bold(text, no_color)       -- conditional ANSI wrapping
        --> context::render_bar(u, tokens, no_color) -- conditional color/plain
```

### After

```
main.rs: (no env var check)
  --> format::build_statusline(input)
        --> format::dim(text)        -- always wraps in ANSI
        --> format::bold(text)       -- always wraps in ANSI
        --> context::render_bar(u, tokens) -- always applies color thresholds
```

## Detailed Change List

### `src/main.rs`
- **Delete** lines 19-21 (the `NO_COLOR` env var check and associated comments)
- **Modify** line 24: remove `no_color` argument from `build_statusline()` call

### `src/format.rs`
- **`dim()`**: remove `no_color: bool` parameter; remove the `if no_color` branch; unconditionally return `format!("{}{}{}", DIM, text, RESET)`
- **`bold()`**: remove `no_color: bool` parameter; remove the `if no_color` branch; unconditionally return `format!("{}{}{}", BOLD, text, RESET)`
- **`build_statusline()`**: remove `no_color: bool` parameter; update all 5 call sites of `dim()`, `bold()`, and `render_bar()` within this function to drop the `no_color` argument
- **Tests**: delete `dim_returns_text_unchanged_when_no_color` (line 100-104) and `bold_returns_text_unchanged_when_no_color` (line 114-118). All remaining tests already pass `false` for `no_color` and will just need the argument removed.

### `src/context.rs`
- **`render_bar()`**: remove `no_color: bool` parameter; delete the early-return branch at lines 64-66; keep the color-enabled path (lines 68-81) as the sole implementation
- **Tests**: delete `render_bar_no_color_omits_ansi_sequences` (line 288-297). All remaining `render_bar` tests pass `false` and will just need the argument removed.

### `tests/integration.rs`
- **Delete** the entire Test 6 block: `no_color_env_strips_ansi_escape_sequences` (lines 157-193)

### `README.md`
- **Delete** the "Disable ANSI colors" section (lines 32-43): the heading, description, and JSON code block

### Planning docs (`.planning/codebase/`)
- Update `CONCERNS.md`, `TESTING.md`, `INTEGRATIONS.md`, `ARCHITECTURE.md` to remove NO_COLOR references. These are documentation-only changes and can be done as a follow-up or in the same commit.

## Test Impact Summary

| Test | File | Action |
|------|------|--------|
| `dim_returns_text_unchanged_when_no_color` | `src/format.rs` | delete |
| `bold_returns_text_unchanged_when_no_color` | `src/format.rs` | delete |
| `render_bar_no_color_omits_ansi_sequences` | `src/context.rs` | delete |
| `no_color_env_strips_ansi_escape_sequences` | `tests/integration.rs` | delete |
| All other tests in `format.rs` and `context.rs` | both files | modify (remove `no_color` argument) |
| All other integration tests | `tests/integration.rs` | no change needed (they don't set NO_COLOR) |

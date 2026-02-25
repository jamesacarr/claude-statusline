# Concerns

> Last mapped: 2026-02-24T00:00:00Z

## Tech Debt

| Area | Description | Files | Severity |
|------|------------|-------|----------|
| Stale bridge file accumulation | Bridge files written to `$TMPDIR/claude-ctx-{session_id}.json` are never cleaned up. Long-running multi-session usage will accumulate files indefinitely with no TTL or cleanup mechanism. | `src/bridge.rs:12` | medium |
| `TERM=dumb` not handled | `main.rs` explicitly checks `NO_COLOR` but never checks `TERM=dumb`. Terminals declaring `dumb` do not support ANSI codes. The comment acknowledges TTY detection was intentionally skipped but does not account for `TERM=dumb`. | `src/main.rs:19-21` | low |
| Undocumented todo filename convention | The pattern `{session_id}-agent-{agent_id}.json` in `~/.claude/todos/` is matched empirically. It is not part of Claude Code's public statusline API. A naming change in Claude Code would silently break todo lookup. | `src/todos.rs:35` | medium |
| No sanitisation of ANSI in input data | Model names and task descriptions from JSON are embedded directly in ANSI-coloured output strings without stripping potential embedded escape sequences. Corrupt or adversarial JSON could inject terminal escape codes. | `src/format.rs:82-91` | low |
| 80% scaling is a magic constant | The bar graph scaling logic (`raw_used / 80.0 * 100.0`) treats 80% context usage as the effective maximum. This constant is not configurable and has no named constant explaining the rationale in code (it is only in a doc comment). | `src/context.rs:30` | low |

## Known Pitfalls

- **`used_percentage` vs. cumulative token totals**: The `total_input_tokens` + `total_output_tokens` fields in `context_window` are cumulative across the session, not the current context size. Computing a percentage from these will produce values well above 100% in long sessions (known Claude Code bug #13783). The code correctly prefers `used_percentage` — do not introduce manual token-based percentage calculations. Affects: `src/context.rs:17-36`, `src/types.rs:48-53`.

- **`remaining_percentage` and `used_percentage` are null early in sessions**: Documented Claude Code behaviour. The code handles this by returning `None` from `compute_usage`, which causes the context bar to be omitted. Any new feature that assumes these fields are always present will behave incorrectly at session start. Affects: `src/context.rs:21-27`, `src/format.rs:68-71`.

- **`is_terminal()` must not be used for color detection**: Claude Code pipes stdout; `is_terminal()` will always return false in normal operation. Using it to gate ANSI output would permanently disable colours. The current approach (`NO_COLOR` env var only) is intentional. See comment at `src/main.rs:20`.

- **1 MB stdin cap is a hard truncation**: `stdin().take(1_048_576)` in `src/main.rs:13` silently truncates input exceeding 1 MB. Truncated JSON will fail deserialization and produce empty output. This is undocumented behaviour — a future large payload (e.g., if Claude Code adds base64 content) would silently fail.

- **mtime-based todo file selection is racy**: `src/todos.rs:45` sorts candidate todo files by filesystem mtime. On filesystems with coarse mtime granularity (1-second precision) or when two files are written within the same second, sort order is undefined. This can yield the wrong task in fast-moving sessions.

## Fragile Areas

- **Context bar threshold boundaries** — `src/context.rs:75-83` uses hardcoded thresholds (`>= 95`, `>= 81`, `>= 63`) applied to the *scaled* value, not the raw percentage. Any change to the 80% scaling constant at `src/context.rs:30` will silently shift all colour thresholds without any obvious visible indication in the threshold constants themselves.

- **Bridge file session_id validation** — `src/bridge.rs:23-29` blocks `/`, `..`, and `\0` but does not block other special characters (e.g., `:`, `*`, `?` on Windows; backticks). If the project targets Windows in future, the current validation is insufficient.

- **todo file matching prefix logic** — `src/todos.rs:35` uses `name.starts_with(session_id)`. If one session_id is a prefix of another (e.g., `abc` and `abc-def`), files from the longer session could be matched incorrectly. Session IDs appear to be UUIDs in practice, making collision unlikely but not impossible.

- **ANSI reset codes** — Colour wrapping in `src/format.rs:85-88` appends a single `\x1b[0m` at the end of `render_bar`. If the function is refactored to concatenate with non-coloured segments that follow, a missing reset could bleed colour into the rest of the statusline. The current structure is safe, but future refactors of `build_statusline` output assembly must be careful.

## Do Not Touch

- `src/main.rs:20` (comment about `is_terminal()`) — the decision to skip TTY detection is load-bearing. Removing the `NO_COLOR` check or adding `is_terminal()` will break colour output in all normal Claude Code usage.
- `src/bridge.rs:23-29` (session_id path traversal validation) — weakening or removing these guards opens a path traversal vulnerability when constructing filenames from external input.

### Prescriptive Guidance

- When adding fields to `StatusInput` or any nested struct, always use `Option<T>` and `#[serde(default)]`. Never add a non-optional field — Claude Code schema evolves additively and older versions will not send new fields.
- Do not compute context percentages from `total_input_tokens` / `total_output_tokens`. Always use `used_percentage` or derive from `remaining_percentage`. See `src/context.rs:17-36`.
- When writing new code that reads from the filesystem (todos, bridge, home dir), always treat failure as `None`/skip — never propagate I/O errors to stdout. Output correctness must not depend on filesystem availability.
- Any new ANSI colour segment added to `build_statusline` must end with `\x1b[0m` to prevent colour bleed.
- If bridge file cleanup is ever implemented, do it by TTL (compare `timestamp` in the JSON) not by session presence, as external monitors may need to read the file after the session ends.

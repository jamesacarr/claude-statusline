# Integrations

> Last mapped: 2026-02-24T00:00:00Z

## External APIs / Services
None. The binary has no network calls or SDK clients.

## Filesystem Integrations

| Path | Purpose | Code |
|------|---------|------|
| `~/.claude/todos/<session_id>-agent-*.json` | Claude Code todo list files read to surface the current in-progress task | `src/todos.rs` |
| `$TMPDIR/claude-ctx-<session_id>.json` | Bridge file written atomically for `gsd-context-monitor.js` compatibility | `src/bridge.rs` |

The bridge file is written via a write-then-rename pattern to avoid partial reads by external consumers.

## Environment Variables

| Variable | Purpose | Referenced in |
|----------|---------|--------------|
| `NO_COLOR` | When present (any value including empty), disables ANSI colour sequences in output — follows the [no-color.org](https://no-color.org) convention | `src/main.rs` |

No `.env` files or secret stores are used. The binary is stateless and holds no credentials.

## Stdin / Stdout Protocol
- **Stdin**: JSON blob matching `StatusInput` struct (`src/types.rs`) — piped from Claude Code
- **Stdout**: ANSI-formatted statusline string — consumed by the terminal multiplexer or editor statusline plugin
- Stdin is capped at 1 MB (`src/main.rs:13`) to guard against unbounded input

### Prescriptive Guidance
- Do not add network calls or external SDK dependencies — this binary must remain a fast, offline, pipe-safe tool
- If a new filesystem path is needed, follow the pattern in `src/todos.rs` and `src/bridge.rs`: accept a `&Path` parameter in an internal function and expose a public wrapper that resolves the real path; this keeps the code testable with `tempfile`
- New env var checks should be placed in `src/main.rs` alongside the existing `NO_COLOR` check and passed as parameters into library functions rather than read inside library modules
- Do not read env vars inside `src/lib.rs` modules — keep side-effect-free logic in the library, side-effectful wiring in `src/main.rs`

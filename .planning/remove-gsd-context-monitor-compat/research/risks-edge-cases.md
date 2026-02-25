# Risks & Edge Cases Research

> Task: Removing all compatibility with gsd-context-monitor.js from the claude-statusline Rust binary. Research what could go wrong with this removal.
> Last researched: 2026-02-24T21:45:00Z

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| gsd-context-monitor.js stops receiving metrics, context warnings disappear for the user's active sessions | **high** | **high** | Must remove or update `gsd-context-monitor.js` PostToolUse hook in `~/.claude/settings.json` (line 145) AND remove or update `/Users/jamescarr/.claude/hooks/gsd-context-monitor.js` at the same time as deploying the Rust binary. If only the bridge writing is removed but the hook remains registered, the hook will silently exit (no bridge file found) -- context warnings simply stop working. |
| gsd-statusline.js (the JS fallback) still writes bridge files, creating inconsistent state during transition | **medium** | **medium** | The current `~/.claude/settings.json` (line 163) still uses `gsd-statusline.js` as the statusline command. If the user switches to the Rust binary but forgets to update, both could run (though only one statusline command runs at a time). The real risk is a partial migration where the user switches to Rust but leaves the JS hook files in place. Document migration steps clearly. |
| GSD upstream ships bridge-file-dependent features in future versions | **medium** | **medium** | GSD (`gsd-build/get-shit-done`) has 19.5k stars and actively ships `hooks/gsd-context-monitor.js` + `hooks/gsd-statusline.js` that use the bridge file protocol. If the user runs `npx get-shit-done-cc@latest` to update GSD, it may reinstall these hooks. The Rust binary would no longer write bridge files, breaking the newly installed hooks. Pin or fork, or provide bridge writing as opt-in. |
| Stale bridge files accumulate in tmpdir with no writer to refresh them | **low** | **low** | After removal, existing `claude-ctx-*.json` files in `/var/folders/.../T/` become orphaned. The context monitor's 60s staleness check (`STALE_SECONDS = 60` in `gsd-context-monitor.js` line 26) means they're ignored quickly. No cleanup action needed. |
| Removing `bridge.rs` reduces code but may leave dead imports/types | **low** | **low** | `BridgeData` in `src/types.rs` (line 82-87) exists solely for bridge serialization. `src/format.rs` lines 73-79 call `bridge::write_bridge`. `src/lib.rs` line 1 declares `pub mod bridge`. All three must be cleaned up or the build fails -- Rust compiler will catch this. |
| Integration tests that invoke the binary may produce different side effects | **low** | **low** | The integration test `valid_full_input_exits_zero_and_contains_expected_output` in `tests/integration.rs` (line 56) passes `session_id: "test-session-123"` with context data. Currently this writes a bridge file to tmpdir as a side effect. After removal, the side effect disappears. No test asserts on bridge file existence, so tests will still pass. |
| `tempfile` dev-dependency becomes unused after removing bridge tests | **low** | **low** | `tempfile` is used in `src/bridge.rs` tests and `src/todos.rs` tests. Only remove from `Cargo.toml` if `todos.rs` tests also stop using it (they won't -- they still need it). |

## Edge Cases

- **User has both JS statusline AND Rust binary installed** -- Only one statusline command runs per Claude Code session (configured in `settings.json` `statusLine.command`). But if user switches between them, the context monitor expects bridge files from whichever is active. Expected behaviour after removal: context monitor silently exits when no bridge file exists (line 44-47 of `gsd-context-monitor.js`).

- **Subagent sessions** -- `gsd-context-monitor.js` already handles missing bridge files (line 44: "this is a subagent or fresh session -- exit silently"). Removing bridge writing doesn't create new failures for subagents.

- **Context window data present but session_id missing** -- Current code at `src/format.rs` line 74 only writes bridge when `session_id` is non-empty. After removal, this guard becomes irrelevant. No edge case.

- **`remaining_percentage` is None but `used_percentage` is present** -- The Rust binary currently only writes bridge files when `remaining_pct` is `Some` (`src/format.rs` line 75). This means bridge files are already not written in some cases. The context monitor handles missing files gracefully. No new edge case from removal.

- **Concurrent sessions writing bridge files** -- Currently each session writes to its own file (`claude-ctx-{session_id}.json`). After removal, no concurrent write issues. However, the context monitor's `-warned.json` sidecar files (`claude-ctx-{session_id}-warned.json`) will also become orphaned.

- **`NO_COLOR` and bridge writing are independent** -- Bridge writing has no interaction with color mode. Removing bridge writing does not affect `NO_COLOR` behaviour.

## Backward Compatibility

**This is a breaking change for the gsd-context-monitor.js workflow.** The break is silent -- no errors, no crashes -- the context monitor hook simply stops receiving data and stops emitting context warnings to the agent.

### What breaks

1. **Agent-facing context warnings disappear.** `gsd-context-monitor.js` is a PostToolUse hook that injects `additionalContext` messages when context usage exceeds 35% (WARNING) or 25% remaining (CRITICAL). Without bridge files, the agent never sees these warnings and may exhaust context without notice.

2. **The bridge file protocol (`claude-ctx-{session_id}.json`) is the only IPC mechanism between the statusline and the context monitor.** There is no alternative communication channel. Removing bridge writing without providing a replacement means context monitoring stops entirely.

### What does NOT break

- The statusline output itself (model, directory, context bar, task) is unaffected.
- The `gsd-context-monitor.js` hook will continue to run but will `process.exit(0)` silently on every invocation (no metrics file found).
- No other hooks (`gsd-check-update.js`, `gsd-intel-index.js`, `gsd-intel-prune.js`, `gsd-intel-session.js`) reference the bridge file.

### Migration path

Three options:

1. **Remove bridge writing from Rust binary AND remove gsd-context-monitor.js hook** -- Simplest, but loses context monitoring entirely. User accepts the tradeoff.

2. **Move context monitoring into the Rust binary itself** -- Instead of writing a bridge file for a JS hook to read, have the Rust binary output context warnings directly. However, the statusline hook runs on a different lifecycle event than PostToolUse, so this doesn't directly replace the monitor's functionality.

3. **Keep bridge writing as an opt-in feature** -- Add a CLI flag or env var (e.g., `CLAUDE_STATUSLINE_BRIDGE=1`) to optionally write bridge files. Default off, but users who still use gsd-context-monitor.js can opt in.

## Fragile Areas

- **`/Users/jamescarr/.claude/settings.json` lines 140-149** -- The PostToolUse hook registration for `gsd-context-monitor.js`. If bridge writing is removed but this hook config remains, the hook runs uselessly on every tool use (spawns a Node.js process, reads stdin JSON, checks for missing file, exits). Minimal performance cost but unnecessary.

- **`/Users/jamescarr/.claude/hooks/gsd-context-monitor.js`** -- This entire file becomes dead code if bridge files are no longer written. Should be removed or disabled in settings.

- **`/Users/jamescarr/.claude/hooks/gsd-statusline.js` lines 30-45** -- The JS statusline also writes bridge files. If the user switches back to the JS statusline (e.g., Rust binary not available on a machine), bridge writing resumes and the context monitor works again. This creates an inconsistent experience depending on which statusline is active.

## Unknowns

- **Does the user rely on context monitoring warnings in practice?** If the agent-facing context warnings are critical to the user's workflow (preventing context exhaustion during long sessions), removing them has high practical impact. If they're rarely triggered or the user has other mechanisms, the impact is low.

- **Will GSD upstream change the bridge file protocol?** The GSD project is actively maintained (881 commits, 19.5k stars). Future versions may change the context monitoring approach, making the current bridge protocol obsolete anyway. Checking the GSD changelog before deciding would be prudent.

- **Is there a plan to replace the context monitoring functionality?** If Claude Code itself adds native context warnings (some community projects suggest this is coming), the entire bridge+monitor pattern may become unnecessary. This is speculative -- based on training data, not confirmed.

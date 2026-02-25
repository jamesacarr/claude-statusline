# Codebase Integration Research

> Task: Create a Claude Code statusline binary in Rust that reads JSON from stdin and outputs a formatted terminal statusline, replacing the existing Node.js statusline at ~/.claude/hooks/gsd-statusline.js.
> Last researched: 2026-02-25T00:06:21Z

## Affected Code

| File/Module | Role | Change Type |
|------------|------|-------------|
| `/Users/jamescarr/Git/jamesacarr/claude-statusline/` | Empty git repo; all Rust project files go here | create |
| `/Users/jamescarr/.claude/settings.json` (line 162) | `statusLine.command` currently points to Node.js hook | modify (post-build) |
| `/Users/jamescarr/.claude/hooks/gsd-statusline.js` | Existing Node.js statusline (being replaced) | no change (kept as fallback) |
| `/Users/jamescarr/.claude/hooks/gsd-context-monitor.js` | PostToolUse hook that reads the bridge file written by the statusline | no change (must remain compatible) |

## Entry Points

### 1. Claude Code statusLine Configuration

The Rust binary hooks into Claude Code via the `statusLine` field in `~/.claude/settings.json`. Current config at line 160-163:

```json
"statusLine": {
  "type": "command",
  "command": "node \"/Users/jamescarr/.claude/hooks/gsd-statusline.js\""
}
```

After build, this changes to point at the compiled binary:

```json
"statusLine": {
  "type": "command",
  "command": "/Users/jamescarr/Git/jamesacarr/claude-statusline/target/release/claude-statusline"
}
```

Or, if installed to a PATH location, just `"command": "claude-statusline"`.

### 2. Stdin JSON Input

Claude Code pipes JSON to the command via stdin. The binary must read all of stdin, parse JSON, extract fields, and write formatted text to stdout.

## Existing Patterns to Follow

### Stdin JSON Schema (Official, from Claude Code docs)

The full JSON schema sent to statusline commands -- source: [Claude Code statusline docs](https://code.claude.com/docs/en/statusline):

```json
{
  "cwd": "/current/working/directory",
  "session_id": "abc123...",
  "transcript_path": "/path/to/transcript.jsonl",
  "model": {
    "id": "claude-opus-4-6",
    "display_name": "Opus"
  },
  "workspace": {
    "current_dir": "/current/working/directory",
    "project_dir": "/original/project/directory"
  },
  "version": "1.0.80",
  "output_style": { "name": "default" },
  "cost": {
    "total_cost_usd": 0.01234,
    "total_duration_ms": 45000,
    "total_api_duration_ms": 2300,
    "total_lines_added": 156,
    "total_lines_removed": 23
  },
  "context_window": {
    "total_input_tokens": 15234,
    "total_output_tokens": 4521,
    "context_window_size": 200000,
    "used_percentage": 8,
    "remaining_percentage": 92,
    "current_usage": {
      "input_tokens": 8500,
      "output_tokens": 1200,
      "cache_creation_input_tokens": 5000,
      "cache_read_input_tokens": 2000
    }
  },
  "exceeds_200k_tokens": false,
  "vim": { "mode": "NORMAL" },
  "agent": { "name": "security-reviewer" }
}
```

**Nullable/absent fields:**
- `vim` -- only present when vim mode is enabled
- `agent` -- only present with `--agent` flag
- `context_window.current_usage` -- `null` before first API call
- `context_window.used_percentage`, `context_window.remaining_percentage` -- may be `null` early in session

### Fields Used by the Existing Node.js Hook

From `/Users/jamescarr/.claude/hooks/gsd-statusline.js` (lines 16-19):

| Field | Usage | Line |
|-------|-------|------|
| `data.model.display_name` | Model name display | 16 |
| `data.workspace.current_dir` | Directory display (falls back to `process.cwd()`) | 17 |
| `data.session_id` | Todo file lookup + bridge file naming | 18 |
| `data.context_window.remaining_percentage` | Context usage calculation | 19 |

### Fields Needed for New Rust Binary

All of the above, plus for the new "percentage AND token value" display (e.g. `15% (30.5k)`):

| Field | Purpose |
|-------|---------|
| `context_window.used_percentage` | Direct percentage display (avoid re-calculating from `remaining_percentage`) |
| `context_window.current_usage.input_tokens` | Token count for the `(30.5k)` display |
| `context_window.current_usage.cache_creation_input_tokens` | Part of total context tokens |
| `context_window.current_usage.cache_read_input_tokens` | Part of total context tokens |
| `context_window.context_window_size` | For computing actual token usage if needed |

**Token value calculation:** Per the docs, `used_percentage` is calculated from `input_tokens + cache_creation_input_tokens + cache_read_input_tokens`. To display the absolute token value like `30.5k`, sum those three fields from `current_usage` and format with `k` suffix (divide by 1000, one decimal place).

### Context Usage Scaling (80% Limit)

The existing Node.js hook (lines 26-28) scales usage to an 80% ceiling:
```javascript
const rawUsed = Math.max(0, Math.min(100, 100 - rem));
const used = Math.min(100, Math.round((rawUsed / 80) * 100));
```

**Decision needed:** The new binary should decide whether to keep this 80% scaling or use the raw `used_percentage` directly. The official `used_percentage` field does NOT apply the 80% scaling -- it's the raw percentage of the context window.

### Bridge File Format

Written by the statusline for the context-monitor hook to consume. From `/Users/jamescarr/.claude/hooks/gsd-statusline.js` lines 34-39 and `/Users/jamescarr/.claude/hooks/gsd-context-monitor.js` lines 42-57:

**Path:** `{tmpdir}/claude-ctx-{session_id}.json`

**Content:**
```json
{
  "session_id": "abc123...",
  "remaining_percentage": 92,
  "used_pct": 25,
  "timestamp": 1708905600
}
```

**Critical compatibility requirement:** The `gsd-context-monitor.js` hook at `/Users/jamescarr/.claude/hooks/gsd-context-monitor.js` reads this bridge file (line 42-49). It expects:
- `remaining_percentage` (number) -- used for threshold checks at lines 57-63
- `used_pct` (number) -- used in warning messages at lines 101-107
- `timestamp` (unix epoch seconds) -- checked for staleness (60s) at line 53

The Rust binary MUST write this file in the same format to maintain compatibility with the context monitor.

### Todo File Structure

**Directory:** `~/.claude/todos/`

**Filename pattern:** `{session_id}-agent-{agent_id}.json` (from gsd-statusline.js line 70)

The existing hook filters for files matching `session_id` prefix with `-agent-` in the name, sorts by mtime descending, reads the most recent.

**File content:** JSON array of todo objects. From actual file at `/Users/jamescarr/.claude/todos/28a6e2a5-aab1-4c90-8d4c-d2518bcfebd1-agent-28a6e2a5-aab1-4c90-8d4c-d2518bcfebd1.json`:

```json
[
  {
    "content": "Task 1: Create UnflattenedListItem Component with Tests",
    "status": "completed",
    "activeForm": "Creating UnflattenedListItem component with tests"
  },
  {
    "content": "Task 8: Final Manual Verification",
    "status": "in_progress",
    "activeForm": "Awaiting manual verification"
  }
]
```

**Fields per todo:**
- `content` (string) -- task description
- `status` (string) -- one of: `"completed"`, `"in_progress"`, `"pending"`
- `activeForm` (string) -- human-readable active verb form of the task

**Extraction logic** (gsd-statusline.js line 78): Find first todo with `status === "in_progress"`, use its `activeForm` field.

### Directory Display Change

Existing (line 99): `path.basename(dir)` -- shows only the last directory component.

New requirement: Show last 3 path levels with `...` prefix. For `/Users/jamescarr/Git/jamesacarr/claude-statusline`:
- Old: `claude-statusline`
- New: `.../jamesacarr/claude-statusline` (if only 2 levels below a cutoff) or `...Git/jamesacarr/claude-statusline`

### Output Format

Existing Node.js output (lines 101-103):
```
{gsdUpdate}{dim model} | {bold task} | {dim dirname}{ctx}
```
or without task:
```
{gsdUpdate}{dim model} | {dim dirname}{ctx}
```

New Rust output (no GSD update, new dir format, new context format):
```
{dim model} | {bold task} | {dim ...path/components}{ctx with tokens}
```

### ANSI Color Codes Used

From the existing hook:
- `\x1b[2m` -- dim (model, dirname)
- `\x1b[1m` -- bold (task)
- `\x1b[0m` -- reset
- `\x1b[32m` -- green (context < 63%)
- `\x1b[33m` -- yellow (context 63-80%)
- `\x1b[38;5;208m` -- orange/256-color (context 81-94%)
- `\x1b[5;31m` -- blinking red (context >= 95%)

## Shared Code to Reuse

- No existing Rust code in the repo (empty project)
- The bridge file protocol is shared with `/Users/jamescarr/.claude/hooks/gsd-context-monitor.js` and must be maintained

## Dependencies

### Rust Crates Needed

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | JSON deserialization of stdin input and bridge file serialization |
| `dirs` | Cross-platform home directory resolution (`~/.claude/todos/`) |

No other crates should be needed. `std::io` handles stdin reading, `std::fs` handles file I/O, `std::env::temp_dir()` handles tmpdir, and `std::time` handles unix timestamps.

### Build Output

The binary should be compiled with `cargo build --release` and the resulting artifact at `target/release/claude-statusline` is what gets referenced in `settings.json`.

## Data Flow

### Before (Node.js)

```
Claude Code --> stdin JSON --> node gsd-statusline.js --> stdout (formatted text)
                                    |
                                    +--> writes /tmp/claude-ctx-{session}.json (bridge)
                                    |
                                    +--> reads ~/.claude/todos/{session}-agent-*.json
                                    |
                                    +--> reads ~/.claude/cache/gsd-update-check.json (GSD update)
```

### After (Rust binary)

```
Claude Code --> stdin JSON --> claude-statusline (Rust) --> stdout (formatted text)
                                    |
                                    +--> writes /tmp/claude-ctx-{session}.json (bridge, same format)
                                    |
                                    +--> reads ~/.claude/todos/{session}-agent-*.json
                                    |
                                    (no GSD update check)
```

### Execution Model

Per the [official docs](https://code.claude.com/docs/en/statusline):
- Script runs after each new assistant message, permission mode change, or vim mode toggle
- Updates debounced at 300ms
- If a new update triggers while the script is running, the in-flight execution is cancelled
- The binary must be fast -- ideally sub-50ms to avoid being cancelled
- Non-zero exit codes or no output cause the status line to go blank
- Must never crash or produce stderr output that could interfere

### tmpdir on macOS

`std::env::temp_dir()` on macOS returns `/var/folders/.../T/` (the per-user temp dir), while Node.js `os.tmpdir()` returns the same. Both resolve to the same location via `$TMPDIR`. This should be compatible, but worth verifying the existing bridge files are at the path the context monitor expects.

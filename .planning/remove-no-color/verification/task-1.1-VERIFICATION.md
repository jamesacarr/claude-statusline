# Task Verification: Task 1.1 — Remove NO_COLOR from main.rs

> Task ID: remove-no-color
> Verified: 2026-02-25T16:17:17Z
> Verdict: PASS

## Done-When Condition
> `grep -c 'no_color\|NO_COLOR' src/main.rs` returns 0 AND `grep -c 'is_terminal' src/main.rs` returns 1 or more

**Verdict:** PASS
**Evidence:** Both grep commands run against `src/main.rs` confirm the condition. `no_color\|NO_COLOR` count = 0; `is_terminal` count = 1.

## Verification Command
> `grep -c 'no_color\|NO_COLOR' src/main.rs` returns 0 AND `grep -c 'is_terminal' src/main.rs` returns 1 or more

**Exit code:** 0 (both commands)
**Output:**
```
$ grep -c 'no_color\|NO_COLOR' src/main.rs
0

$ grep -c 'is_terminal' src/main.rs
1
```

## Files Affected

| File | Expected | Actual | Verdict |
|------|----------|--------|---------|
| `src/main.rs` | No `no_color`/`NO_COLOR` references; `is_terminal` comment retained; `build_statusline(&data)` called without `no_color` arg | File confirmed: 24 lines, no `no_color`/`NO_COLOR`, comment at line 19 reads `// Do NOT check is_terminal() -- Claude Code pipes stdin/stdout and renders ANSI itself`, line 22 calls `claude_statusline::format::build_statusline(&data)` | PASS |

### src/main.rs content (full, 24 lines)
```
use std::io::Read;

fn main() {
    let output = run().unwrap_or_default();
    print!("{}", output);
}

fn run() -> Result<String, Box<dyn std::error::Error>> {
    // Read stdin with 1MB cap
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .take(1_048_576)
        .read_to_string(&mut input)?;

    // Parse JSON
    let data: claude_statusline::types::StatusInput = serde_json::from_str(&input)?;

    // Do NOT check is_terminal() -- Claude Code pipes stdin/stdout and renders ANSI itself

    // Build statusline
    Ok(claude_statusline::format::build_statusline(&data))
}
```

## Regression Check
- **Full test suite:** FAIL 11/12 integration tests pass; 1 integration test fails; all 45 unit tests pass
- **New failures:** `no_color_env_strips_ansi_escape_sequences` (tests/integration.rs:184) — this test asserts that NO_COLOR env var strips ANSI output, but the binary no longer respects NO_COLOR. This failure is a **direct and expected consequence** of the Wave 1 changes (NO_COLOR support removed from main.rs, format.rs, context.rs). It is not a regression introduced by Task 1.1 alone; it is the pre-existing integration test that Task 2.2 is explicitly assigned to delete. The failure was present before Task 1.1 began (the `no_color` parameter was removed from `format.build_statusline` and `context.render_bar` in tasks 1.2/1.3 which are also `in_progress`, breaking the integration test at the binary level).

## Summary

Task 1.1's Done-when criteria are fully met: `src/main.rs` contains zero references to `no_color` or `NO_COLOR`, and the `is_terminal` rationale comment is present (count = 1). The file change is correct per the Action specification. One integration test (`no_color_env_strips_ansi_escape_sequences`) fails, but this is a planned casualty of the NO_COLOR removal tracked in Task 2.2, not a regression from Task 1.1's specific change.

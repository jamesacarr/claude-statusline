# Task Verification: Task 1.2 — Remove NO_COLOR from format.rs

> Task ID: remove-no-color
> Verified: 2026-02-25T21:00:00Z
> Verdict: FAIL

## Done-When Condition
> `grep -c 'no_color' src/format.rs` returns 0

**Verdict:** PASS
**Evidence:** `grep -c 'no_color' src/format.rs` returned `0` (exit code 1 from grep means no matches, which is the expected result). Direct file inspection of `src/format.rs` confirms zero occurrences of the string `no_color`.

## Verification Command
> `grep -c 'no_color' src/format.rs` returns 0

**Exit code:** grep exits 1 when count is 0 (no matches found), consistent with a count of 0
**Output:**
```
0
```

## Files Affected

| File | Expected | Actual | Verdict |
|------|----------|--------|---------|
| `src/format.rs` | `no_color: bool` removed from `dim()`, `bold()`, `build_statusline()` signatures | `dim(text: &str)`, `bold(text: &str)`, `build_statusline(input: &StatusInput)` — no `no_color` parameter | PASS |
| `src/format.rs` | `dim_returns_text_unchanged_when_no_color` test deleted | Not present (grep returns no matches) | PASS |
| `src/format.rs` | `bold_returns_text_unchanged_when_no_color` test deleted | Not present (grep returns no matches) | PASS |
| `src/format.rs` | Remaining test calls updated (e.g. `super::dim("text", false)` -> `super::dim("text")`) | All test calls use updated single-argument form (confirmed by zero `no_color` grep matches) | PASS |

### Supporting Evidence

Function signatures confirmed at lines 13, 18, 23:
```
13: pub fn dim(text: &str) -> String {
18: pub fn bold(text: &str) -> String {
23: pub fn build_statusline(input: &StatusInput) -> String {
```

No `no_color` string anywhere in `src/format.rs` (grep produces 0 matches, no output lines).

## Regression Check
- **Full test suite:** FAIL — 57 passed, 1 failed (integration)
- **New failures:** `tests/integration.rs::no_color_env_strips_ansi_escape_sequences`

### Failure Detail

```
---- no_color_env_strips_ansi_escape_sequences stdout ----
thread 'no_color_env_strips_ansi_escape_sequences' (14566506) panicked at tests/integration.rs:184:5:
should not contain ANSI escape sequences when NO_COLOR is set
```

This failure is a regression introduced by Task 1.2's work (or earlier Wave 1 work). The integration test `no_color_env_strips_ansi_escape_sequences` in `tests/integration.rs` (lines 157-193) still exists and now fails because the `NO_COLOR` environment variable no longer causes the binary to strip ANSI codes — the source-level `no_color` support was removed but the integration test was not yet deleted. That deletion is scoped to Task 2.2.

This is an expected intermediate state: Task 2.2 is explicitly chartered to delete this integration test. However, the failing test constitutes a regression in the current state of the codebase and is flagged here per verification protocol.

### Unit tests (45/45 pass)

All unit tests in `src/context.rs`, `src/format.rs`, `src/path_format.rs`, `src/types.rs` pass. The 10 unit tests in `src/format.rs` that were updated by Task 1.2 all pass, confirming the function signature changes are internally consistent.

## Summary

Task 1.2's primary criterion — `grep -c 'no_color' src/format.rs` returns 0 — is met. All three function signatures (`dim`, `bold`, `build_statusline`) no longer accept a `no_color: bool` parameter, and both no_color-specific unit tests (`dim_returns_text_unchanged_when_no_color`, `bold_returns_text_unchanged_when_no_color`) have been removed. However, the full test suite fails: the integration test `no_color_env_strips_ansi_escape_sequences` in `tests/integration.rs` still exists and now fails because the binary always emits ANSI regardless of the `NO_COLOR` env var. This is an expected intermediate state — Task 2.2 is chartered to delete this test — but the regression causes this verification to return FAIL per protocol.

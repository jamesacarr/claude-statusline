---
task_id: remove-gsd-context-monitor-compat
title: Remove gsd-context-monitor.js compatibility layer
status: planning
created: 2026-02-25T02:27:19Z
updated: 2026-02-25T02:50:00Z
current_wave: null
current_task: null
pause_reason: null
---

# Remove gsd-context-monitor.js compatibility layer

## Goal

When this plan is complete, all code, types, tests, imports, and module declarations related to the `gsd-context-monitor.js` bridge compatibility layer have been removed from the codebase. The binary compiles cleanly, all remaining tests pass, clippy produces no warnings, and the binary's stdout output is unchanged for all existing inputs.

## Success Criteria

1. `src/bridge.rs` does not exist
2. No reference to `bridge`, `BridgeData`, or `write_bridge` exists anywhere in `src/`
3. `serde::Serialize` is not imported in `src/types.rs` (no remaining consumer)
4. `cargo build` succeeds with no errors
5. `cargo test` passes with all remaining tests (8 fewer tests than before: 7 from `bridge.rs`, 1 from `types.rs`)
6. `cargo clippy -- -D warnings` produces no warnings
7. `cargo fmt --check` produces no formatting drift
8. The binary's stdout output for all integration test inputs is byte-identical before and after the change

## Non-Functional Requirements

1. **Security improvement (verifiable):** After removal, the binary performs no filesystem writes (the bridge was the only write path). Verify by grepping `src/` for `fs::write` and `fs::rename` -- zero matches expected since the only occurrences are in the deleted `bridge.rs`.
2. **No new `#[allow(dead_code)]` annotations:** Verify by grepping `src/` for `allow(dead_code)` -- count must not increase from zero.

## Wave 1: Delete bridge module and remove module declaration
Status: pending

### Task 1.1: Delete src/bridge.rs
- **Status:** pending
- **Files affected:** `src/bridge.rs`
- **Action:** Delete the file `src/bridge.rs` entirely. This file contains 174 lines: the `write_bridge()` and `write_bridge_to()` functions, session ID validation guards, atomic file write logic, and 7 unit tests -- all exclusively serving gsd-context-monitor.js compatibility.
- **Verification:** `test ! -f src/bridge.rs`
- **Done when:** `src/bridge.rs` does not exist on disk
- **Retries:** 0
- **Last failure:** null

### Task 1.2: Remove bridge module declaration from src/lib.rs
- **Status:** pending
- **Files affected:** `src/lib.rs`
- **Action:** Remove line 1 (`pub mod bridge;`) from `src/lib.rs`. The remaining module declarations (`context`, `format`, `path_format`, `todos`, `types`) are unchanged. After this edit, `src/lib.rs` should contain 5 `pub mod` lines.
- **Verification:** `grep -c 'bridge' src/lib.rs` returns 0 (exit code 1)
- **Done when:** `src/lib.rs` contains no reference to `bridge`
- **Retries:** 0
- **Last failure:** null

## Wave 2: Remove bridge references from format.rs and types.rs
Status: pending

### Task 2.1: Remove bridge import and call site from src/format.rs
- **Status:** pending
- **Files affected:** `src/format.rs`
- **Action:** In `src/format.rs`, make two removals: (1) Remove line 1 (`use crate::bridge;`) -- the bridge import. (2) Remove lines 73-79 -- the bridge-writing block inside `build_statusline()` which reads:
  ```rust
  // Write bridge file (best-effort, only if session and remaining exist)
  if !session_id.is_empty() {
      if let Some(remaining) = remaining_pct {
          let scaled = usage.as_ref().map(|u| u.scaled_used).unwrap_or(0);
          bridge::write_bridge(session_id, remaining, scaled);
      }
  }
  ```
  After removal, `build_statusline()` should flow directly from the `context_bar` match block to the "Assemble output" section. The variables `remaining_pct`, `session_id`, and `usage` remain in use by other code in the function -- do not remove them.
- **Verification:** `grep -c 'bridge' src/format.rs` returns 0 (exit code 1)
- **Done when:** `src/format.rs` contains no reference to `bridge` or `write_bridge`
- **Retries:** 0
- **Last failure:** null

### Task 2.2: Remove BridgeData struct, its test, and clean up Serialize import from src/types.rs
- **Status:** pending
- **Files affected:** `src/types.rs`
- **Action:** In `src/types.rs`, make three changes: (1) On line 1, change `use serde::{Deserialize, Serialize};` to `use serde::Deserialize;` -- `Serialize` is only used by `BridgeData` and no other struct in this file derives it. (2) Remove lines 80-87 -- the `BridgeData` struct and its doc comment (`/// Bridge data written for gsd-context-monitor.js compatibility.`). (3) Remove lines 233-246 -- the `serializes_bridge_data` test inside `mod tests`. Preserve all surrounding code: `TodoItem` (lines 71-78) remains above the removal site, and the `deserializes_null_optional_fields_as_none` test (currently lines 248-255) remains after.
- **Verification:** `grep -c 'BridgeData\|Serialize\|bridge' src/types.rs` returns 0 (exit code 1)
- **Done when:** `src/types.rs` contains no reference to `BridgeData`, `Serialize`, or `bridge`
- **Retries:** 0
- **Last failure:** null

## Wave 3: Compile, test, and lint verification
Status: pending

### Task 3.1: Verify clean build and all tests pass
- **Status:** pending
- **Files affected:** (none -- verification only)
- **Action:** Run the full build and test suite from the project root to confirm the removal is complete and no dangling references remain. The Rust compiler will catch any missed imports or type references at compile time. Execute `cargo build` followed by `cargo test`. All remaining tests must pass. Expect 8 fewer tests than before (7 from deleted `bridge.rs`, 1 `serializes_bridge_data` from `types.rs`).
- **Verification:** `cargo test`
- **Done when:** `cargo test` exits 0 with all tests passing and no compilation errors
- **Retries:** 0
- **Last failure:** null

### Task 3.2: Verify clippy produces no warnings
- **Status:** pending
- **Files affected:** (none -- verification only)
- **Action:** Run clippy with warnings-as-errors to confirm no dead code, unused imports, or other lint issues were introduced by the removal. This matches the CI configuration in `.github/workflows/ci.yml:35`.
- **Verification:** `cargo clippy -- -D warnings`
- **Done when:** `cargo clippy -- -D warnings` exits 0 with no warnings or errors
- **Retries:** 0
- **Last failure:** null

### Task 3.3: Verify formatting is clean
- **Status:** pending
- **Files affected:** (none -- verification only)
- **Action:** Run `cargo fmt --check` to confirm no formatting drift was introduced. This matches the CI configuration in `.github/workflows/ci.yml:33`.
- **Verification:** `cargo fmt --check`
- **Done when:** `cargo fmt --check` exits 0 with no diff output
- **Retries:** 0
- **Last failure:** null

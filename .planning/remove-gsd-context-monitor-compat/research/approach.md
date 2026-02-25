# Approach Research

> Task: Removing all compatibility with gsd-context-monitor.js from the claude-statusline Rust binary. The gsd-context-monitor.js compatibility is unnecessary and should be fully stripped out. Research viable approaches to identify and remove all traces of this compatibility layer -- code paths, conditionals, format handling, etc.
> Last researched: 2026-02-24 (timestamp MCP unavailable)

## Scope of the gsd-context-monitor.js Compatibility Layer

The bridge exists solely to write a JSON file (`{tmpdir}/claude-ctx-{session_id}.json`) that `gsd-context-monitor.js` reads. The compatibility surface is small and well-isolated:

### Files and code to remove

| File | Lines/Items | What it does |
|------|-------------|--------------|
| `src/bridge.rs` | Entire file (174 lines) | `write_bridge()` and `write_bridge_to()` functions + 6 unit tests |
| `src/types.rs` | Lines 80-87 (`BridgeData` struct) | Serializable struct for bridge JSON |
| `src/types.rs` | Lines 234-246 (test `serializes_bridge_data`) | Unit test for `BridgeData` serialization |
| `src/format.rs` | Line 1 (`use crate::bridge`) | Import of bridge module |
| `src/format.rs` | Lines 73-79 (bridge write block) | Conditional call to `bridge::write_bridge()` in `build_statusline()` |
| `src/lib.rs` | Line 1 (`pub mod bridge`) | Module declaration |

### No external impact

- No integration tests in `tests/integration.rs` reference bridge functionality.
- No other modules (`context.rs`, `todos.rs`, `path_format.rs`, `main.rs`) depend on bridge.
- The `BridgeData` struct derives `Serialize` only (not `Deserialize`), so removing it has no effect on input parsing.

## Viable Approaches

### 1. Direct Surgical Deletion

- **What:** Delete `src/bridge.rs` entirely, remove `BridgeData` from `src/types.rs`, remove the bridge call site in `src/format.rs`, remove the module declaration in `src/lib.rs`.
- **How:** Six targeted edits across 4 files, plus deleting 1 file. The changes are:
  1. Delete `src/bridge.rs`
  2. Remove `pub mod bridge;` from `src/lib.rs`
  3. Remove `use crate::bridge;` from `src/format.rs`
  4. Remove lines 73-79 in `src/format.rs` (the bridge write block)
  5. Remove `BridgeData` struct (lines 80-87) from `src/types.rs`
  6. Remove `serializes_bridge_data` test (lines 234-246) from `src/types.rs`
- **Pros:**
  - Minimal diff -- only touches what's necessary
  - No risk of unintended behavioral changes
  - Easy to review (each change is self-contained)
  - Compile errors immediately surface any missed reference
- **Cons:**
  - None significant. The code is well-isolated.
- **Best when:** The compatibility layer is well-isolated and there are no partial-reuse considerations. This is the case here.
- **Sources:** `src/bridge.rs`, `src/format.rs:1,73-79`, `src/types.rs:80-87,234-246`, `src/lib.rs:1`

### 2. Gut and Stub (Deprecation-First)

- **What:** Keep `src/bridge.rs` and `BridgeData` but make `write_bridge()` a no-op. Remove the call site in `format.rs`. Leave the module declared but empty (or with a deprecation comment).
- **How:**
  1. Replace `write_bridge()` body with `{}` (empty)
  2. Remove the call in `src/format.rs` lines 73-79
  3. Add `#[deprecated]` attribute to `write_bridge` and `BridgeData`
- **Pros:**
  - Preserves the module boundary if someone wants to reintroduce bridge functionality later
  - Slightly smaller diff
- **Cons:**
  - Leaves dead code in the codebase (struct, module, empty function)
  - `cargo clippy` / `#[warn(dead_code)]` will flag the unused items
  - Adds maintenance burden for no benefit
  - Contradicts the task goal of "fully stripped out"
- **Best when:** There is uncertainty about whether the bridge might be needed again. Not the case here -- the task explicitly says to remove all traces.
- **Sources:** N/A (general refactoring pattern)

## Recommendation

**Approach 1: Direct Surgical Deletion** is the clear choice.

Rationale:
1. The compatibility layer is perfectly isolated -- `bridge.rs` is a leaf module with no dependents other than a single call site in `format.rs` and the `BridgeData` type in `types.rs`.
2. The Rust compiler enforces correctness -- after deletion, any missed reference will produce a compile error, making it impossible to leave dangling references.
3. The task explicitly requires full removal ("fully stripped out"), so leaving stubs (Approach 2) directly contradicts the requirement.
4. The total surface is ~180 lines of code removal across 4 files plus 1 file deletion. This is a 15-minute change with high confidence.

**Suggested execution order:**
1. Delete `src/bridge.rs`
2. Edit `src/lib.rs` -- remove `pub mod bridge;`
3. Edit `src/format.rs` -- remove `use crate::bridge;` and the bridge write block (lines 73-79)
4. Edit `src/types.rs` -- remove `BridgeData` struct and its test
5. Run `cargo build` to verify no compile errors
6. Run `cargo test` to verify no test failures
7. Run `cargo clippy` to verify no new warnings

## Open Questions

- **Serde dependency cleanup:** The `Serialize` derive on `BridgeData` is currently the only use of `serde::Serialize` in `types.rs` (all other types use `Deserialize` only). After removal, check whether `Serialize` is still imported/needed anywhere. If not, it can be cleaned from the `use serde::{Deserialize, Serialize}` import in `types.rs` -- though this is cosmetic since the derive macro feature is still required by `Deserialize`.

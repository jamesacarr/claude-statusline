# Quality & Standards Research

> Task: Removing all compatibility with gsd-context-monitor.js from the claude-statusline Rust binary. Research the testing, correctness, and quality implications.
> Last researched: 2026-02-24T20:47:00Z

## Security

Removing the bridge module **eliminates a filesystem write side-effect** that currently writes JSON to `{tmpdir}/claude-ctx-{session_id}.json`. This is a net security improvement:

- Removes atomic temp-file write to a world-readable `/tmp` directory
- Removes session ID-based file naming (path traversal guards in `src/bridge.rs:23-29` become unnecessary)
- The binary becomes a pure stdin-to-stdout filter with no filesystem writes (aside from the read-only todo lookup in `src/todos.rs`)

No new security concerns are introduced by this removal.

## Performance

The bridge write is best-effort and involves:
- One `serde_json::to_string` serialisation
- One `fs::write` + one `fs::rename` (atomic write pattern)

Removing it eliminates two syscalls per invocation. The performance gain is negligible for a statusline binary, but it simplifies the hot path in `src/format.rs:build_statusline()` and removes the `serde::Serialize` derive requirement from `BridgeData`.

## Accessibility

Not applicable (no UI changes). This is a CLI binary producing ANSI-formatted stdout.

## Testing Strategy

### Tests to Remove Entirely

**`src/bridge.rs` -- entire file (7 tests)**
The whole module is being deleted. All 7 unit tests in the `mod tests` block are bridge-specific:

| Test | Lines | Purpose |
|------|-------|---------|
| `writes_bridge_file_with_correct_json_fields` | 66-85 | Validates bridge JSON output |
| `bridge_file_schema_matches_expected_types` | 88-107 | Validates bridge JSON types |
| `does_not_write_file_for_empty_session_id` | 110-120 | Path traversal guard |
| `does_not_write_file_for_session_id_with_path_traversal` | 123-133 | Path traversal guard |
| `does_not_write_file_for_session_id_with_slash` | 136-146 | Path traversal guard |
| `does_not_write_file_for_session_id_with_null_byte` | 149-159 | Path traversal guard |
| `timestamp_is_a_reasonable_unix_epoch` | 162-173 | Timestamp validity |

**`src/types.rs` -- `serializes_bridge_data` test (lines 234-246)**
Tests `BridgeData` serialisation. Must be removed along with the `BridgeData` struct (lines 80-87).

### Tests to Modify

**`src/format.rs` -- `build_statusline_with_full_input_contains_model_directory_and_bar` (lines 137-174)**
This test calls `build_statusline()` with a `session_id` and `remaining_percentage` set, which currently triggers `bridge::write_bridge()` as a side effect (line 77). After removal, the test itself does not assert on bridge behaviour, so it continues to pass **without modification** -- the bridge call simply disappears from `build_statusline()`.

However, the test currently creates a side-effect file in `/tmp`. After removal, this side-effect goes away, which is correct. **No test modification needed**, but worth verifying no test relies on the bridge file existing.

**`tests/integration.rs` -- `valid_full_input_exits_zero_and_contains_expected_output` (lines 56-69)**
Same situation: the integration test pipes JSON with `session_id` through the binary. The bridge file write is a side-effect. The test asserts only on stdout content, not the bridge file. **No modification needed.**

### Tests That Are Unaffected

All other tests across the codebase do not reference bridge functionality:
- `src/context.rs` (9 unit tests) -- pure computation, no bridge dependency
- `src/path_format.rs` (9 unit tests) -- pure path formatting
- `src/todos.rs` (7 unit tests) -- file-based todo lookup, no bridge dependency
- `src/main.rs` (2 unit tests) -- stdin/run error handling
- `src/types.rs` (remaining 6 tests) -- StatusInput/TodoItem deserialisation
- `src/format.rs` (remaining 5 tests) -- statusline assembly (none set session_id + remaining_percentage together to trigger bridge path, except the one noted above which is benign)
- `tests/integration.rs` (remaining 10 tests) -- end-to-end binary tests

### New Tests to Add

No new tests are required for the removal itself. However, consider adding:

1. **Negative regression test** (optional): Assert that `build_statusline()` produces no filesystem side-effects. This would catch accidental re-introduction. Example approach: run `build_statusline()` in a controlled temp dir, assert no files created. Low priority since the bridge code path is being fully deleted.

### Mocking Approach

No mocking changes needed. The bridge module uses a `write_bridge_to()` internal function that accepts a custom directory for testability, but since the entire module is being deleted, the testability pattern is removed with it.

### Verification After Removal

1. `cargo test` -- all remaining tests pass (expect 8 fewer tests: 7 from bridge.rs + 1 from types.rs)
2. `cargo clippy -- -D warnings` -- no dead code warnings, no unused imports
3. `cargo build --release` -- compiles cleanly
4. `cargo fmt --check` -- formatting preserved
5. Manual smoke test: pipe sample JSON through the binary, confirm stdout is unchanged

### Dev Dependency Impact

The `tempfile = "3"` dev-dependency in `Cargo.toml` (line 17) is still needed by `src/todos.rs` tests. No dependency removal needed.

## Standards Checklist

1. All 7 bridge unit tests in `src/bridge.rs` are removed (not left as dead code)
2. The `BridgeData` struct and its serialisation test in `src/types.rs` are removed
3. `pub mod bridge;` is removed from `src/lib.rs`
4. `use crate::bridge;` is removed from `src/format.rs`
5. The bridge-writing block in `src/format.rs:73-79` is removed
6. `cargo test` passes with 0 failures after removal
7. `cargo clippy -- -D warnings` produces no warnings (no dead code, unused imports)
8. The binary's stdout output for all existing integration test inputs is byte-identical before and after removal
9. No new `#[allow(dead_code)]` annotations are introduced
10. CI pipeline (`cargo fmt --check`, `cargo clippy`, `cargo test`) passes green

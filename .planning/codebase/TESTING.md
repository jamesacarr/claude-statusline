# Testing

> Last mapped: 2026-02-24T00:00:00Z

## Test Framework

- Runner: Rust built-in (`cargo test`) — no external test runner config file
- Run command: `cargo test`
- Assertions: standard Rust `assert!`, `assert_eq!` macros plus `predicates` crate for integration tests
- Binary testing: `assert_cmd` crate (`dev-dependencies` in `Cargo.toml`)

## Test Organisation

- Unit tests: colocated in each source file inside `#[cfg(test)]` modules — e.g. `src/context.rs`, `src/bridge.rs`, `src/format.rs`, `src/path_format.rs`, `src/todos.rs`, `src/types.rs`
- Integration tests: `tests/integration.rs` — tests the compiled binary end-to-end via stdin/stdout
- No separate E2E or snapshot test directories

## Test Patterns

### Setup

- **Inline JSON strings**: test data constructed as raw string literals `r#"..."#` — `tests/integration.rs:11-51`, `src/types.rs:95-132`
- **Tempdir fixtures**: `tempfile::tempdir()` used to create isolated filesystem state for `bridge` and `todos` tests — `src/bridge.rs:66`, `src/todos.rs:86`
- **Helper functions**: shared setup extracted into private helpers within `#[cfg(test)]` — `create_todo_file` in `src/todos.rs:77`, `cmd()` and `full_json()` in `tests/integration.rs:6-51`
- **Struct construction with `..Default::default()`**: unit tests build partial `StatusInput` structs using struct update syntax — `src/format.rs:139-156`

### Assertions

- Unit tests: `assert_eq!(actual, expected)` and `assert!(condition, "message")` with descriptive failure messages
- Integration tests: `assert_cmd` fluent API — `.assert().success().stdout(predicate::str::contains(...))` — `tests/integration.rs:57-68`
- Direct output inspection: `cmd().output()` + `String::from_utf8_lossy` for negative assertions (absence of ANSI codes, absence of bar characters) — `tests/integration.rs:117-131`

### Mocking

- No mocking library used
- Filesystem isolation: internal functions accept a custom directory parameter (`write_bridge_to`, `get_current_task_from`) to enable testing without touching real paths — `src/bridge.rs:17`, `src/todos.rs:17`
- Public entry functions (`write_bridge`, `get_current_task`) delegate to these internal `pub(crate)` variants

## Coverage

- No coverage tool configured (`tarpaulin` or similar not in `Cargo.toml` or CI)
- CI runs `cargo test` on every push and PR — `.github/workflows/ci.yml:37`

## What Is Tested

| Area | File | Approach |
|------|------|---------|
| JSON deserialization (all fields, edge cases) | `src/types.rs` | Unit, inline JSON fixtures |
| Path truncation logic | `src/path_format.rs` | Unit, string assertions |
| Context usage computation and clamping | `src/context.rs` | Unit, numeric assertions |
| Bar rendering, color thresholds | `src/context.rs` | Unit, ANSI escape assertions |
| ANSI dim/bold helpers | `src/format.rs` | Unit |
| Full statusline assembly | `src/format.rs` | Unit, struct construction |
| Bridge file write, path traversal guards | `src/bridge.rs` | Unit, tempdir |
| Todo file lookup, mtime ordering | `src/todos.rs` | Unit, tempdir |
| Binary stdin/stdout pipeline | `tests/integration.rs` | Integration, assert_cmd |
| Error resilience (empty/invalid stdin) | `src/main.rs`, `tests/integration.rs` | Unit + integration |

### Prescriptive Guidance

- New unit tests: add a `#[cfg(test)]` module at the bottom of the same `.rs` file; follow the `verb_noun_condition` naming pattern (e.g. `returns_none_for_empty_session_id`)
- New integration tests: add to `tests/integration.rs`; use the `cmd()` helper and `assert_cmd` fluent API; group related tests with a `// --- Test N: Description ---` comment
- New filesystem-touching code: expose a `pub(crate) fn foo_with_dir(dir: &Path, ...)` variant and call it from the public `foo()` function — follow the pattern in `src/bridge.rs:17` and `src/todos.rs:17`
- Example to copy for unit tests: `src/context.rs` — comprehensive coverage of edge cases with clear naming
- Example to copy for integration tests: `tests/integration.rs` — covers happy path, null optionals, env vars, and error cases

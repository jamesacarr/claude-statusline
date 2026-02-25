# Conventions

> Last mapped: 2026-02-24T00:00:00Z

## Naming

- Files: `snake_case.rs` — examples: `src/path_format.rs`, `src/context.rs`, `src/todos.rs`
- Functions/methods: `snake_case` — examples: `build_statusline`, `format_directory`, `get_current_task`
- Variables/constants: `snake_case` for variables; `SCREAMING_SNAKE_CASE` for `const` — examples: `DIM`, `BOLD`, `RESET`, `SEPARATOR` in `src/format.rs`
- Types/structs: `PascalCase` — examples: `StatusInput`, `ModelInfo`, `UsageInfo`, `BridgeData` in `src/types.rs`
- Test functions: descriptive `snake_case` sentences describing behaviour — e.g. `truncates_five_component_path_to_last_three_with_ellipsis_prefix` in `src/path_format.rs`

## File Organisation

- Source root: `src/`
- Integration tests: `tests/`
- No separate `benches/` or `examples/` directories
- Shared types: `src/types.rs`
- Module declarations: `src/lib.rs` (all public modules listed here)
- Entry point: `src/main.rs`

## Modules

Each module is a single flat file — no subdirectories. Module boundaries:

| File | Responsibility |
|------|---------------|
| `src/types.rs` | All shared data types, serde derives |
| `src/format.rs` | Statusline assembly, ANSI formatting helpers |
| `src/context.rs` | Context window computation and bar rendering |
| `src/path_format.rs` | Directory path truncation |
| `src/todos.rs` | Todo file lookup logic |
| `src/bridge.rs` | Bridge file write for external monitor |
| `src/main.rs` | Entry point; reads stdin, calls lib, prints output |

## Imports

- Style: `crate::` paths for internal modules — e.g. `use crate::bridge;`, `use crate::types::StatusInput;`
- External crates imported at the top of each file — e.g. `use serde::{Deserialize, Serialize};` in `src/types.rs`
- Test-only imports inside `#[cfg(test)]` blocks, colocated with the module under test
- No re-exports through `src/lib.rs`; callers use full `crate::module::Item` paths

## Error Handling

- Pattern: propagate with `?` in `run()`, convert to silent fallback at the top level via `unwrap_or_default()` — `src/main.rs:4`
- Best-effort operations (bridge write, todo lookup): silently return early on error using `return;` or `.ok()?` — `src/bridge.rs:47-58`, `src/todos.rs:26`
- No `unwrap()` or `expect()` in production code paths; only in tests
- Errors are never surfaced to the user; a failed parse results in empty stdout

## Code Style

- Formatter: `rustfmt` (default config, no `rustfmt.toml`) — enforced via `cargo fmt --check` in `.github/workflows/ci.yml:33`
- Linter: `clippy` — enforced with `-D warnings` in `.github/workflows/ci.yml:35`
- Doc comments: `///` on all public functions and structs; inline `//` for implementation notes — see `src/bridge.rs`, `src/context.rs`
- `#[serde(default)]` on all deserializable structs to handle missing JSON fields gracefully — `src/types.rs`

### Prescriptive Guidance

- New files: place in `src/`, name in `snake_case.rs`, declare in `src/lib.rs` as `pub mod <name>;`
- New functions: use `snake_case`, add a `///` doc comment, propagate errors with `?` in non-best-effort paths; use early-return patterns for best-effort paths
- New types: add to `src/types.rs`, derive `Debug`, `Default`, and appropriate serde traits; apply `#[serde(default)]` for all JSON-deserialized types
- ANSI constants: add to `src/format.rs` alongside `DIM`, `BOLD`, `RESET`
- Run `cargo fmt` before committing; CI blocks on format and clippy violations

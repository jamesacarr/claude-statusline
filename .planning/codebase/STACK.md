# Stack

> Last mapped: 2026-02-24T00:00:00Z

## Languages
- Rust — edition 2024, MSRV 1.85 — configured in `Cargo.toml`

## Frameworks
- None (binary crate with a library crate, no web/async framework) — entry point: `src/main.rs`

## Package Manager
- Cargo — lockfile: `Cargo.lock`

## Key Dependencies

### Runtime

| Dependency | Version | Purpose | Used in |
|-----------|---------|---------|---------|
| `serde` | 1.x (with `derive` feature) | Serialisation/deserialisation derive macros | `src/types.rs` |
| `serde_json` | 1.x | JSON parsing of Claude Code stdin input and bridge file output | `src/main.rs`, `src/bridge.rs`, `src/todos.rs` |
| `dirs` | 6.x | Resolve `~/.claude/todos/` from home directory | `src/todos.rs` |

### Dev / Test

| Dependency | Version | Purpose | Used in |
|-----------|---------|---------|---------|
| `assert_cmd` | 2.x | Integration test runner — spawns the binary as a subprocess | `tests/integration.rs` |
| `predicates` | 3.x | Assertion predicates for stdout/stderr matching in integration tests | `tests/integration.rs` |
| `tempfile` | 3.x | Create temporary directories for unit tests that touch the filesystem | `src/bridge.rs`, `src/todos.rs` |

## Build & Dev Tools
- `rustfmt`: enforced via `cargo fmt --check` in CI — no custom `rustfmt.toml` present
- `clippy`: enforced via `cargo clippy -- -D warnings` in CI
- Release profile: `Cargo.toml` — `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` (aggressive size optimisation)
- CI: `.github/workflows/ci.yml` — runs check/lint/test then cross-platform builds (linux + macos), releases on `v*` tags via `softprops/action-gh-release@v2`

### Prescriptive Guidance
- Use Rust stable (the CI pins `dtolnay/rust-toolchain@stable`; do not target nightly features)
- Minimum supported Rust version is 1.85 (edition 2024 syntax is in use)
- All new runtime dependencies go in `[dependencies]` in `Cargo.toml`; test-only helpers go in `[dev-dependencies]`
- Run `cargo fmt` before committing — CI will fail on formatting drift
- Run `cargo clippy -- -D warnings` before committing — all clippy warnings are treated as errors in CI
- Use `cargo build --release` to produce the size-optimised binary; the release profile is already tuned

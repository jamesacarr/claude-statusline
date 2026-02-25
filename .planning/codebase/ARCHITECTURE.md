# Architecture

> Last mapped: 2026-02-24T00:00:00Z

## Directory Structure

```
claude-statusline/
├── src/                   # Library + binary source
│   ├── main.rs            # Binary entry point — stdin read, JSON parse, print output
│   ├── lib.rs             # Crate root — re-exports all public modules
│   ├── types.rs           # All serde-deserialisable input/output structs
│   ├── format.rs          # Statusline assembler — orchestrates all other modules
│   ├── context.rs         # Context window usage computation and bar rendering
│   ├── path_format.rs     # Directory path truncation (>3 components → "...")
│   ├── todos.rs           # Reads ~/.claude/todos/ for in-progress task label
│   └── bridge.rs          # Writes tmpdir JSON file for external JS monitor compat
├── tests/
│   └── integration.rs     # Full binary pipeline tests via assert_cmd
├── .github/workflows/
│   └── ci.yml             # CI: fmt check, clippy, test, cross-platform release build
├── Cargo.toml             # Package manifest — single binary crate
└── Cargo.lock             # Locked dependency versions
```

## Module Boundaries

This is a single-crate Rust binary with a thin library layer (`lib.rs` re-exports all modules). There are no workspaces or sub-packages.

| Module | Responsibility | Depends On |
|--------|---------------|------------|
| `main.rs` | I/O orchestration — reads stdin, parses JSON, prints to stdout | `types`, `format` |
| `types.rs` | Data model — all `Deserialize`/`Serialize` structs | (none — leaf) |
| `format.rs` | Assembles the full ANSI statusline string | `types`, `context`, `path_format`, `todos`, `bridge` |
| `context.rs` | Computes usage percentages and renders bar graph | `types` |
| `path_format.rs` | Truncates filesystem paths for display | (none — leaf) |
| `todos.rs` | Reads `~/.claude/todos/*.json` to find active task | `types` |
| `bridge.rs` | Writes `{tmpdir}/claude-ctx-{session_id}.json` atomically | `types` |

## Data Flow

```
stdin (Claude Code JSON)
  └─▶ main::run()                         [src/main.rs:8]
        └─▶ serde_json::from_str → StatusInput   [src/types.rs:6]
              └─▶ format::build_statusline()      [src/format.rs:33]
                    ├─▶ path_format::format_directory()   [src/path_format.rs:5]
                    ├─▶ todos::get_current_task()          [src/todos.rs:10]
                    │     └── reads ~/.claude/todos/*.json
                    ├─▶ context::compute_usage()           [src/context.rs:17]
                    ├─▶ context::format_token_count()      [src/context.rs:43]
                    ├─▶ context::render_bar()              [src/context.rs:66]
                    └─▶ bridge::write_bridge()             [src/bridge.rs:11]
                           └── writes {tmpdir}/claude-ctx-{session_id}.json
stdout ◀── formatted ANSI string          [src/main.rs:5]
```

Errors from `run()` are swallowed via `unwrap_or_default()` — the binary always exits 0 and prints an empty string on failure.

## Key Patterns

**Single-responsibility modules.** Each `src/*.rs` file has one narrow job. `format.rs` is the sole orchestrator; all other modules are pure functions or I/O helpers.

**Testable-by-injection.** Modules that touch the filesystem expose a `pub(crate)` `_from(base_dir: &Path, ...)` variant alongside the public function that resolves real paths. This avoids test pollution without mocking frameworks:
- `bridge::write_bridge_to()` — `src/bridge.rs:17`
- `todos::get_current_task_from()` — `src/todos.rs:17`

**Best-effort side-effects.** `bridge::write_bridge()` uses atomic rename (`write → .tmp → rename`) and silently discards all errors — the statusline output is never blocked by bridge failures (`src/bridge.rs:52-57`).

**Stdin size cap.** Input is capped at 1 MiB (`take(1_048_576)`) to prevent runaway memory usage from unexpected large inputs (`src/main.rs:13`).

**Release profile aggressively optimised.** `Cargo.toml` configures `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` for minimal binary size.

### Prescriptive Guidance

- **New modules:** create `src/<module>.rs`, add `pub mod <module>;` to `src/lib.rs`. Keep each module to a single concern. If the module touches filesystem paths, add a `pub(crate) fn <name>_from(base: &Path, ...)` variant for testability.
- **New statusline segments:** add logic inside `format::build_statusline()` in `src/format.rs`. Do not add I/O or computation directly there — delegate to a new module function.
- **New structs:** add to `src/types.rs`. Always derive `Default` and use `Option<T>` for fields that may be absent in JSON; add `#[serde(default)]` at struct level to tolerate missing keys gracefully.
- **New entry points:** there are none — this is a single-invocation CLI. Do not add async runtimes or long-lived processes.
- **Error handling:** follow the existing pattern — return `Result<_, Box<dyn std::error::Error>>` from internal functions; `main()` uses `unwrap_or_default()` so the binary always exits 0.

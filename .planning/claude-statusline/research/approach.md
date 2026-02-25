# Approach Research

> Task: Create a Claude Code statusline binary in Rust that reads JSON from stdin, outputs a formatted terminal statusline with model, task, directory (last 3 path levels), and context usage (percentage + token count + bar graph). Replaces existing Node.js implementation at `~/.claude/hooks/gsd-statusline.js` minus GSD update checking.
> Last researched: 2026-02-25T00:05:48Z

## Viable Approaches

### Approach 1: Minimal Dependencies (Raw ANSI + serde_json)

- **What:** Use `serde` + `serde_json` for JSON parsing and write ANSI escape codes as raw string literals. No terminal color crate.
- **How:** Define a typed struct matching the expected JSON input via `#[derive(Deserialize)]`. Read stdin with `std::io::read_to_string`. Format output with `\x1b[...]` sequences in `format!()` macros. Write bridge file with `std::fs::write`. Read todo files with `std::fs::read_dir` + `std::fs::metadata` for sorting by mtime.
- **Pros:**
  - Minimal dependency tree: only `serde` and `serde_json` (both battle-tested, 614M+ downloads)
  - Fastest compile times
  - Smallest binary size (fewer dependencies to link)
  - The existing Node.js code already uses raw escape codes (`\x1b[32m`, `\x1b[5;31m`, etc.) so the translation is 1:1
  - No abstraction overhead for something this simple
- **Cons:**
  - ANSI codes as string literals are less readable than named APIs
  - No Windows console API support (irrelevant for this use case -- macOS/Linux tmux statusline)
- **Best when:** The tool has a narrow, well-defined output format with known escape sequences (this case exactly)
- **Sources:** [serde_json docs](https://docs.rs/serde_json), [serde_json GitHub](https://github.com/serde-rs/json), [Rust JSON ecosystem analysis](https://ecton.dev/rust-json-ecosystem/)

### Approach 2: serde_json + Terminal Color Crate (ansi_term or colored)

- **What:** Use `serde_json` for parsing and a terminal color crate (`ansi_term`, `colored`, or `owo-colors`) for ANSI output.
- **How:** Same JSON parsing as Approach 1, but use crate APIs like `Color::Green.paint("text")` or `"text".green()` instead of raw escape codes.
- **Pros:**
  - More readable color code (`Color::Green.paint(bar)` vs `\x1b[32m{bar}\x1b[0m`)
  - Crate handles reset codes automatically, reducing bugs from mismatched sequences
- **Cons:**
  - Extra dependency for ~10 color calls in the entire program
  - `ansi_term` is unmaintained (last release 2021); `colored` adds 2 transitive deps; `owo-colors` is zero-dep but less known
  - The existing code uses specific sequences like `\x1b[38;5;208m` (256-color orange) and `\x1b[5;31m` (blinking red) which some color crates don't expose cleanly
  - Adds compile time for marginal benefit
- **Best when:** The project has complex or dynamic color requirements, or readability is paramount in a larger team
- **Sources:** [ansi_term crate](https://crates.io/crates/ansi_term), [ANSI Terminal - Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/cli/ansi_terminal.html)

### Approach 3: simd-json for High-Performance Parsing

- **What:** Replace `serde_json` with `simd-json` for SIMD-accelerated JSON parsing.
- **How:** Use `simd_json::from_slice` instead of `serde_json::from_str`. Requires `&mut [u8]` input (simd-json mutates the input buffer).
- **Pros:**
  - Up to 3x faster on large JSON payloads
- **Cons:**
  - **1.6x slower than serde_json on small objects** -- the statusline JSON payload is tiny (~200-500 bytes)
  - Requires mutable input buffer (API ergonomics penalty)
  - Larger dependency tree with platform-specific SIMD code
  - Adds binary size and compile time
  - Overkill for a tool that parses one small JSON blob per invocation
- **Best when:** Parsing large JSON documents (MB+) where SIMD throughput matters
- **Sources:** [simd-json benchmarks](https://github.com/simd-lite/simd-json), [Rust JSON ecosystem analysis](https://ecton.dev/rust-json-ecosystem/), [Rust JSON parsing benchmarks](https://github.com/AnnikaCodes/rust-json-parsing-benchmarks)

## Key Design Decisions

### Sync vs Async I/O

**Use synchronous I/O.** This is a run-once CLI tool that reads stdin to EOF, does computation, writes to stdout. There is zero concurrency requirement. Async (tokio) would add ~2MB to binary size, increase compile time significantly, and provide no benefit. Tokio's stdin implementation actually spawns a blocking thread internally anyway. The entire program completes in <1ms.

Source: [Tokio stdin docs](https://docs.rs/tokio/latest/tokio/io/struct.Stdin.html) -- "This handle is best used for non-interactive uses, such as when a file is piped into the application."

### Project Structure

For a tool this small (~200-300 lines), a flat `src/main.rs` is appropriate. If it grows, split into:

```
src/
  main.rs          -- entry point, stdin read, stdout write
  context.rs       -- context window calculation and bar graph
  bridge.rs        -- bridge file writing
  todos.rs         -- todo file reading
  types.rs         -- serde input/output structs
```

But starting with `main.rs` only is the pragmatic choice. Extract modules only when the file exceeds ~300 lines or when unit testing demands it.

Source: [Cargo Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)

### Bridge File Writing

Use `std::fs::write` for atomic-enough writes (the existing Node.js uses `writeFileSync` which is the same semantics). The bridge file is best-effort -- wrap in a match/if-let and silently ignore errors, matching the existing behavior. Use `std::env::temp_dir()` for cross-platform tmpdir resolution.

### Directory Display (New Behavior)

The task specifies showing last 3 path levels with `...` prefix. Implementation: split path by separator, if components > 3, take last 3 and prepend `...`. Use `std::path::Path::components()` for correct cross-platform splitting.

### Context Display (New Behavior)

Show percentage AND token count: `15% (30.5k)`. Format tokens as `{n}k` with one decimal when >= 1000, raw number otherwise. This is pure string formatting -- no special crate needed.

### Cargo.toml for Minimal Binary Size

```toml
[package]
name = "claude-statusline"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

Expected release binary size: ~500KB-1MB (vs ~60MB for Node.js runtime).

Source: [min-sized-rust](https://github.com/johnthagen/min-sized-rust), [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)

## Recommendation

**Approach 1: Minimal Dependencies (Raw ANSI + serde_json).**

Rationale:
1. The existing Node.js code uses raw ANSI codes -- direct 1:1 translation is simplest and least error-prone
2. The JSON payload is tiny, making serde_json the fastest parser option (1.6x faster than simd-json on small objects)
3. A color crate adds dependency weight for ~10 color format calls -- not worth it
4. Sync I/O is the only sensible choice for a run-once stdin-to-stdout tool
5. Two dependencies total (serde + serde_json) keeps compile time fast and binary small
6. The tool's entire purpose is to be fast and lightweight -- replacing a Node.js script that requires a 60MB+ runtime

## Open Questions

1. **Token count field name:** The task description mentions `context_window.used_tokens` with "(or similar)". The exact JSON field name needs to be confirmed from Claude Code's actual hook output. The Planner should determine this before implementation -- possibly by inspecting a real stdin payload.
2. **Bridge file consumers:** Are there other tools that read the bridge file (`claude-ctx-{session}.json`)? If so, the JSON schema must remain backward-compatible. The existing Node.js writes `session_id`, `remaining_percentage`, `used_pct`, `timestamp`.
3. **Installation method:** Should the binary be installed via `cargo install --path .`, copied to a PATH directory, or referenced by absolute path in the Claude hook config? This affects whether we need a `Makefile` or install script.
4. **Cross-compilation:** Is the binary needed only on macOS (current platform), or also Linux? This affects CI setup but not the code itself.

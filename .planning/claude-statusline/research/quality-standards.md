# Quality & Standards Research

> Task: Create a Claude Code statusline binary in Rust that reads JSON from stdin and outputs a formatted terminal statusline. Performance-critical tool invoked on every Claude Code interaction.
> Last researched: 2026-02-25T00:06:24Z

## Performance

### Startup Time Target

**Target: < 5ms cold start** (vs ~30ms Node.js overhead being replaced).

- Bare Rust binaries achieve ~0.5ms startup on modern hardware ([startup-time benchmarks](https://github.com/bdrung/startup-time))
- Real-world Rust CLIs with minimal dependencies achieve < 5ms ([Actionbook CLI example](https://www.xugj520.cn/en/archives/actionbook-cli-browser-automation-rust.html))
- Key risk: heavy dependencies (e.g., `reqwest`, `tokio`) can push startup to 100ms+. This tool does synchronous I/O only, so async runtimes are unnecessary and must be avoided

### Binary Size Target

**Target: < 2MB stripped release binary.**

Cargo.toml release profile for size + speed balance:

```toml
[profile.release]
opt-level = "z"        # Optimize for size (alternative: "s" for slightly more speed)
lto = "fat"            # Whole-program link-time optimization
codegen-units = 1      # Single codegen unit for max optimization
panic = "abort"        # No unwinding overhead
strip = "symbols"      # Remove debug symbols
```

Source: [Rust Performance Book - Build Configuration](https://nnethercote.github.io/perf-book/build-configuration.html), [Rust Project Primer - Size](https://rustprojectprimer.com/building/size.html)

Additional measures:
- Use `cargo-bloat` to audit per-crate size contribution ([cargo-bloat](https://github.com/RazrFalcon/cargo-bloat))
- Prefer `serde` + `serde_json` with minimal features over heavier alternatives
- Avoid pulling in `chrono` (large); use `std::time` or lightweight alternatives
- Consider `opt-level = "s"` if benchmarks show `"z"` hurts runtime performance noticeably

### Benchmarking Strategy

Use [hyperfine](https://github.com/sharkdp/hyperfine) for end-to-end CLI benchmarking:

```bash
# Compare against Node.js version (if it exists)
hyperfine --warmup 3 -N 'echo "{}" | ./target/release/claude-statusline'

# Compare Rust vs Node with identical input
hyperfine --warmup 3 -N \
  'echo "$JSON_INPUT" | ./target/release/claude-statusline' \
  'echo "$JSON_INPUT" | node ./index.js'
```

Key hyperfine flags:
- `-N` / `--shell=none`: eliminates shell startup overhead for sub-5ms commands
- `--warmup 3`: warm disk caches
- `--export-markdown`: generate comparison tables for CI

For micro-benchmarks of internal functions (context calculation, bar rendering), use Rust's built-in `#[bench]` or the `criterion` crate. Given the simplicity of the functions, `#[bench]` is sufficient.

## Security

### Input Validation

- **stdin JSON**: Untrusted input. Use `serde_json::from_reader` with typed deserialization (not `Value`) to reject malformed input structurally. Limit read size to prevent memory exhaustion (e.g., cap at 1MB of stdin)
- **File paths**: The tool reads `~/.claude/todos/` and writes to a tmpdir. Use `dirs::home_dir()` or `std::env::var("HOME")` for home directory resolution. Validate that resolved paths are within expected directories to prevent path traversal
- **Environment variables**: `TMPDIR` is read for bridge file location. Validate it resolves to an existing directory

### File System Safety

- **Bridge file writes**: Use atomic write pattern (write to temp file, then rename) to prevent partial reads by consumers
- **Todo file reads**: Handle missing directory, empty directory, unreadable files gracefully
- **No network access**: This tool has no network surface area, which limits the attack surface significantly

### Dependency Audit

- Run `cargo audit` in CI to check for known vulnerabilities in dependencies
- Minimal dependency tree reduces supply chain risk. Target dependencies: `serde`, `serde_json`, `dirs` (or `std::env` for home dir)

## Accessibility

Not applicable (no UI changes). This tool outputs ANSI escape sequences to a terminal. However:

- ANSI color output should degrade gracefully when `NO_COLOR` env var is set ([no-color.org](https://no-color.org/)) or when stdout is not a TTY
- Use `std::io::stdout().is_terminal()` (stable since Rust 1.70) to detect non-TTY contexts
- This is a display convention, not an a11y requirement, but improves interoperability with log capture and piping

## Testing Strategy

### Test Types Needed

| Type | Scope | Tool |
|------|-------|------|
| Unit tests | Pure functions: context calculation, bar graph rendering, color threshold logic, path formatting | `#[cfg(test)]` inline modules |
| Integration tests | Full binary: stdin -> stdout with known inputs | `assert_cmd` + `predicates` crates |
| Benchmark tests | Startup time, throughput | `hyperfine` (CLI), optionally `criterion` |

### Key Test Cases

**Context calculation (unit)**
- 0% usage -> empty bar
- 50% usage -> half bar
- 80% usage (threshold) -> full bar, color change
- > 80% usage -> clamped, warning color
- Missing or null context fields -> graceful default

**Bar graph rendering (unit)**
- Known percentage -> exact character sequence
- Boundary values: 0%, 1%, 49%, 50%, 79%, 80%, 100%
- Verify ANSI color codes at threshold boundaries (green/yellow/red)

**JSON parsing (unit)**
- Valid complete JSON -> correct struct
- Valid JSON with missing optional fields -> defaults applied
- Invalid JSON -> error result (not panic)
- Empty stdin -> error result (not panic)
- Extremely large JSON (> 1MB) -> rejected or truncated

**Todo file reading (unit/integration)**
- Directory exists with files -> parsed correctly
- Directory missing -> empty list, no error
- Directory exists but empty -> empty list
- Files with unexpected content -> skipped gracefully

**Bridge file writing (integration)**
- File written to expected tmpdir path
- File contains valid content
- Tmpdir doesn't exist -> error handled (not panic)

**Full pipeline (integration with `assert_cmd`)**
- Pipe known JSON via stdin, assert stdout contains expected ANSI output
- Pipe invalid JSON, assert process exits with code 0 (silent failure) and outputs fallback/empty statusline
- Verify `NO_COLOR` env var strips ANSI codes

### Mocking Approach

- **stdin**: Use `assert_cmd`'s `.write_stdin()` for integration tests. For unit tests, accept `impl Read` parameter instead of hardcoding `std::io::stdin()`
- **File system**: Use `tempfile::tempdir()` for bridge file write tests. For todo reading, create temp directories with known test fixtures
- **Environment variables**: Use `std::env::set_var` in tests (with `#[serial]` from `serial_test` crate if tests share env state)
- **Real implementations preferred for**: JSON parsing (serde is deterministic), ANSI formatting (string comparison is sufficient)

### Existing Test Patterns

This is a greenfield project (empty repo), so no existing patterns to follow. Recommended structure:

```
src/
  main.rs          # Entry point, thin: parse args, call lib
  lib.rs           # Core logic, all testable functions
  context.rs       # Context usage calculation + bar rendering
  todo.rs          # Todo file reading
  bridge.rs        # Bridge file writing
  format.rs        # ANSI formatting + statusline assembly
tests/
  integration.rs   # assert_cmd-based full binary tests
```

Dev dependencies:
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
serde_json = "1"  # for constructing test inputs
```

Source: [Rust CLI Book - Testing](https://rust-cli.github.io/book/tutorial/testing.html), [assert_cmd docs](https://docs.rs/assert_cmd)

## Error Handling Strategy

**Core principle: never break the statusline.** The tool must always exit 0 and output *something* (even empty string) regardless of input errors. This matches the Node.js version's behavior.

Implementation pattern:

```rust
fn main() {
    // Catch all errors at top level, output fallback on any failure
    let output = match run() {
        Ok(line) => line,
        Err(_) => String::new(), // or minimal fallback statusline
    };
    print!("{}", output);
}

fn run() -> Result<String, Box<dyn std::error::Error>> {
    // All fallible operations use ? operator
    // Specific error types via thiserror if warranted,
    // but anyhow or Box<dyn Error> is fine for this scope
}
```

- Use `Result` throughout internal functions, propagate with `?`
- Top-level `main()` catches all errors and outputs fallback
- No `unwrap()` or `expect()` on user-controlled data paths
- `panic = "abort"` in release profile means panics terminate immediately (no unwinding), but panics should never reach production paths

Source: [Error Handling in Rust CLI Apps](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)

## Cross-Platform Considerations

| Concern | macOS (primary) | Linux (secondary) | Mitigation |
|---------|----------------|-------------------|------------|
| Home directory | `/Users/<name>` | `/home/<name>` | Use `dirs::home_dir()` or `std::env::var("HOME")` |
| Temp directory | `/var/folders/...` or `$TMPDIR` | `/tmp` or `$TMPDIR` | Use `std::env::temp_dir()` |
| Path separators | `/` | `/` | Not an issue (both Unix) |
| Terminal capabilities | iTerm2, Terminal.app | varies widely | Respect `NO_COLOR`, `TERM` |
| Binary target | `aarch64-apple-darwin`, `x86_64-apple-darwin` | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | CI matrix build |

Source: [dirs crate](https://crates.io/crates/dirs)

## CI/CD for Release Binaries

Recommended GitHub Actions workflow:

**Matrix targets:**
- `aarch64-apple-darwin` (Apple Silicon - primary)
- `x86_64-apple-darwin` (Intel Mac)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64, optional)

**CI pipeline stages:**
1. `check` - `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`
2. `test` - `cargo test` on all supported targets
3. `build` - release builds with optimized profile
4. `benchmark` - hyperfine comparison (optional, on tag/release only)
5. `release` - create GitHub Release with binary assets on version tags

**Key actions:**
- `dtolnay/rust-toolchain@stable` for toolchain setup
- `actions/cache@v4` for `~/.cargo` and `target/` caching
- `houseabsolute/actions-rust-cross` for cross-compilation ([source](https://github.com/houseabsolute/actions-rust-cross))
- `softprops/action-gh-release` for release artifact upload
- `cargo audit` for dependency vulnerability checking

**Native vs cross builds:** macOS targets should build natively on `macos-latest` runner (supports both ARM64 and x86_64). Linux targets can cross-compile from Linux runner using `cross` tool.

Source: [Cross-Platform Rust Pipeline](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/), [Deploy Rust Binaries](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)

## Standards Checklist

1. Binary startup time < 5ms (measured with `hyperfine -N`)
2. Binary size < 2MB (stripped release build)
3. All JSON parsing uses typed deserialization, not `serde_json::Value`
4. No `unwrap()`/`expect()` on any user-input-derived data path
5. Process always exits 0 and produces output (even on error)
6. `NO_COLOR` environment variable respected
7. TTY detection prevents ANSI output to non-terminals
8. `cargo clippy -- -D warnings` passes with zero warnings
9. `cargo fmt --check` passes
10. `cargo audit` shows no known vulnerabilities
11. Unit test coverage for all context calculation boundary values (0%, 1%, 50%, 79%, 80%, 100%)
12. Integration tests verify stdin-to-stdout pipeline with `assert_cmd`
13. Bridge file uses atomic write (write tmp + rename)
14. stdin read capped at reasonable size (1MB) to prevent memory exhaustion
15. Release profile includes `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`

# claude-statusline

A fast Rust binary that reads Claude Code JSON from stdin and writes a formatted ANSI statusline to stdout.

## Install

### Pre-built binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/jamesacarr/claud-statusline/releases):

| Platform      | Archive                        |
|---------------|--------------------------------|
| Linux (x64)   | `claud-statusline-linux-x64.tar.gz`   |
| Linux (arm64)  | `claud-statusline-linux-arm64.tar.gz` |
| macOS (arm64)  | `claud-statusline-macos-arm64.tar.gz`|
| Windows (x64)  | `claud-statusline-windows-x64.zip`   |

Extract the binary and place it somewhere on your `PATH`.

### Cargo

```sh
cargo install --git https://github.com/jamesacarr/claud-statusline.git
```

### From source

```sh
cargo install --path .
```

Or build the release binary directly:

```sh
cargo build --release
```

## Usage

Add `claude-statusline` as a [custom status line](https://code.claude.com/docs/en/statusline) in your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "statusLine": {
    "type": "command",
    "command": "claude-statusline"
  }
}
```

Claude Code pipes JSON session data (model, context window, costs, etc.) to the command via stdin and displays whatever it prints to stdout.

## Development

```sh
make help      # Show all targets
make build     # Build debug binary
make test      # Run all tests
make fmt       # Format code
make lint      # Run clippy linter
make check     # Check formatting and linting
```

Requires Rust 1.85+.

## License

MIT

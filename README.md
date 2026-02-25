# claude-statusline

A fast Rust binary that reads Claude Code JSON from stdin and writes a formatted ANSI statusline to stdout.

## Install

```sh
cargo install --path .
```

Or build a release binary:

```sh
make release
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

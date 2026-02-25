use std::io::Read;

fn main() {
    let output = match run() {
        Ok(line) => line,
        Err(_) => String::new(),
    };
    print!("{}", output);
}

fn run() -> Result<String, Box<dyn std::error::Error>> {
    // Read stdin with 1MB cap
    let mut input = String::new();
    std::io::stdin().lock().take(1_048_576).read_to_string(&mut input)?;

    // Parse JSON
    let data: claude_statusline::types::StatusInput = serde_json::from_str(&input)?;

    // Check NO_COLOR -- presence of the variable (any value including empty) disables color
    // Do NOT check is_terminal() -- Claude Code pipes stdin/stdout and renders ANSI itself
    let no_color = std::env::var("NO_COLOR").is_ok();

    // Build statusline
    Ok(claude_statusline::format::build_statusline(&data, no_color))
}

#[cfg(test)]
mod tests {
    #[test]
    fn run_returns_error_on_empty_stdin() {
        // run() reads from stdin which is empty in test context,
        // causing serde_json::from_str("") to fail with a parse error
        let result = super::run();
        assert!(result.is_err(), "run() should return Err when stdin has no valid JSON");
    }

    #[test]
    fn main_does_not_panic_when_run_fails() {
        // The main function catches errors from run() and prints empty string.
        // In unit tests stdin is empty, so run() will fail, but main() should not panic.
        super::main();
    }
}

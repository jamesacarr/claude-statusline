use std::io::Read;

fn main() {
    let output = run().unwrap_or_default();
    print!("{}", output);
}

fn run() -> Result<String, Box<dyn std::error::Error>> {
    // Read stdin with 1MB cap
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .take(1_048_576)
        .read_to_string(&mut input)?;

    // Parse JSON
    let data: claude_statusline::types::StatusInput = serde_json::from_str(&input)?;

    // Check NO_COLOR -- presence of the variable (any value including empty) disables color
    // Do NOT check is_terminal() -- Claude Code pipes stdin/stdout and renders ANSI itself
    let no_color = std::env::var("NO_COLOR").is_ok();

    // Build statusline
    Ok(claude_statusline::format::build_statusline(&data, no_color))
}

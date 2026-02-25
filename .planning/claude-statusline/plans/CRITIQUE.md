# Plan Critique

> Task: claude-statusline
> Reviewed: 2026-02-25T01:15:23Z
> Verdict: no objections

## Objections

None. All 5 objections from the previous critique (token count field selection, Rust edition minimum version, output_tokens omission, TTY detection risk, imprecise integration test assertions) were accepted and addressed in the revision.

## Observations

- **`TodoItem.active_form` typed as `String` instead of `Option<String>`**: Task 1.2 defines `active_form: String` in `TodoItem`, while NFR 5 mandates "all input struct fields use `Option<T>` with `#[serde(default)]`". `TodoItem` is deserialized from todo files (user input). With `#[serde(default)]`, a missing `activeForm` field would deserialize to an empty string rather than `None`, which would cause `get_current_task` to return `Some("")` instead of `None` for a malformed todo item. The executor should consider making this `Option<String>` for consistency, but in practice `activeForm` has always been present in observed data and an empty string would render as an empty bold segment (cosmetic only, not a crash).
- **Missing `.gitignore`**: The plan creates all project files manually but does not include a `.gitignore` to exclude `/target/`. An executor using `cargo init` would get one automatically, but the plan's Task 1.2 creates files individually. The `target/` directory could be accidentally committed. The executor should add a `.gitignore` with `/target/` and `Cargo.lock` is typically committed for binary crates.
- **Task 1.1 verification depends on Task 1.2**: Task 1.1's verification command (`cargo check`) cannot succeed until Task 1.2 creates `src/main.rs`. The plan notes this parenthetically "(after Task 1.2 completes the source files)" but since both tasks are in Wave 1 (parallel execution), the executor must understand that verification for 1.1 should be deferred until 1.2 completes. This is documented but could be clearer if verification were "Cargo.toml exists with all specified fields" (a file content check) rather than a build check.
- **No test for >100% `used_percentage`**: The risks research (risks-edge-cases.md) documents that `used_percentage` can exceed 100 due to a cumulative token bug (issue #13783). The plan clamps to 0-100, which is the correct behavior matching the JS, but there is no unit test or integration test for this edge case. Adding a test with `used_percentage: 105` asserting it clamps to 100 would improve coverage.
- **`format_token_count` at scale**: For extended 1M-token context windows, total tokens could reach ~200k+, displaying as `200.0k`. The plan handles this correctly but has no test case above 20k tokens. A test with `total_input_tokens: 150000, total_output_tokens: 50000` -> `"200.0k"` would verify large values.

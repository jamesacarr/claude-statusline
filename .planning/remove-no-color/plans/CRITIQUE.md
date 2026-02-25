# Plan Critique

> Task: remove-no-color
> Reviewed: 2026-02-25T21:35:00Z
> Verdict: has objections

## Objections

### Objection 1: Task 3.2 deletes the TERM=dumb tech debt entry instead of updating it

- **Category:** internal-consistency
- **Severity:** medium
- **Affected tasks:** Task 3.2
- **Evidence:** CONCERNS.md line 10 documents `TERM=dumb` as unhandled tech debt: "`main.rs` explicitly checks `NO_COLOR` but never checks `TERM=dumb`. Terminals declaring `dumb` do not support ANSI codes." The research file `risks-edge-cases.md` identifies "Terminals that do not support ANSI (e.g., `TERM=dumb`)" as a risk of always-on ANSI output. Task 3.2 action says: "In the Tech Debt table, delete the entire `TERM=dumb` row (line 10)."
- **Problem:** The `TERM=dumb` tech debt entry contains two concerns: (1) `NO_COLOR` is checked but `TERM=dumb` is not, and (2) terminals declaring `dumb` cannot handle ANSI codes. After removing `NO_COLOR`, concern (1) is moot but concern (2) becomes more relevant -- the binary now always emits ANSI with no escape hatch at all. Deleting the row entirely loses institutional knowledge about a separate, still-valid concern. An executor following this plan would silently remove awareness that `TERM=dumb` is unhandled.
- **Suggestion:** Rewrite the `TERM=dumb` row instead of deleting it. Updated description: "`TERM=dumb` not handled -- the binary always emits ANSI codes. Terminals declaring `dumb` do not support ANSI. Since output is consumed by Claude Code (which renders ANSI internally), this is acceptable for the primary use case but would cause garbled output if piped to a dumb terminal." Keep severity as `low`. Update the Files column to `src/main.rs` (remove the line range `19-21` since those lines are deleted).

### Objection 2: Wave 2 tasks have an ordering dependency that violates wave parallelism

- **Category:** internal-consistency
- **Severity:** medium
- **Affected tasks:** Task 2.2, Task 2.3
- **Evidence:** Plan schema states: "Tasks within a wave are independent and may run in parallel." Task 2.2 removes integration Test 6 from `tests/integration.rs` and verifies with `make test`. Task 2.3 runs `make lint` (which per CONVENTIONS.md runs `cargo clippy --all-targets -D warnings`). Both tasks are in Wave 2.
- **Problem:** Task 2.3's `make lint` will succeed regardless of Task 2.2's execution order (the dead Test 6 compiles fine). However, the larger concern is that `make lint` runs against a codebase that may still contain the stale Test 6. A lint "pass" before Task 2.2 completes means the lint result does not reflect the final state of the codebase. If Task 2.2 introduces an error during the test deletion (e.g., accidentally deleting too many lines), the lint pass from Task 2.3 is stale and misleading. Moving `make lint` to after all code changes ensures the lint gate validates the complete, final codebase.
- **Suggestion:** Move Task 2.3 (`make lint`) into a new Wave 3 that runs after Wave 2. Rename the current Wave 3 (documentation) to Wave 4. This makes `make lint` a true gate between code changes and documentation, verifying the entire cleaned codebase compiles and lints cleanly before moving to docs.

## Observations

- All 30+ line number references in the plan were verified against the current source files and are accurate. This level of precision significantly reduces executor error risk.
- The research (quality-standards.md, testing section) suggests adding a test that verifies `NO_COLOR=1` is truly ignored after removal. The plan omits this. This is defensible -- existing integration tests already verify ANSI output is present (Tests 1, 9-11 assert `\x1b[` sequences in output), which implicitly proves `NO_COLOR` has no effect. Not raised as an objection.
- `bold()` is defined as a public function but never called from production code (`build_statusline` only calls `dim()` and `render_bar()`). The plan correctly updates `bold()` anyway since it has the `no_color` parameter in its public signature. No action needed.
- Task 3.2 touches 4 files in a single task. While no file overlaps with Task 3.1, the 4-file scope makes the task harder to debug if one edit fails. Splitting per-file would improve retry granularity but is not worth the added complexity for documentation-only changes.

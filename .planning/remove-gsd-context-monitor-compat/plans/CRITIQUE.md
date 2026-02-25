# Plan Critique

> Task: remove-gsd-context-monitor-compat
> Reviewed: 2026-02-25T03:15:00Z
> Verdict: no objections

## Objections

None.

## Observations

1. **Previous objection resolved.** The prior critique flagged missing wave-level `Status:` fields. The current plan includes `Status: pending` on all three waves. This is resolved.

2. **CONCERNS.md "Do Not Touch" entry becomes stale.** The codebase map file `.planning/codebase/CONCERNS.md` line 40 lists `src/bridge.rs:23-29` as a "Do Not Touch" area. After bridge.rs is deleted, this entry is meaningless. This is outside the plan's scope (codebase maps are regenerated separately), but worth noting for the next mapping run.

3. **Intermediate compilation failure between waves is intentional and acceptable.** After Wave 1 removes `pub mod bridge;` from `lib.rs` but before Wave 2 removes `use crate::bridge;` from `format.rs`, the codebase will not compile. The plan correctly defers compilation verification to Wave 3. An executor should not attempt `cargo build` between waves.

4. **Research identifies external-system impact not addressed in plan.** The `risks-edge-cases.md` research notes that removing bridge writing silently breaks the `gsd-context-monitor.js` PostToolUse hook registered in `~/.claude/settings.json`. The plan does not include a task to remove or update that hook registration. This is not an objection because (a) the task description explicitly states the compatibility is "unnecessary and should be fully stripped out" -- the user has accepted this tradeoff, and (b) the hook gracefully exits when no bridge file is found, so there is no runtime error. However, the user may want to clean up the stale hook config separately.

5. **Line count discrepancy is trivial.** The plan's Task 1.1 says `bridge.rs` contains "174 lines" while the actual file has 175 lines (including the trailing newline). This has no impact on execution since the entire file is being deleted.

6. **Research inconsistency on test count.** The `approach.md` research doc states bridge.rs contains "6 unit tests" while the actual source and `quality-standards.md` both show 7. The plan correctly uses 7. No action needed.

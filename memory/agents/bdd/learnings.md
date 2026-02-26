# Learnings: bdd

## Codebase Patterns
- BDD framework: cucumber-rs (crate `cucumber` 0.21) with Gherkin `.feature` files
- Feature files live in `tests/features/`, step definitions in `tests/bdd/steps/`
- Harness entry point: `tests/bdd/main.rs` with `harness = false` in Cargo.toml
- Steps shell out to `tk` binary via assert_cmd `Command::cargo_bin("tk")`, not direct Rust function calls
- World struct holds: TempDir, db_path, last_stdout/stderr/exit_code, task_ids HashMap

## Gotchas
- `--json` flag is essential for step assertions — parse structured JSON, don't scrape table formatting
- `TACKS_DB` env var overrides default `.tacks/tacks.db` path — use this in steps to point at temp DB
- Feature files serve dual purpose: executable tests AND agent-readable behavioral documentation
- Agents can `cat tests/features/ready.feature` to understand command behavior before calling it
- `last_stdout` holds only the most recent `tk` invocation — any Then step that calls tk internally overwrites it. Keep Then steps that call tk and Then steps that read last_stdout as distinct responsibilities (added: 2026-02-25, dispatch: tacks-5sl.22)
- cucumber-rs 0.21 requires tokio: use `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }` and `#[tokio::main]` on harness main (added: 2026-02-25, dispatch: tacks-5sl.22)
- `assert_cmd::Command::cargo_bin` has deprecation warnings (cosmetic) — suppress with `#[allow(deprecated)]` or upgrade when stable replacement lands (added: 2026-02-25, dispatch: tacks-5sl.22)
- Status enum needed `#[serde(rename_all = "snake_case")]` to serialize as "open"/"in_progress" not "Open"/"InProgress" — always check enum serde config (added: 2026-02-25, dispatch: tacks-5sl.22)

## Preferences
- Feature files written BEFORE implementation for new features (true BDD red-green cycle)
- Retroactive feature files complete: task_lifecycle (5), dependencies (5), agent_commands (7), filtering (5) = 22 scenarios
- Organize by behavior area, not by command: task_lifecycle.feature, dependencies.feature, agent_commands.feature, filtering.feature
- Test both human-readable and JSON output modes
- `#[step]` does not exist as attribute macro in cucumber-rs 0.21 — register under both `#[given]` and `#[when]` with distinct fn names when step needs to work after either keyword (added: 2026-02-25, dispatch: tacks-5sl.23)
- `#![allow(deprecated)]` at top of step module files suppresses cosmetic cargo_bin deprecation across entire module (added: 2026-02-25, dispatch: tacks-5sl.23)
- When a Given block sets up tasks and closes one, `And I close the task` parses as a given step — structure registrations accordingly (added: 2026-02-25, dispatch: tacks-5sl.23)

## Cross-Agent Notes
- Coordinate with core-dev — they implement commands, bdd writes the feature file first
- Phase 2 priority: set up harness first, then retroactive features for existing 9 commands + 3 new Phase 1 commands

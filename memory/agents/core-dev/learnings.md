# Learnings: core-dev

## Codebase Patterns
- CLI uses clap derive macros in `src/main.rs` with one file per subcommand in `src/commands/`
- Each command's `run()` function takes individual params (not a struct), prints to stdout, returns `Result<(), String>`
- Commands follow a consistent pattern: open DB, call DB method, format output (table or JSON based on `--json` flag)
- All commands support `--json` for machine-readable output — this is critical for AI agent consumption

## Gotchas
- No TaskType column — epic/task/bug are distinguished by tags, not a type field (meeting decision)
- When `--parent <id>` is used in create, auto-add `epic` tag to the parent if not present
- Tags are stored as comma-separated text; query pattern is `WHERE (',' || tags || ',') LIKE '%,tag,%'`
- `task_count_by_tag` splits CSV in Rust rather than SQL — consistent with how tags are handled elsewhere (added: 2026-02-25, dispatch: tacks-5sl.8)
- `get_ready_tasks` limit is injected as literal u32 into SQL string (safe since from clap, not user string input) (added: 2026-02-25, dispatch: tacks-5sl.18)
- Cargo.toml `[[test]]` entries must have their main.rs present or entire manifest fails to parse — check tests/ for missing entry points early (added: 2026-02-25, dispatch: tacks-5sl.8)
- Stats --oneline uses SQLite GROUP BY order (alphabetical) — if canonical status ordering (open, in_progress, blocked, done) is wanted, add sort step in stats.rs (added: 2026-02-25, dispatch: tacks-5sl.8)

## Preferences
- Phase 1 compatibility complete: stats, prime, ready --limit all implemented
- close_reason is a structured nullable column with enum (done/duplicate/absorbed/stale/superseded), not a comment
- notes field is mutable working context (overwrites), distinct from append-only comments

## Cross-Agent Notes
- Coordinate with db agent on any new columns — migration system is now in place
- Coordinate with bdd agent — new commands need feature files written BEFORE implementation (BDD cycle)
- The `--json` flag is global (cli.json) — new commands never need to declare it on the subcommand variant (added: 2026-02-25, dispatch: tacks-5sl.17)
- Silent-exit for missing DB: check `db_path.exists()` before `Database::open()`, return `Ok(())` immediately (added: 2026-02-25, dispatch: tacks-5sl.17)
- JSON stats output should always include all four canonical status keys with zero-defaults (added: 2026-02-25, dispatch: tacks-5sl.17)

# Learnings: db

## Codebase Patterns
- Single file: `src/db/mod.rs` owns all schema, migrations, and queries
- SQLite with WAL mode set in `Database::open()`, foreign keys enabled via PRAGMA
- `bundled` feature compiles SQLite from source — no system dependency needed
- `row_to_task` helper deserializes rows — must stay in sync with tasks table schema
- Parameterized queries only via `rusqlite::params!` — never interpolate SQL

## Gotchas
- Migration system implemented: `schema_version` key in config table, sequential `if version < N` blocks in `run_migrations()` (added: 2026-02-25, dispatch: tacks-p50.22)
- `INSERT OR IGNORE` seeds initial schema_version=0 so migrate() is idempotent on re-run (added: 2026-02-25, dispatch: tacks-p50.22)
- `set_schema_version` has `#[allow(dead_code)]` — remove it when first real migration (v1 close_reason) is activated (added: 2026-02-25, dispatch: tacks-p50.22)
- Each migration block wraps DDL in BEGIN/COMMIT for atomicity (added: 2026-02-25, dispatch: tacks-p50.22)
- No rollback support — local-only means delete-and-reinit is acceptable recovery
- Migration v1: `ALTER TABLE tasks ADD COLUMN close_reason TEXT` (nullable)
- Migration v2: `ALTER TABLE tasks ADD COLUMN notes TEXT` (nullable)
- Cycle detection implemented: `would_create_cycle()` is a module-level free fn taking `&Connection`, BFS from parent through blocker edges checking reachability of child (added: 2026-02-25, dispatch: tacks-p50.6)
- Dep directionality: "child depends on parent" means BFS asks "what does current depend on?" via `SELECT parent_id FROM dependencies WHERE child_id = current` (added: 2026-02-25, dispatch: tacks-p50.6)
- Replaced `INSERT OR IGNORE` silent-swallow with explicit duplicate check + plain INSERT — safer and more informative (added: 2026-02-25, dispatch: tacks-p50.6)
- `get_dependents` reverse lookup implemented, `#[allow(dead_code)]` until close guard calls it (added: 2026-02-25, dispatch: tacks-p50.6)

## Preferences
- Cycle detection: reject at write time on `dep add`, not detect-and-report at read time
- Close guard: `get_dependents` reverse lookup needed, warn in close when open dependents exist (--force override)
- Keep migrations simple: sequential functions in db/mod.rs, no migration files, no framework
- Schema version 0 = current schema, version 1 = close_reason, version 2 = notes

## Cross-Agent Notes
- core-dev depends on migration system before any new columns can be wired to commands
- bdd agent needs feature scenarios for cycle detection and migration upgrade paths

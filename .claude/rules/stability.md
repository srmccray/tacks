# Interface Stability Contract

Tacks has downstream consumers (e.g., Tackline) that depend on its CLI interface. All changes must be backwards-compatible.

## What Is Stable

- **CLI commands**: All 16 subcommands (init, create, list, ready, show, update, close, dep, comment, stats, prime, children, epic, blocked) and their flags
- **JSON output schema**: The Task struct (12 fields), Comment, Dependency, Status enum (open/in_progress/done/blocked), CloseReason values (done/duplicate/absorbed/stale/superseded)
- **Global flags**: `--json`, `--db`
- **Exit codes**: 0 for success, 1 for error
- **ID format**: `tk-XXXX` for tasks, `tk-XXXX.N` for subtasks
- **Env var**: `TACKS_DB` override for database path
- **DB schema**: Existing tables (tasks, config, dependencies, comments) and their columns

## Rules

1. **Never remove a command, flag, or output field.** Add new ones freely.
2. **Never rename a JSON field or change its type.** A field that was a string stays a string.
3. **Never change Status or CloseReason enum values.** New values may be added.
4. **New DB columns must be nullable or defaulted.** Never remove or rename existing columns.
5. **New command flags must be optional.** Existing flags keep their behavior.
6. **Human-readable output format may change** (it is not a stable interface), but JSON output is frozen.
7. **Exit codes are stable.** Success is 0, error is 1.

## When You Must Break Compatibility

If a breaking change is truly necessary:
1. Flag it in the commit message with `BREAKING:` prefix
2. Document the migration path for downstream consumers
3. Get explicit approval from the user before proceeding

## Why This Matters

Downstream projects call `tk` commands in hooks, skills, and agent prompts. A renamed flag, removed field, or changed JSON structure silently breaks their workflows. The cost of a breaking change is not just fixing tacks — it is finding and fixing every consumer.

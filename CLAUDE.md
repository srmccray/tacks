# Tacks

Lightweight task manager for AI coding agents. A local-only alternative to [beads](https://github.com/steveyegge/beads) built in Rust with SQLite. Optimized for consumption by tools like Claude Code.

## Project Overview

- **Language**: Rust (edition 2024)
- **Binary**: `tk` (installed via `cargo install`)
- **Storage**: SQLite via rusqlite (bundled), local-only (no git, no sync)
- **CLI framework**: clap (derive)
- **Testing**: BDD with cucumber-rs (Gherkin feature files + assert_cmd)
- **Output**: Human-readable tables (default) or JSON (`--json`, global flag)

## Architecture

```
src/
  main.rs           # CLI definition (clap derive) and dispatch
  models/mod.rs     # Data types: Task, Comment, Dependency, Status, CloseReason
  db/mod.rs         # SQLite database layer (open, migrate, CRUD, cycle detection)
  commands/         # One file per subcommand
    init.rs         # tk init [--prefix]
    create.rs       # tk create <title> [-p priority] [-d desc] [-t tags] [--parent id]
    list.rs         # tk list [-a] [-s status] [-p pri] [-t tag] [--parent id]
    ready.rs        # tk ready [--limit N]
    show.rs         # tk show <id> (includes blockers, dependents, notes, close_reason)
    update.rs       # tk update <id> [fields...] [--claim] [--notes text]
    close.rs        # tk close <id> [-c comment] [-r reason] [--force]
    dep.rs          # tk dep add|remove <child> <parent>
    comment.rs      # tk comment <id> <body>
    stats.rs        # tk stats [--oneline] [--json]
    prime.rs        # tk prime [--json] (AI context output)
    children.rs     # tk children <id> (list subtasks)
    epic.rs         # tk epic (show epic progress)
    blocked.rs      # tk blocked (tasks blocked by open deps)
tests/
  features/         # Gherkin .feature files (BDD specs + agent-readable docs)
    task_lifecycle.feature    dependencies.feature
    agent_commands.feature    filtering.feature
    close_reason.feature      close_guard.feature
    epic_tagging.feature      notes.feature
    children.feature          epic_status.feature
    blocked.feature           parent_filter.feature
  bdd/
    main.rs         # cucumber-rs harness (World struct, runner)
    steps/          # Step definitions (shell out to tk binary via assert_cmd)
```

## Interface Stability

Tacks has downstream consumers (e.g., [Tackline](https://github.com/steveyegge/tackline)) that depend on the CLI. The interface is stable — see `.claude/rules/stability.md` for the full contract. In short: never remove or rename commands, flags, JSON fields, or DB columns. Add freely, break never.

## Key Design Decisions

- **Local-only storage**: No git integration, no sync, no distributed concerns
- **Hash-based IDs**: `tk-a1b2` format (same as beads)
- **Hierarchical IDs**: Subtasks use `parent.N` format (e.g., `tk-a1b2.1`)
- **Tags over types**: Epic/task/bug are tags, not a type column. `epic` tag auto-added on child creation.
- **WAL mode**: SQLite WAL journal for concurrent read safety
- **Version-gated migrations**: `schema_version` in config table, sequential `if version < N` blocks in `run_migrations()`
- **Cycle detection**: Write-time BFS guard on `dep add` rejects circular dependencies
- **No external dependencies**: SQLite is bundled (no system sqlite needed)
- **Env var override**: `TACKS_DB` overrides default `.tacks/tacks.db` path
- **BDD-driven**: Feature files are both executable tests and agent-readable behavioral documentation
- **`--json` is global**: Declared on top-level Cli struct, accessed via `cli.json`

## Build & Test

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test --test bdd    # Run BDD scenarios (49 scenarios, 274 steps)
cargo clippy             # Lint
cargo fmt --check        # Format check
```

## CLI Quick Reference

```bash
tk init                           # Initialize in current dir
tk create "Title" -p 1            # Create P1 task
tk create "Sub" --parent <id>     # Create subtask (auto-tags parent as epic)
tk list                           # Show open tasks
tk list -s done -t backend        # Filter by status, tag
tk list --parent <id>             # Show only children of a task
tk ready                          # Tasks with no blockers
tk ready --limit 1                # Next task for agent to pick
tk show <id>                      # Task details + blockers + dependents
tk update <id> --claim            # Claim task (in_progress + assignee)
tk update <id> --notes "context"  # Set working notes (overwrites)
tk close <id> -c "Done"           # Close with comment
tk close <id> -r duplicate        # Close with reason (done/duplicate/absorbed/stale/superseded)
tk close <id> --force             # Close even with open subtasks
tk dep add <child> <parent>       # Add blocker (cycle-checked)
tk comment <id> "message"         # Add comment
tk children <id>                  # List subtasks of a task
tk epic                           # Show epic progress (completion stats)
tk blocked                        # List tasks blocked by open deps
tk stats                          # Backlog overview (status/priority/tag counts)
tk stats --oneline                # Compact: "3 open, 2 in_progress, 5 done"
tk prime                          # AI context: stats + in-progress + ready queue
```

All commands support `--json` for machine-readable output.

## Team

Defined in `.claude/team.yaml`. Three agents with file ownership:

| Agent | Owns | Role |
|-------|------|------|
| core-dev | `src/commands/**`, `src/models/**`, `src/main.rs` | CLI commands, models, wiring |
| bdd | `tests/**` | BDD feature files, step definitions, harness |
| db | `src/db/**` | Schema, migrations, queries, data integrity |

Learnings persist in `memory/agents/<name>/learnings.md`. Team decisions in `memory/team/decisions.md`.

## Operating Mode: Orchestrator

**The primary Claude Code session operates as an orchestrator only.** Do not directly implement tasks -- instead, dispatch work to specialized subagents.

### Orchestrator Responsibilities

1. **Task Dispatch**: Delegate implementation work to appropriate subagents via the Task tool
2. **Coordination**: Manage dependencies between tasks, unblock work, review agent outputs
3. **Backlog Management**: Use `tk` commands to triage, prioritize, and track issues
4. **Session Management**: Run `tk prime` at session start, persist state before ending

### When to Invoke Each Agent

| Agent | Invoke When... |
|-------|----------------|
| core-dev | Adding/modifying CLI commands, models, or main.rs wiring |
| bdd | Writing feature files, step definitions, or test harness changes |
| db | Schema changes, migrations, query optimization, data integrity |

### Serialized Dispatching

**Dispatch tasks one at a time, not in parallel.** This approach:
- Avoids API throttling, enabling longer uninterrupted work sessions
- Allows learning from each task's output before starting the next
- Reduces context bloat from concurrent agent results

Workflow: dispatch -> wait for completion -> review -> dispatch next task

## Skill Quick Reference

| I want to... | Use |
|---|---|
| Explore something unknown | /blossom or /fractal |
| Research + prioritize | /gather -> /distill -> /rank |
| Review code | /review |
| Run a session | /status -> ... -> /retro -> /handoff |

## Do Not Modify

- `.tacks/tacks.db` -- managed by tacks, not by hand
- `Cargo.lock` -- managed by cargo
- `memory/agents/*/learnings.md` -- managed by individual agents

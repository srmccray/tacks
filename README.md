# tacks

Lightweight task manager for AI coding agents. Local-only, single-binary, SQLite-backed.

## Install

### Pre-built binaries (recommended)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/srmccray/tacks/releases/latest/download/tacks-installer.sh | sh

# Windows
powershell -ExecutionPolicy ByPass -c "irm https://github.com/srmccray/tacks/releases/latest/download/tacks-installer.ps1 | iex"
```

Or download binaries directly from [GitHub Releases](https://github.com/srmccray/tacks/releases).

### From crates.io

```bash
cargo install tacks
```

## Quick start

```bash
tk init                              # initialize in current directory
tk create "Implement auth" -p 1      # create a P1 task
tk create "Write tests" -d "Unit and integration tests for auth module"
tk list                              # show open tasks
tk ready                             # tasks with no blockers
tk update <id> --claim               # claim a task (sets in_progress + assignee)
tk close <id> -c "Done"              # close with comment
```

## Commands

| Command | Description |
|---------|-------------|
| `tk init` | Initialize a tacks database in the current directory |
| `tk create <title>` | Create a task (`-p` priority, `-d` description, `-t` tags, `--parent` subtask) |
| `tk list` | List open tasks (`-a` all, `-s` status, `-p` priority, `-t` tag, `--parent` filter) |
| `tk ready` | Show tasks with no open blockers (`--limit N`) |
| `tk show <id>` | Task details with blockers, dependents, comments, notes |
| `tk update <id>` | Update fields (`--claim`, `--notes`, `-d`, `-p`, `-t`, `-s`) |
| `tk close <id>` | Close a task (`-c` comment, `-r` reason, `--force` to bypass subtask guard) |
| `tk dep add <child> <parent>` | Add a dependency (cycle-checked) |
| `tk dep remove <child> <parent>` | Remove a dependency |
| `tk comment <id> <body>` | Add a comment |
| `tk children <id>` | List subtasks of a task |
| `tk epic` | Show epic progress (completion stats) |
| `tk blocked` | List tasks blocked by open dependencies |
| `tk stats` | Backlog overview (`--oneline` for compact output) |
| `tk prime` | AI context output: stats + in-progress + ready queue |

All commands support `--json` for machine-readable output.

## Designed for agents

Tacks is built to be consumed by AI coding agents like Claude Code:

- **`tk prime --json`** gives agents a snapshot of project state: what's in progress, what's ready, backlog stats
- **`tk ready --limit 1`** picks the next task for an agent to work on
- **`--json` on every command** means agents can parse output reliably
- **Hash-based IDs** (`tk-a1b2`) are short and unambiguous
- **Dependency tracking** with cycle detection prevents agents from picking up blocked work
- **Subtask hierarchies** with auto-tagging: creating a subtask automatically tags the parent as an epic

## Key concepts

- **Priority**: 0-4 (0 = critical, 4 = backlog)
- **Close reasons**: `done`, `duplicate`, `absorbed`, `stale`, `superseded`
- **Notes vs comments**: Notes are mutable working context (overwritten). Comments are append-only history.
- **Close guard**: Can't close a task with open subtasks unless you use `--force`
- **Tags over types**: Epic, bug, etc. are tags, not a type system. The `epic` tag is auto-added when you create a subtask.

## Storage

Tacks uses SQLite (bundled, no system dependency) stored at `.tacks/tacks.db` in your project directory. Override with `TACKS_DB` environment variable.

No sync, no git integration, no network calls. Everything stays local.

## License

MIT

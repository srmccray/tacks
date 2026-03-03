---
description: List tasks with optional filters for status, priority, tag, and parent
---

Run `tk list` to view open tasks. Apply filters to narrow by status, priority, tag, or parent task.

## Usage

```bash
tk list [-a] [-s <status>] [-p <priority>] [-t <tag>] [--parent <id>] [--json]
```

## Flags

| Flag | Description |
|------|-------------|
| `-a` | Show all tasks (including closed/done) |
| `-s <status>` | Filter by status: `open`, `in_progress`, `done`, `blocked` |
| `-p <priority>` | Filter by priority level (e.g., `-p 1` for P1 tasks) |
| `-t <tag>` | Filter by tag (e.g., `-t backend`) |
| `--parent <id>` | Show only children of the given task ID |

## Instructions

1. Determine what the user wants to see (all tasks, a filtered subset, subtasks of a specific epic).
2. Run `tk list <flags> --json` using the Bash tool to get structured output.
3. Present the task list in a readable summary.

## Examples

```bash
# List all open tasks (default)
tk list --json

# List all tasks including done
tk list -a --json

# Filter by status and tag
tk list -s in_progress -t backend --json

# Show only P1 tasks
tk list -p 1 --json

# Show subtasks of a specific epic
tk list --parent tk-a1b2 --json
```

## Notes

- Default behavior shows only `open` and `in_progress` tasks.
- Use `-a` to include `done` and `blocked` tasks.
- Filters can be combined (e.g., `-s open -t frontend -p 2`).
- For a quick AI context summary (stats + in-progress + ready queue), use `/prime` instead.

---
description: Find tasks that are ready to work on — open with no unresolved blockers
---

Run `tk ready` to list tasks that have no open dependencies blocking them. Use `--limit 1` to find the single best next task to pick up.

## Usage

```bash
tk ready [--limit <n>] [--json]
```

## Flags

| Flag | Description |
|------|-------------|
| `--limit <n>` | Return at most N tasks (e.g., `--limit 1` for the next task) |

## Instructions

1. Run `tk ready --json` using the Bash tool to get unblocked tasks.
2. For picking the next task to work on, use `tk ready --limit 1 --json`.
3. Present the ready tasks sorted by priority.
4. Optionally claim the top task with `tk update <id> --claim`.

## Examples

```bash
# List all unblocked tasks
tk ready --json

# Find the single best next task
tk ready --limit 1 --json

# Claim the next task after finding it
tk ready --limit 1 --json
tk update tk-a1b2 --claim
```

## Notes

- "Ready" means: status is `open` AND all dependency tasks are closed/done.
- Tasks are returned sorted by priority (P1 first).
- This is the recommended way for an AI agent to discover what to work on next.
- After claiming a task with `tk update --claim`, its status becomes `in_progress`.

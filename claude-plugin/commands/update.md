---
description: Update task fields, claim a task, or set working notes
---

Run `tk update <id>` to modify task properties. Use `--claim` to take ownership and move the task to in_progress.

## Usage

```bash
tk update <id> [--title <title>] [--priority <n>] [--status <status>]
               [--tags <tags>] [--assignee <name>] [--claim] [--notes <text>]
```

## Flags

| Flag | Description |
|------|-------------|
| `--title <title>` | New title for the task |
| `--priority <n>` | New priority level (1 = highest) |
| `--status <status>` | New status: `open`, `in_progress`, `done`, `blocked` |
| `--tags <tags>` | Replace tags with comma-separated list |
| `--assignee <name>` | Assign to a person or agent |
| `--claim` | Set status to `in_progress` and assignee to current user |
| `--notes <text>` | Set mutable working notes (overwrites previous notes) |

## Instructions

1. Identify the task ID and which fields need updating.
2. Run `tk update <id> <flags> --json` using the Bash tool.
3. Confirm the update to the user.

## Examples

```bash
# Claim a task before starting work
tk update tk-a1b2 --claim

# Update priority and tags
tk update tk-a1b2 --priority 1 --tags backend,urgent

# Set working notes (overwrites previous notes)
tk update tk-a1b2 --notes "Investigated root cause — issue is in auth middleware"

# Change status to blocked
tk update tk-a1b2 --status blocked
```

## Notes

- `--claim` is the standard way to start working on a task: it sets `status=in_progress` and records the assignee.
- `--notes` **overwrites** previous notes — it is mutable working context, not a log. Use `tk comment` to append to the activity log instead.
- Multiple flags can be combined in a single `tk update` call.
- Use `--json` to parse the updated task back.

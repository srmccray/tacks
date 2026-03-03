---
description: Close a task with an optional comment and structured close reason
---

Run `tk close <id>` to mark a task as done. Optionally provide a comment and a structured close reason.

## Usage

```bash
tk close <id> [-c <comment>] [-r <reason>] [--force]
```

## Flags

| Flag | Description |
|------|-------------|
| `-c <comment>` | Closing comment explaining what was done |
| `-r <reason>` | Structured close reason (see values below) |
| `--force` | Close even if the task has open subtasks |

## Close Reason Values

| Value | Meaning |
|-------|---------|
| `done` | Work completed successfully (default) |
| `duplicate` | Duplicate of another task |
| `absorbed` | Scope absorbed into another task |
| `stale` | No longer relevant |
| `superseded` | Replaced by a newer task |

## Instructions

1. Identify the task ID and reason for closing.
2. Run `tk close <id> -c "<comment>" -r <reason>` using the Bash tool.
3. If the task has open subtasks and you still want to close it, add `--force`.
4. Confirm to the user: "Closed task `<id>`."

## Examples

```bash
# Close with a comment
tk close tk-a1b2 -c "Implemented and shipped in PR #42"

# Close as duplicate
tk close tk-a1b2 -r duplicate -c "Duplicate of tk-c3d4"

# Close stale task
tk close tk-a1b2 -r stale

# Force-close an epic with open subtasks
tk close tk-a1b2 --force -c "Cancelling — requirements changed"
```

## Notes

- By default, `tk close` will refuse to close a task that has open subtasks (to protect the hierarchy). Use `--force` to override.
- The close reason is stored as a structured field, not just a comment — useful for analytics via `tk stats`.
- Closed tasks remain in the database and are visible with `tk list -s done`.

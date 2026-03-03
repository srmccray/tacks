---
description: Show full details for a task including blockers, dependents, subtasks, and comments
---

Run `tk show <id>` to display all details for a task: title, status, priority, description, tags, assignee, notes, close reason, blockers, dependents, children, and comments.

## Usage

```bash
tk show <id> [--json]
```

## Instructions

1. Obtain the task ID (from context, a previous command, or the user).
2. Run `tk show <id> --json` using the Bash tool to get full structured output.
3. Present the relevant details to the user in a readable format.

## What's Included

- **Core fields**: id, title, description, status, priority, assignee, tags, created_at, updated_at
- **Workflow fields**: close_reason (if closed), notes (mutable working context)
- **Blockers**: tasks that must be completed before this one
- **Dependents**: tasks that are waiting on this one
- **Children**: subtasks under this task
- **Comments**: append-only activity log

## Examples

```bash
# Show task details (human-readable)
tk show tk-a1b2

# Show task details (JSON for parsing)
tk show tk-a1b2 --json
```

## Notes

- Use `--json` when you need to programmatically extract fields (e.g., check if a task is blocked).
- The `notes` field is mutable working context set via `tk update --notes`; it is distinct from comments.
- Subtask IDs follow the `tk-a1b2.N` format — use the full ID with `tk show`.

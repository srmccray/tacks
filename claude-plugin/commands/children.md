---
description: List all subtasks of a given task (its direct children)
---

Run `tk children <id>` to list all subtasks that belong to a given parent task.

## Usage

```bash
tk children <id> [--json]
```

## Instructions

1. Obtain the parent task ID.
2. Run `tk children <id> --json` using the Bash tool.
3. Present the subtasks with their status and priority.

## Examples

```bash
# List subtasks of an epic
tk children tk-a1b2 --json
```

## Notes

- Subtask IDs follow the `tk-a1b2.N` pattern (e.g., `tk-a1b2.1`, `tk-a1b2.2`).
- For a combined view of an epic with progress stats, use `/epic` instead.
- You can also filter by parent using `tk list --parent <id>` for more filter options.
- A parent task is automatically tagged as `epic` when the first child is created.

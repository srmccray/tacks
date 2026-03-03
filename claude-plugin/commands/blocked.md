---
description: Show tasks that are blocked by unresolved dependencies
---

Run `tk blocked` to list all tasks that have at least one open (unresolved) dependency blocking them.

## Usage

```bash
tk blocked [--json]
```

## Instructions

1. Run `tk blocked --json` using the Bash tool.
2. Review which tasks are blocked and what is blocking them.
3. Use `tk show <id>` to inspect the specific blockers on any task.
4. To unblock a task, either close the blocking task or remove the dependency with `tk dep remove`.

## Examples

```bash
# List all blocked tasks
tk blocked --json

# Inspect what's blocking a specific task
tk show tk-a1b2 --json
```

## Notes

- A task appears here when it has a `dep add` relationship where the depended-upon task is still open.
- To see the inverse (tasks with no blockers that are ready to work), use `/ready` instead.
- To remove a dependency relationship: `tk dep remove <child> <parent>`.

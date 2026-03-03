---
description: Manage task dependencies — add or remove blocker relationships between tasks
---

Run `tk dep add <child> <parent>` to declare that `<child>` is blocked by `<parent>`. Use `tk dep remove` to undo.

## Usage

```bash
tk dep add <child-id> <parent-id>
tk dep remove <child-id> <parent-id>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a dependency: child is blocked by parent |
| `remove` | Remove a dependency relationship |

## Instructions

### Adding a dependency

1. Identify the two task IDs: the task being blocked (`child`) and the blocker (`parent`).
2. Run `tk dep add <child-id> <parent-id>` using the Bash tool.
3. Confirm: "`<child>` now depends on `<parent>`."

### Removing a dependency

1. Identify the dependency to remove.
2. Run `tk dep remove <child-id> <parent-id>` using the Bash tool.
3. Confirm: "Dependency removed — `<child>` is no longer blocked by `<parent>`."

## Examples

```bash
# Task tk-b1c2 cannot start until tk-a1b2 is done
tk dep add tk-b1c2 tk-a1b2

# Remove that dependency
tk dep remove tk-b1c2 tk-a1b2
```

## Notes

- Dependencies are cycle-checked at write time — tacks will reject a `dep add` that would create a circular dependency.
- `<child>` is the downstream task (the one being blocked). `<parent>` is the upstream blocker.
- To see which tasks are blocked, use `/blocked`. To see which tasks are ready (no blockers), use `/ready`.
- To inspect the full dependency graph for a task, use `tk show <id>` which lists both `blockers` and `dependents`.

---
description: Show epic progress — completion stats for all epics and their subtasks
---

Run `tk epic` to see all epics with their subtask completion statistics.

## Usage

```bash
tk epic [--json]
```

## Instructions

1. Run `tk epic --json` using the Bash tool.
2. Present each epic with its completion ratio (e.g., "5/8 subtasks done").
3. Use this to assess overall project progress and which epics need attention.

## Examples

```bash
# Show all epic progress (human-readable)
tk epic

# Show epic progress (JSON for parsing)
tk epic --json
```

## Notes

- An epic is any task tagged with `epic`. The tag is automatically applied when a subtask is created with `--parent <id>`.
- Each epic entry shows total subtasks, completed subtasks, and a percentage.
- To drill into a specific epic's subtasks, use `tk children <id>` or `tk list --parent <id>`.
- To see the full epic detail including the description and comments, use `tk show <id>`.

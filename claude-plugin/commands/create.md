---
description: Create a new task in the tacks backlog
---

Run `tk create <title>` to create a new task. Optionally specify priority, description, tags, and parent task.

## Usage

```bash
tk create "Title" [-p <priority>] [-d <description>] [-t <tags>] [--parent <id>]
```

## Flags

| Flag | Description |
|------|-------------|
| `-p <priority>` | Priority level (1 = highest). Default: 3 |
| `-d <description>` | Longer description for the task |
| `-t <tags>` | Comma-separated tags (e.g., `backend,api`) |
| `--parent <id>` | Parent task ID — makes this a subtask (e.g., `tk-a1b2`) |

## Instructions

1. Determine the task title, and optionally priority, description, tags, and parent.
2. Run `tk create <title> --json` using the Bash tool to get structured output.
3. Parse the JSON response to get the new task's ID.
4. Confirm to the user: "Created task `<id>`: <title>"

## Examples

```bash
# Create a simple task
tk create "Fix login bug" --json

# Create a P1 task with tags
tk create "Add OAuth support" -p 1 -t backend,auth --json

# Create a subtask under an epic
tk create "Write unit tests" --parent tk-a1b2 --json
```

## Notes

- Task IDs are hash-based: `tk-XXXX` format.
- Subtask IDs use hierarchical format: `tk-a1b2.1`, `tk-a1b2.2`, etc.
- When `--parent` is used, the parent task is automatically tagged as `epic` if it isn't already.
- Use `--json` for machine-readable output; omit for a human-friendly table row.

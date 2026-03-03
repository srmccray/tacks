---
description: Show backlog overview — counts by status, priority, and tag
---

Run `tk stats` for a full breakdown of the backlog. Use `--oneline` for a compact single-line summary.

## Usage

```bash
tk stats [--oneline] [--json]
```

## Flags

| Flag | Description |
|------|-------------|
| `--oneline` | Compact summary: e.g., "3 open, 2 in_progress, 5 done" |

## Instructions

1. Run `tk stats --json` using the Bash tool for structured output.
2. Or run `tk stats --oneline` for a quick inline summary.
3. Present the counts by status, priority, and tag to give a project overview.

## Examples

```bash
# Full stats breakdown
tk stats --json

# Quick one-liner
tk stats --oneline
```

## Notes

- Use `tk stats --oneline` when you need a compact status line (e.g., in a session summary).
- The full `tk stats` output breaks down counts by: status, priority, and top tags.
- For a combined AI-context view (stats + in-progress + ready queue), use `/prime` instead.

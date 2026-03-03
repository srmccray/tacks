---
description: Initialize tacks in the current directory
---

Run `tk init` to set up a tacks database in the current directory. This creates a `.tacks/tacks.db` SQLite file that stores all tasks locally.

## Usage

```bash
tk init
```

## Instructions

1. Run `tk init` using the Bash tool.
2. Confirm success by checking the output message.
3. The database is now ready — you can start creating tasks with `/create`.

## Notes

- Only needs to be run once per project.
- The database file lives at `.tacks/tacks.db` relative to the directory where `tk init` was run.
- To use a different database path, set the `TACKS_DB` environment variable.
- If already initialized, running `tk init` again is safe (no-op or informational message).

# /retro -- Session Retrospective

Capture what happened, what worked, and what to improve.

## Usage

```
/retro
```

## Process

1. **Gather**: What was accomplished this session? (commits, tasks closed, decisions made)
2. **Assess**: What went well? What was frustrating or slow?
3. **Learn**: Any new patterns, gotchas, or conventions discovered?
4. **Persist**: Write findings to `memory/sessions/last.md` and `memory/team/retro-history.md`
5. **Output**: Compact session summary

## Persistence Targets

- `memory/sessions/last.md`: Overwritten each session (latest state only)
- `memory/team/retro-history.md`: Appended (running history)
- Agent learnings: Update `memory/agents/<name>/learnings.md` if agent-specific discoveries were made

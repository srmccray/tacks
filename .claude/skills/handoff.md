# /handoff -- Session Handoff

Prepare context for the next session (human or AI).

## Usage

```
/handoff
```

## Process

1. **State snapshot**: Current branch, uncommitted changes, backlog state
2. **Work in progress**: What was started but not finished?
3. **Blockers**: What is blocking progress? External dependencies?
4. **Next steps**: What should the next session do first?
5. **Persist**: Write to `memory/sessions/last.md`

## Output Format

Write to `memory/sessions/last.md`:

```markdown
# Session Handoff

## Date
<date>

## Completed
- <item>

## In Progress
- <item> -- <what remains>

## Blocked
- <item> -- <blocker>

## Next Session Should
1. <first action>
2. <second action>
```

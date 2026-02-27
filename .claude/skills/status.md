# /status -- Session Orientation

Orient a new session by checking project state.

## Usage

```
/status
```

## Process

1. **Git state**: `git status`, current branch, uncommitted changes
2. **Backlog state**: `tk prime` for stats + in-progress + ready queue
3. **Recent history**: `git log --oneline -5` for recent commits
4. **Memory check**: Read `memory/sessions/last.md` for prior session context
5. **Output**: Compact orientation summary with recommended next action

## Output Format

```
Branch: <branch> | Uncommitted: <yes/no>
Backlog: <open> open, <in_progress> in progress, <done> done
Last session: <summary from memory/sessions/last.md>
Recommended: <next action>
```

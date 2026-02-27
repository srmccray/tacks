# /review -- Structured Code Review

Review code changes with a systematic checklist.

## Usage

```
/review [file or diff range]
```

## Process

1. **Scope**: Identify what changed (git diff, specific files, or PR)
2. **Correctness**: Does the code do what it claims? Edge cases handled?
3. **Style**: Follows project conventions (see rules/rust.md, rules/commits.md)?
4. **Tests**: Are changes covered by BDD scenarios? New edge cases need new tests?
5. **Integration**: Does it break existing callers? Check `row_to_task`, `list_tasks`, `update_task` signatures.
6. **Output**: List of findings categorized as: blocking, suggestion, nit

## Tacks-Specific Checks

- New commands support `--json` output
- DB function signature changes propagate to all callers
- Feature files written BEFORE implementation for new features
- `cargo clippy -- -D warnings` clean
- `cargo fmt --check` clean
- No `unwrap()` outside tests

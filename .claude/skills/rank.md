# /rank -- Prioritize Items

Order a collection of items by impact, urgency, or other criteria.

## Usage

```
/rank <items or source> [--by <criteria>]
```

## Process

1. **Input**: Read the items to rank (from /distill output, task list, or specified source)
2. **Criteria**: Apply ranking criteria (default: impact vs effort)
3. **Score**: Assign relative priority to each item
4. **Output**: Ordered list with brief justification for ranking

## Default Criteria

- **Impact**: How much does this matter for project goals?
- **Effort**: How much work is required?
- **Risk**: What happens if we skip this?
- **Dependencies**: Does this unblock other work?

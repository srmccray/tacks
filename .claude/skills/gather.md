# /gather -- Structured Information Collection

Systematically collect information from the codebase or external sources.

## Usage

```
/gather <topic>
```

## Process

1. **Scope**: Define what information is needed and where to look
2. **Collect**: Read files, grep patterns, check configs -- one source at a time
3. **Organize**: Group findings by category or relevance
4. **Output**: Structured list of findings with source references

## Notes

- Respects batch-safety: if collecting more than 12 items, checkpoint after each chunk
- Pairs with /distill (summarize findings) and /rank (prioritize findings)

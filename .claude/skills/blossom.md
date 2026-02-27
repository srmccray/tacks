# /blossom -- Spike-Driven Exploration

Explore unfamiliar territory through time-boxed spikes that branch and converge.

## Usage

```
/blossom <goal>
```

## Process

1. **Frame**: State the exploration goal and what "enough understanding" looks like
2. **Branch**: Identify 2-4 spike areas to investigate (15 min each max)
3. **Spike**: For each area, gather facts, note surprises, identify connections
4. **Converge**: Synthesize findings into actionable next steps
5. **Output**: Structured summary with findings, decisions, and tasks to create

## Spike Areas for Tacks

Common spike targets in this project:
- **Commands**: How a specific `src/commands/*.rs` file implements its subcommand
- **Database**: Schema structure, migration patterns, query patterns in `src/db/mod.rs`
- **Models**: Data types and their relationships in `src/models/mod.rs`
- **Tests**: BDD patterns in `tests/features/` and `tests/bdd/`
- **CLI wiring**: How `src/main.rs` dispatches to command handlers

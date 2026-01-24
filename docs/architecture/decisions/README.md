# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) documenting significant technical decisions made during development.

## What is an ADR?

An ADR captures a decision along with its context and consequences. They serve as a historical record for future developers (and AI assistants) to understand *why* things are the way they are.

## Format

Each ADR follows this structure:

```markdown
# ADR-NNN: Title

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-XXX]

## Context
What is the issue or situation that motivates this decision?

## Decision
What is the change that we're proposing and/or doing?

## Consequences
What becomes easier or harder because of this change?
```

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [001](./ADR-001-ecs-architecture.md) | ECS Architecture | Accepted |
| [002](./ADR-002-rust-bevy.md) | Rust + Bevy | Accepted |
| [003](./ADR-003-fixed-point-math.md) | Fixed-Point Math | Accepted |
| [004](./ADR-004-workspace-structure.md) | Workspace Structure | Accepted |
| [005](./ADR-005-error-handling.md) | Error Handling | Accepted |
| [006](./ADR-006-tracing.md) | Tracing and Logging | Accepted |

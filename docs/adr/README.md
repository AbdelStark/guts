# Architecture Decision Records (ADR)

This directory contains Architecture Decision Records for the Guts project.

## What is an ADR?

An Architecture Decision Record captures an important architectural decision made along with its context and consequences.

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](001-commonware-primitives.md) | Use Commonware for P2P and Consensus | Accepted |
| [ADR-002](002-content-addressed-storage.md) | Content-Addressed Storage for Git Objects | Accepted |
| [ADR-003](003-git-protocol-implementation.md) | Custom Git Smart HTTP Protocol | Accepted |
| [ADR-004](004-collaboration-data-model.md) | Collaboration Data Model | Accepted |
| [ADR-005](005-permission-hierarchy.md) | Permission and Access Control Hierarchy | Accepted |
| [ADR-006](006-api-design.md) | REST API Design Principles | Accepted |
| [ADR-007](007-crate-architecture.md) | Modular Crate Architecture | Accepted |

## ADR Template

```markdown
# ADR-XXX: Title

## Status
Proposed | Accepted | Deprecated | Superseded

## Context
What is the issue we're seeing that motivates this decision?

## Decision
What is the change we're proposing/have decided?

## Consequences
What becomes easier or harder because of this change?

## Alternatives Considered
What other options were evaluated?
```

## References

- [ADR GitHub Organization](https://adr.github.io/)
- [Michael Nygard's Article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)

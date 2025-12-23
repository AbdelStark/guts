# Architecture Decision Records

This section contains Architecture Decision Records (ADRs) for the Guts project.

## What is an ADR?

An Architecture Decision Record captures an important architectural decision made along with its context and consequences. ADRs help future contributors understand why certain decisions were made.

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](/architecture/adr/001-commonware-primitives) | Use Commonware for P2P and Consensus | Accepted |
| [ADR-002](/architecture/adr/002-content-addressed-storage) | Content-Addressed Storage for Git Objects | Accepted |
| [ADR-003](/architecture/adr/003-git-protocol-implementation) | Custom Git Smart HTTP Protocol | Accepted |
| [ADR-004](/architecture/adr/004-collaboration-data-model) | Collaboration Data Model | Accepted |
| [ADR-005](/architecture/adr/005-permission-hierarchy) | Permission and Access Control Hierarchy | Accepted |
| [ADR-006](/architecture/adr/006-api-design) | REST API Design Principles | Accepted |
| [ADR-007](/architecture/adr/007-crate-architecture) | Modular Crate Architecture | Accepted |

## ADR Template

When creating a new ADR, use this template:

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

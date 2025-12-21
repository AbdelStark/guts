# ADR-007: Modular Crate Architecture

## Status

Accepted

## Date

2025-12-20

## Context

Guts is a complex system with multiple concerns:

- Core data types and traits
- Git storage and protocol
- P2P networking
- Collaboration features
- Access control and governance
- HTTP API serving
- CLI tooling
- Web interface

These concerns need clear boundaries for:
- Independent testing and development
- Clear dependency direction
- Reusability of components
- Separation of library and binary code

## Decision

We organize Guts as a Cargo workspace with 9 crates:

### Crate Hierarchy

```
                    ┌─────────────┐
                    │ guts-types  │  Foundation layer
                    └──────┬──────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼─────┐  ┌──────▼──────┐       │
    │guts-storage│  │  guts-git   │       │
    └──────┬─────┘  └──────┬──────┘       │
           │               │               │
           └───────┬───────┘               │
                   │                       │
           ┌───────▼───────┐               │
           │   guts-p2p    │◄──────────────┘
           └───────┬───────┘
                   │
     ┌─────────────┼─────────────┐
     │             │             │
┌────▼─────┐ ┌─────▼────┐ ┌──────▼─────┐
│guts-collab│ │guts-auth │ │  guts-web  │  Feature layer
└────┬─────┘ └─────┬────┘ └──────┬─────┘
     │             │             │
     └─────────────┼─────────────┘
                   │
           ┌───────▼───────┐
           │  guts-node    │  Application layer
           └───────┬───────┘
                   │
           ┌───────▼───────┐
           │   guts-cli    │  Binary layer
           └───────────────┘
```

### Crate Descriptions

| Crate | Type | Purpose |
|-------|------|---------|
| `guts-types` | lib | Core types, traits, error types, constants |
| `guts-storage` | lib | Content-addressed object store, reference management |
| `guts-git` | lib | Git protocol (pktline, pack files, smart HTTP) |
| `guts-p2p` | lib | P2P messaging, replication, node discovery |
| `guts-collaboration` | lib | PRs, Issues, Comments, Reviews, Labels |
| `guts-auth` | lib | Orgs, Teams, Permissions, Webhooks, Branch Protection |
| `guts-web` | lib | Web gateway, HTML templates, Markdown rendering |
| `guts-node` | bin | Full node binary, HTTP API, P2P integration |
| `guts-cli` | bin | CLI client for all operations |

### Dependency Rules

1. **Acyclic**: No circular dependencies between crates
2. **Downward only**: Higher layers depend on lower layers
3. **Types shared**: `guts-types` is the common foundation
4. **Binaries thin**: `guts-node` and `guts-cli` are thin wrappers

### File Layout

```
crates/
├── guts-types/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public exports
│       ├── identity.rs     # Identity types
│       ├── repository.rs   # Repository types
│       └── error.rs        # Error types
│
├── guts-storage/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Storage traits and impls
│       ├── object.rs       # Object store
│       └── refs.rs         # Reference store
│
├── guts-git/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── pktline.rs      # Packet-line protocol
│       ├── pack.rs         # Pack file handling
│       ├── protocol.rs     # Smart HTTP protocol
│       └── objects.rs      # Git object types
│
├── guts-p2p/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── message.rs      # P2P message types
│       ├── protocol.rs     # Replication protocol
│       └── collaboration_message.rs
│
├── guts-collaboration/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── pull_request.rs
│       ├── issue.rs
│       ├── comment.rs
│       ├── review.rs
│       ├── label.rs
│       └── store.rs        # Collaboration store
│
├── guts-auth/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── permission.rs
│       ├── organization.rs
│       ├── team.rs
│       ├── collaborator.rs
│       ├── branch_protection.rs
│       ├── webhook.rs
│       └── store.rs
│
├── guts-web/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── routes.rs
│   │   └── templates.rs
│   └── templates/          # Askama HTML templates
│
├── guts-node/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs         # Node binary entry
│   │   ├── lib.rs
│   │   ├── api.rs          # Core API routes
│   │   ├── collaboration_api.rs
│   │   ├── auth_api.rs
│   │   └── state.rs        # Application state
│   └── tests/              # E2E tests
│
└── guts-cli/
    ├── Cargo.toml
    └── src/
        ├── main.rs         # CLI entry
        └── commands.rs     # Subcommands
```

## Consequences

### Positive

- **Clear boundaries**: Each crate has focused responsibility
- **Parallel development**: Teams can work independently
- **Faster builds**: Incremental compilation per crate
- **Testability**: Each crate tested in isolation
- **Reusability**: Library crates usable by other projects

### Negative

- **Coordination**: Changes spanning crates need care
- **Version sync**: All crates share version (0.1.0)
- **More files**: Additional Cargo.toml per crate

### Neutral

- Workspace dependencies centralized in root Cargo.toml
- All crates share edition and profile settings

## Workspace Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/guts-types",
    "crates/guts-storage",
    "crates/guts-git",
    "crates/guts-p2p",
    "crates/guts-collaboration",
    "crates/guts-auth",
    "crates/guts-web",
    "crates/guts-node",
    "crates/guts-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
# Internal crates
guts-types = { version = "0.1.0", path = "crates/guts-types" }
# ... other internal crates

# External dependencies shared across crates
tokio = { version = "1.43", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
# ...
```

## Alternatives Considered

### Monolithic Crate

Single crate with modules.

**Rejected because:**
- Slower builds
- All-or-nothing dependency
- Harder to maintain boundaries

### More Granular Crates

Separate crate for each type (e.g., guts-pr, guts-issue).

**Rejected because:**
- Excessive fragmentation
- Too many Cargo.toml files
- Diminishing returns on separation

### Feature Flags Instead

Use features to enable/disable functionality.

**Partially adopted:**
- Feature flags within crates for optional features
- Crate boundaries for primary separation

## References

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Package Layout Convention](https://doc.rust-lang.org/cargo/guide/project-layout.html)

//! # Guts Node
//!
//! Decentralized code collaboration node library.
//!
//! This crate provides the core functionality for running a Guts node,
//! including HTTP API endpoints, P2P networking, and integration with
//! storage, collaboration, and authentication subsystems.
//!
//! ## Architecture
//!
//! A Guts node consists of several integrated components:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Guts Node                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                    HTTP API Layer                     │  │
//! │  │  • Git Smart HTTP (clone, push, pull)                │  │
//! │  │  • Repository Management (CRUD)                       │  │
//! │  │  • Collaboration API (PRs, Issues, Reviews)           │  │
//! │  │  • Auth API (Orgs, Teams, Permissions, Webhooks)      │  │
//! │  │  • Web Gateway (HTML views)                           │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! │                              │                              │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                    P2P Network Layer                  │  │
//! │  │  • Node Discovery and Connection                      │  │
//! │  │  • Repository Replication                             │  │
//! │  │  • Collaboration Data Sync                            │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! │                              │                              │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                    Storage Layer                      │  │
//! │  │  • Git Object Store (blobs, trees, commits)           │  │
//! │  │  • Reference Store (branches, tags)                   │  │
//! │  │  • Collaboration Store (PRs, Issues)                  │  │
//! │  │  • Auth Store (Orgs, Teams, Permissions)              │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! To run a node:
//!
//! ```bash
//! cargo run --bin guts-node -- --api-addr 127.0.0.1:8080
//! ```
//!
//! ## Modules
//!
//! - [`api`] - Core HTTP API and Git Smart HTTP protocol
//! - [`auth_api`] - Authorization endpoints (Organizations, Teams, Permissions)
//! - [`collaboration_api`] - Collaboration endpoints (PRs, Issues, Comments)
//! - [`realtime_api`] - Real-time WebSocket API for live updates
//! - [`config`] - Node configuration management
//! - [`p2p`] - Peer-to-peer networking and replication
//! - [`observability`] - Structured logging, metrics, and request tracing
//! - [`validation`] - Input validation middleware
//! - [`health`] - Health check endpoints (liveness, readiness, startup)
//! - [`resilience`] - Retry policies, circuit breakers, and timeouts
//! - [`performance`] - Connection pooling, request coalescing, CDN cache headers
//!
//! ## Example: Creating an AppState
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use guts_storage::RepoStore;
//! use guts_collaboration::CollaborationStore;
//! use guts_auth::AuthStore;
//! use guts_realtime::EventHub;
//! use guts_ci::CiStore;
//! use guts_compat::CompatStore;
//! use guts_node::api::AppState;
//!
//! // Create stores
//! let repos = Arc::new(RepoStore::new());
//! let collaboration = Arc::new(CollaborationStore::new());
//! let auth = Arc::new(AuthStore::new());
//! let realtime = Arc::new(EventHub::new());
//! let ci = Arc::new(CiStore::new());
//! let compat = Arc::new(CompatStore::new());
//!
//! // Create application state
//! let state = AppState {
//!     repos,
//!     p2p: None, // Optional P2P manager
//!     consensus: None, // Optional consensus engine
//!     mempool: None, // Optional transaction mempool
//!     collaboration,
//!     auth,
//!     realtime,
//!     ci,
//!     compat,
//! };
//! ```

pub mod api;
pub mod auth_api;
pub mod ci_api;
pub mod collaboration_api;
pub mod compat_api;
pub mod config;
pub mod health;
pub mod observability;
pub mod p2p;
pub mod performance;
pub mod realtime_api;
pub mod resilience;
pub mod validation;

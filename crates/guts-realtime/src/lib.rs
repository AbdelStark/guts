//! # Guts Real-time
//!
//! Real-time WebSocket support for the Guts code collaboration platform.
//!
//! This crate provides WebSocket-based real-time updates for repository events,
//! enabling live notifications and instant UI updates without page refresh.
//!
//! ## Features
//!
//! - **Event Hub**: Central management of WebSocket connections
//! - **Subscriptions**: Channel-based subscription model
//! - **Event Broadcasting**: Efficient event distribution to subscribed clients
//! - **Notifications**: User notification system
//!
//! ## Channel Types
//!
//! - `repo:owner/name` - All events for a repository
//! - `repo:owner/name/prs` - Pull request events only
//! - `repo:owner/name/issues` - Issue events only
//! - `user:username` - User notifications
//! - `org:orgname` - Organization events
//!
//! ## Example
//!
//! ```rust
//! use guts_realtime::{EventHub, EventKind};
//! use std::sync::Arc;
//!
//! // Create the event hub
//! let hub = Arc::new(EventHub::new());
//!
//! // Connect a client
//! let (client, receiver) = hub.connect().unwrap();
//!
//! // Subscribe to a repository
//! hub.handle_command(
//!     &client,
//!     guts_realtime::ClientCommand::Subscribe {
//!         channel: "repo:alice/myrepo".to_string(),
//!     },
//! ).unwrap();
//!
//! // Emit an event
//! hub.emit_event(
//!     "repo:alice/myrepo".to_string(),
//!     EventKind::Push,
//!     serde_json::json!({
//!         "ref": "refs/heads/main",
//!         "before": "abc123",
//!         "after": "def456"
//!     }),
//! );
//! ```
//!
//! ## WebSocket Protocol
//!
//! ### Client -> Server Messages
//!
//! ```json
//! // Subscribe to a channel
//! {"type": "subscribe", "channel": "repo:owner/name"}
//!
//! // Unsubscribe from a channel
//! {"type": "unsubscribe", "channel": "repo:owner/name"}
//!
//! // Ping for keepalive
//! {"type": "ping"}
//! ```
//!
//! ### Server -> Client Messages
//!
//! ```json
//! // Subscription confirmed
//! {"type": "subscribed", "channel": "repo:owner/name"}
//!
//! // Event notification
//! {"type": "event", "channel": "repo:owner/name", "event": "push", ...}
//!
//! // Pong response
//! {"type": "pong"}
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              EventHub                    │
//! │  ┌─────────────────────────────────┐    │
//! │  │         Clients Map              │    │
//! │  │  client_id -> Client            │    │
//! │  │    └─> subscriptions            │    │
//! │  │    └─> message sender           │    │
//! │  └─────────────────────────────────┘    │
//! │                  │                       │
//! │  ┌───────────────▼───────────────────┐  │
//! │  │     Broadcast Channel             │  │
//! │  │  (for external subscribers)       │  │
//! │  └───────────────────────────────────┘  │
//! └─────────────────────────────────────────┘
//! ```

pub mod client;
pub mod error;
pub mod event;
pub mod hub;
pub mod notification;
pub mod subscription;

// Re-export main types
pub use client::{Client, ClientId, ClientReceiver};
pub use error::RealtimeError;
pub use event::{
    CommentEventData, EventKind, IssueEventData, PullRequestEventData, PushEventData,
    RealtimeEvent, ReviewEventData,
};
pub use hub::{ClientCommand, EventHub, HubStats, ServerMessage};
pub use notification::{Notification, NotificationMetadata, NotificationPreferences, NotificationType};
pub use subscription::{Channel, ChannelType, ClientSubscriptions, MAX_SUBSCRIPTIONS_PER_CLIENT};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        // Test that main types are accessible
        let hub = EventHub::new();
        assert_eq!(hub.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_full_flow() {
        let hub = EventHub::new();

        // Connect
        let (client, mut rx) = hub.connect().unwrap();
        assert_eq!(hub.connection_count(), 1);

        // Subscribe
        let result = hub.handle_command(
            &client,
            ClientCommand::Subscribe {
                channel: "repo:alice/myrepo".to_string(),
            },
        );
        assert!(result.is_ok());

        // Emit event
        hub.emit_event(
            "repo:alice/myrepo".to_string(),
            EventKind::Push,
            serde_json::json!({"ref": "refs/heads/main"}),
        );

        // Verify received
        let msg = rx.try_recv();
        assert!(msg.is_ok());

        // Disconnect
        hub.disconnect(&client.id);
        assert_eq!(hub.connection_count(), 0);
    }
}

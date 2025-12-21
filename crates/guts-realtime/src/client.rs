//! Client connection management.

use crate::error::RealtimeError;
use crate::subscription::{Channel, ClientSubscriptions};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Unique identifier for a connected client.
pub type ClientId = String;

/// A connected WebSocket client.
#[derive(Debug)]
pub struct Client {
    /// Unique client identifier.
    pub id: ClientId,
    /// Channel for sending messages to this client.
    sender: mpsc::UnboundedSender<String>,
    /// Client's subscriptions.
    subscriptions: RwLock<ClientSubscriptions>,
    /// Connection metadata.
    pub metadata: ClientMetadata,
}

impl Client {
    /// Create a new client with a message sender.
    pub fn new(id: ClientId, sender: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id,
            sender,
            subscriptions: RwLock::new(ClientSubscriptions::new()),
            metadata: ClientMetadata::default(),
        }
    }

    /// Send a message to this client.
    pub fn send(&self, message: String) -> Result<(), RealtimeError> {
        self.sender
            .send(message)
            .map_err(|_| RealtimeError::ChannelClosed)
    }

    /// Subscribe to a channel.
    pub fn subscribe(&self, channel: Channel) -> Result<bool, RealtimeError> {
        self.subscriptions.write().subscribe(channel)
    }

    /// Unsubscribe from a channel.
    pub fn unsubscribe(&self, channel: &Channel) -> bool {
        self.subscriptions.write().unsubscribe(channel)
    }

    /// Check if subscribed to a channel.
    pub fn is_subscribed(&self, channel: &Channel) -> bool {
        self.subscriptions.read().is_subscribed(channel)
    }

    /// Check if any subscription matches an event channel.
    pub fn matches_event(&self, event_channel: &str) -> bool {
        self.subscriptions.read().matches_event(event_channel)
    }

    /// Get subscription count.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().count()
    }

    /// Clear all subscriptions.
    pub fn clear_subscriptions(&self) {
        self.subscriptions.write().clear();
    }
}

/// Metadata about a client connection.
#[derive(Debug, Default)]
pub struct ClientMetadata {
    /// When the client connected (Unix timestamp).
    pub connected_at: u64,
    /// Optional user identifier if authenticated.
    pub user_id: Option<String>,
    /// Client IP address.
    pub ip_address: Option<String>,
    /// User agent string.
    pub user_agent: Option<String>,
}

impl ClientMetadata {
    /// Create metadata with current timestamp.
    pub fn now() -> Self {
        Self {
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            user_id: None,
            ip_address: None,
            user_agent: None,
        }
    }
}

/// Handle for receiving messages from the hub to send to WebSocket.
pub type ClientReceiver = mpsc::UnboundedReceiver<String>;

/// Create a new client with its message receiver.
pub fn create_client(id: ClientId) -> (Arc<Client>, ClientReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let client = Arc::new(Client::new(id, sender));
    (client, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let (client, _rx) = create_client("test-client".to_string());
        assert_eq!(client.id, "test-client");
        assert_eq!(client.subscription_count(), 0);
    }

    #[test]
    fn test_client_subscribe() {
        let (client, _rx) = create_client("test-client".to_string());

        let channel = Channel::parse("repo:alice/myrepo").unwrap();
        assert!(client.subscribe(channel.clone()).unwrap());
        assert!(client.is_subscribed(&channel));
        assert_eq!(client.subscription_count(), 1);
    }

    #[test]
    fn test_client_unsubscribe() {
        let (client, _rx) = create_client("test-client".to_string());

        let channel = Channel::parse("repo:alice/myrepo").unwrap();
        client.subscribe(channel.clone()).unwrap();
        assert!(client.unsubscribe(&channel));
        assert!(!client.is_subscribed(&channel));
        assert_eq!(client.subscription_count(), 0);
    }

    #[test]
    fn test_client_matches_event() {
        let (client, _rx) = create_client("test-client".to_string());

        client
            .subscribe(Channel::parse("repo:alice/myrepo").unwrap())
            .unwrap();

        assert!(client.matches_event("repo:alice/myrepo"));
        assert!(client.matches_event("repo:alice/myrepo/prs"));
        assert!(!client.matches_event("repo:bob/otherrepo"));
    }

    #[test]
    fn test_client_send() {
        let (client, mut rx) = create_client("test-client".to_string());

        client.send("test message".to_string()).unwrap();

        // Message should be in the receiver
        let msg = rx.try_recv().unwrap();
        assert_eq!(msg, "test message");
    }

    #[test]
    fn test_client_clear_subscriptions() {
        let (client, _rx) = create_client("test-client".to_string());

        client
            .subscribe(Channel::parse("repo:alice/myrepo").unwrap())
            .unwrap();
        client
            .subscribe(Channel::parse("user:alice").unwrap())
            .unwrap();

        assert_eq!(client.subscription_count(), 2);

        client.clear_subscriptions();
        assert_eq!(client.subscription_count(), 0);
    }
}

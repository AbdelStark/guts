//! Event hub for managing WebSocket connections and broadcasting.

use crate::client::{create_client, Client, ClientId, ClientReceiver};
use crate::error::RealtimeError;
use crate::event::{EventKind, RealtimeEvent};
use crate::subscription::Channel;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info};

/// Capacity of the broadcast channel.
const BROADCAST_CAPACITY: usize = 1024;

/// Maximum number of concurrent connections.
const MAX_CONNECTIONS: usize = 10000;

/// Event hub manages all WebSocket connections and event broadcasting.
#[derive(Debug)]
pub struct EventHub {
    /// Connected clients indexed by ID.
    clients: RwLock<HashMap<ClientId, Arc<Client>>>,
    /// Broadcast channel for events.
    event_tx: broadcast::Sender<RealtimeEvent>,
    /// Statistics.
    stats: RwLock<HubStats>,
}

impl EventHub {
    /// Create a new event hub.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            clients: RwLock::new(HashMap::new()),
            event_tx,
            stats: RwLock::new(HubStats::default()),
        }
    }

    /// Connect a new client and return its message receiver.
    pub fn connect(&self) -> Result<(Arc<Client>, ClientReceiver), RealtimeError> {
        let clients = self.clients.read();
        if clients.len() >= MAX_CONNECTIONS {
            return Err(RealtimeError::SendFailed(
                "maximum connections reached".to_string(),
            ));
        }
        drop(clients);

        let client_id = uuid::Uuid::new_v4().to_string();
        let (client, receiver) = create_client(client_id.clone());

        self.clients
            .write()
            .insert(client_id.clone(), client.clone());
        self.stats.write().total_connections += 1;

        info!(client_id = %client_id, "Client connected");

        Ok((client, receiver))
    }

    /// Disconnect a client.
    pub fn disconnect(&self, client_id: &str) {
        if let Some(client) = self.clients.write().remove(client_id) {
            client.clear_subscriptions();
            info!(client_id = %client_id, "Client disconnected");
        }
    }

    /// Get a client by ID.
    pub fn get_client(&self, client_id: &str) -> Option<Arc<Client>> {
        self.clients.read().get(client_id).cloned()
    }

    /// Handle a client command.
    pub fn handle_command(
        &self,
        client: &Arc<Client>,
        command: ClientCommand,
    ) -> Result<ServerMessage, RealtimeError> {
        match command {
            ClientCommand::Subscribe { channel } => {
                let parsed = Channel::parse(&channel)?;
                let is_new = client.subscribe(parsed)?;

                if is_new {
                    debug!(client_id = %client.id, channel = %channel, "Client subscribed");
                    self.stats.write().total_subscriptions += 1;
                }

                Ok(ServerMessage::Subscribed { channel })
            }
            ClientCommand::Unsubscribe { channel } => {
                let parsed = Channel::parse(&channel)?;
                let was_subscribed = client.unsubscribe(&parsed);

                if was_subscribed {
                    debug!(client_id = %client.id, channel = %channel, "Client unsubscribed");
                }

                Ok(ServerMessage::Unsubscribed { channel })
            }
            ClientCommand::Ping => Ok(ServerMessage::Pong),
        }
    }

    /// Emit an event to all subscribed clients.
    pub fn emit(&self, event: RealtimeEvent) {
        let channel = event.channel.clone();
        let event_kind = event.event;

        // Count how many clients will receive this
        let mut recipient_count = 0;
        let clients = self.clients.read();

        for client in clients.values() {
            if client.matches_event(&channel) {
                if let Ok(json) = serde_json::to_string(&event) {
                    if client.send(json).is_ok() {
                        recipient_count += 1;
                    }
                }
            }
        }

        drop(clients);

        // Also send to broadcast channel for any listeners
        let _ = self.event_tx.send(event);

        self.stats.write().total_events += 1;

        debug!(
            channel = %channel,
            event = %event_kind,
            recipients = recipient_count,
            "Event broadcast"
        );
    }

    /// Emit an event with the given parameters.
    pub fn emit_event(&self, channel: String, event: EventKind, data: serde_json::Value) {
        self.emit(RealtimeEvent::new(channel, event, data));
    }

    /// Subscribe to the broadcast channel for events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<RealtimeEvent> {
        self.event_tx.subscribe()
    }

    /// Get current connection count.
    pub fn connection_count(&self) -> usize {
        self.clients.read().len()
    }

    /// Get hub statistics.
    pub fn stats(&self) -> HubStats {
        let mut stats = self.stats.read().clone();
        stats.current_connections = self.connection_count();
        stats
    }

    /// Broadcast a message to all clients (for system announcements).
    pub fn broadcast_all(&self, message: &str) {
        let clients = self.clients.read();
        for client in clients.values() {
            let _ = client.send(message.to_string());
        }
    }
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Commands that clients can send.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientCommand {
    /// Subscribe to a channel.
    Subscribe { channel: String },
    /// Unsubscribe from a channel.
    Unsubscribe { channel: String },
    /// Ping for keepalive.
    Ping,
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Subscription confirmed.
    Subscribed { channel: String },
    /// Unsubscription confirmed.
    Unsubscribed { channel: String },
    /// Pong response to ping.
    Pong,
    /// Error message.
    Error { message: String },
}

/// Hub statistics.
#[derive(Debug, Clone, Default)]
pub struct HubStats {
    /// Current number of connections.
    pub current_connections: usize,
    /// Total connections since start.
    pub total_connections: u64,
    /// Total subscriptions since start.
    pub total_subscriptions: u64,
    /// Total events broadcast since start.
    pub total_events: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hub_connect() {
        let hub = EventHub::new();
        let (client, _rx) = hub.connect().unwrap();

        assert!(!client.id.is_empty());
        assert_eq!(hub.connection_count(), 1);
    }

    #[tokio::test]
    async fn test_hub_disconnect() {
        let hub = EventHub::new();
        let (client, _rx) = hub.connect().unwrap();
        let client_id = client.id.clone();

        hub.disconnect(&client_id);
        assert_eq!(hub.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_hub_subscribe_command() {
        let hub = EventHub::new();
        let (client, _rx) = hub.connect().unwrap();

        let cmd = ClientCommand::Subscribe {
            channel: "repo:alice/myrepo".to_string(),
        };

        let response = hub.handle_command(&client, cmd).unwrap();
        assert!(matches!(response, ServerMessage::Subscribed { .. }));
        assert_eq!(client.subscription_count(), 1);
    }

    #[tokio::test]
    async fn test_hub_unsubscribe_command() {
        let hub = EventHub::new();
        let (client, _rx) = hub.connect().unwrap();

        // Subscribe first
        hub.handle_command(
            &client,
            ClientCommand::Subscribe {
                channel: "repo:alice/myrepo".to_string(),
            },
        )
        .unwrap();

        // Then unsubscribe
        let response = hub
            .handle_command(
                &client,
                ClientCommand::Unsubscribe {
                    channel: "repo:alice/myrepo".to_string(),
                },
            )
            .unwrap();

        assert!(matches!(response, ServerMessage::Unsubscribed { .. }));
        assert_eq!(client.subscription_count(), 0);
    }

    #[tokio::test]
    async fn test_hub_ping_pong() {
        let hub = EventHub::new();
        let (client, _rx) = hub.connect().unwrap();

        let response = hub.handle_command(&client, ClientCommand::Ping).unwrap();
        assert!(matches!(response, ServerMessage::Pong));
    }

    #[tokio::test]
    async fn test_hub_emit_event() {
        let hub = EventHub::new();
        let (client, mut rx) = hub.connect().unwrap();

        // Subscribe to repository
        hub.handle_command(
            &client,
            ClientCommand::Subscribe {
                channel: "repo:alice/myrepo".to_string(),
            },
        )
        .unwrap();

        // Emit an event
        hub.emit_event(
            "repo:alice/myrepo".to_string(),
            EventKind::Push,
            serde_json::json!({"ref": "refs/heads/main"}),
        );

        // Client should receive the event
        let msg = rx.try_recv().unwrap();
        assert!(msg.contains("push"));
        assert!(msg.contains("repo:alice/myrepo"));
    }

    #[tokio::test]
    async fn test_hub_emit_filtered() {
        let hub = EventHub::new();
        let (client1, mut rx1) = hub.connect().unwrap();
        let (client2, mut rx2) = hub.connect().unwrap();

        // Client 1 subscribes to alice/myrepo
        hub.handle_command(
            &client1,
            ClientCommand::Subscribe {
                channel: "repo:alice/myrepo".to_string(),
            },
        )
        .unwrap();

        // Client 2 subscribes to bob/otherrepo
        hub.handle_command(
            &client2,
            ClientCommand::Subscribe {
                channel: "repo:bob/otherrepo".to_string(),
            },
        )
        .unwrap();

        // Emit an event for alice/myrepo
        hub.emit_event(
            "repo:alice/myrepo".to_string(),
            EventKind::Push,
            serde_json::json!({}),
        );

        // Client 1 should receive it
        assert!(rx1.try_recv().is_ok());

        // Client 2 should not receive it
        assert!(rx2.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_hub_stats() {
        let hub = EventHub::new();

        let (client, _rx) = hub.connect().unwrap();
        hub.handle_command(
            &client,
            ClientCommand::Subscribe {
                channel: "repo:alice/myrepo".to_string(),
            },
        )
        .unwrap();
        hub.emit_event(
            "repo:alice/myrepo".to_string(),
            EventKind::Push,
            serde_json::json!({}),
        );

        let stats = hub.stats();
        assert_eq!(stats.current_connections, 1);
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.total_subscriptions, 1);
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_client_command_serialization() {
        let cmd = ClientCommand::Subscribe {
            channel: "repo:alice/myrepo".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("repo:alice/myrepo"));

        let parsed: ClientCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClientCommand::Subscribe { .. }));
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::Subscribed {
            channel: "repo:test/repo".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribed"));

        let pong = ServerMessage::Pong;
        let json = serde_json::to_string(&pong).unwrap();
        assert!(json.contains("pong"));
    }
}

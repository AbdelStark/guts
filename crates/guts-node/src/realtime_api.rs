//! Real-time WebSocket API for live updates.
//!
//! This module provides WebSocket endpoints for real-time communication:
//!
//! - `/ws` - Main WebSocket endpoint for real-time updates
//! - `/api/realtime/stats` - Statistics about real-time connections
//!
//! ## WebSocket Protocol
//!
//! Clients can subscribe to channels to receive real-time events:
//!
//! ```json
//! // Subscribe to a repository
//! {"type": "subscribe", "channel": "repo:owner/name"}
//!
//! // Unsubscribe from a channel
//! {"type": "unsubscribe", "channel": "repo:owner/name"}
//!
//! // Ping for keepalive
//! {"type": "ping"}
//! ```

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use guts_realtime::{ClientCommand, EventHub, ServerMessage};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::api::AppState;

/// Create the real-time API routes.
pub fn realtime_routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/realtime/stats", get(get_stats))
}

/// WebSocket upgrade handler.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.realtime.clone()))
}

/// Handle a WebSocket connection.
async fn handle_socket(socket: WebSocket, hub: Arc<EventHub>) {
    // Connect the client to the hub
    let (client, mut receiver) = match hub.connect() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect client: {}", e);
            return;
        }
    };

    let client_id = client.id.clone();
    info!(client_id = %client_id, "WebSocket client connected");

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Spawn a task to forward messages from the hub to the WebSocket
    let client_id_clone = client_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
        debug!(client_id = %client_id_clone, "Send task ended");
    });

    // Handle incoming messages from the WebSocket
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text_str: &str = &text;
                match serde_json::from_str::<ClientCommand>(text_str) {
                    Ok(cmd) => {
                        let response = hub.handle_command(&client, cmd);
                        match response {
                            Ok(msg) => {
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = client.send(json);
                                }
                            }
                            Err(e) => {
                                let error_msg = ServerMessage::Error {
                                    message: e.to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = client.send(json);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!(client_id = %client_id, error = %e, "Invalid message format");
                        let error_msg = ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = client.send(json);
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                debug!(client_id = %client_id, "WebSocket close received");
                break;
            }
            Ok(Message::Ping(data)) => {
                // Axum handles pong automatically, but log it
                debug!(client_id = %client_id, "Ping received, len={}", data.len());
            }
            Ok(Message::Pong(_)) => {
                // Ignore pong
            }
            Ok(Message::Binary(_)) => {
                // We don't support binary messages
                debug!(client_id = %client_id, "Binary message ignored");
            }
            Err(e) => {
                error!(client_id = %client_id, error = %e, "WebSocket error");
                break;
            }
        }
    }

    // Clean up
    send_task.abort();
    hub.disconnect(&client_id);
    info!(client_id = %client_id, "WebSocket client disconnected");
}

/// Statistics response.
#[derive(Serialize)]
struct StatsResponse {
    current_connections: usize,
    total_connections: u64,
    total_subscriptions: u64,
    total_events: u64,
}

/// Get real-time connection statistics.
async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.realtime.stats();
    Json(StatsResponse {
        current_connections: stats.current_connections,
        total_connections: stats.total_connections,
        total_subscriptions: stats.total_subscriptions,
        total_events: stats.total_events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_serialization() {
        let stats = StatsResponse {
            current_connections: 10,
            total_connections: 100,
            total_subscriptions: 50,
            total_events: 1000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"current_connections\":10"));
        assert!(json.contains("\"total_events\":1000"));
    }
}

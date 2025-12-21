# Milestone 6: Real-time Updates & Notifications

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 6 implements real-time updates using WebSockets, enabling live notifications and instant updates across the platform. This enhances the user experience by eliminating the need to refresh pages to see new activity.

## Goals

1. **WebSocket Server**: Persistent connections for real-time communication
2. **Event Broadcasting**: Broadcast repository events to connected clients
3. **Notification System**: In-app notifications for subscribed events
4. **Live Updates**: Real-time UI updates for PRs, Issues, Comments, and Repository changes

## Architecture

### New Crate: `guts-realtime`

```
crates/guts-realtime/
├── src/
│   ├── lib.rs           # Public API
│   ├── error.rs         # Error types
│   ├── event.rs         # Event types and serialization
│   ├── hub.rs           # Connection hub (manages all clients)
│   ├── client.rs        # Individual client connection
│   ├── subscription.rs  # Subscription management
│   └── notification.rs  # Notification types
└── Cargo.toml
```

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| WebSocket | tokio-tungstenite | Production-ready, async WebSocket |
| Channels | tokio::sync::broadcast | Efficient multi-consumer broadcasting |
| Serialization | serde_json | JSON messages for browser compatibility |

### WebSocket Protocol

#### Connection URL

```
ws://localhost:8080/ws
wss://localhost:8080/ws  (with TLS)
```

#### Message Types

**Client -> Server:**

```json
// Subscribe to a repository
{
  "type": "subscribe",
  "channel": "repo:alice/myrepo"
}

// Subscribe to user notifications
{
  "type": "subscribe",
  "channel": "user:alice"
}

// Unsubscribe
{
  "type": "unsubscribe",
  "channel": "repo:alice/myrepo"
}

// Ping (keepalive)
{
  "type": "ping"
}
```

**Server -> Client:**

```json
// Subscription confirmed
{
  "type": "subscribed",
  "channel": "repo:alice/myrepo"
}

// Event notification
{
  "type": "event",
  "channel": "repo:alice/myrepo",
  "event": "push",
  "data": {
    "ref": "refs/heads/main",
    "before": "abc123...",
    "after": "def456...",
    "pusher": "alice"
  },
  "timestamp": 1703145600
}

// Pong (keepalive response)
{
  "type": "pong"
}
```

### Channel Types

| Channel Pattern | Description | Example |
|-----------------|-------------|---------|
| `repo:{owner}/{name}` | All events for a repository | `repo:alice/myrepo` |
| `repo:{owner}/{name}/prs` | Pull request events only | `repo:alice/myrepo/prs` |
| `repo:{owner}/{name}/issues` | Issue events only | `repo:alice/myrepo/issues` |
| `user:{username}` | Notifications for a user | `user:alice` |
| `org:{orgname}` | Organization events | `org:acme` |

### Event Types

| Event | Description | Payload |
|-------|-------------|---------|
| `push` | Code pushed to repository | ref, before, after, commits |
| `pr.opened` | Pull request created | pr_number, title, author |
| `pr.closed` | Pull request closed | pr_number, merged |
| `pr.merged` | Pull request merged | pr_number, merge_commit |
| `pr.review` | Review submitted | pr_number, reviewer, state |
| `pr.comment` | Comment on PR | pr_number, comment_id, author |
| `issue.opened` | Issue created | issue_number, title, author |
| `issue.closed` | Issue closed | issue_number |
| `issue.comment` | Comment on issue | issue_number, comment_id |
| `branch.created` | Branch created | ref, sha |
| `branch.deleted` | Branch deleted | ref |

### Data Flow

```
Repository Change (push, PR, issue, etc.)
       │
       ▼
┌─────────────────────────┐
│    guts-node (API)      │
│  ┌───────────────────┐  │
│  │  Event Emitter    │  │
│  └─────────┬─────────┘  │
│            │            │
│  ┌─────────▼─────────┐  │
│  │  guts-realtime    │  │
│  │   ┌───────────┐   │  │
│  │   │ Event Hub │   │  │
│  │   └─────┬─────┘   │  │
│  │         │         │  │
│  │   ┌─────▼─────┐   │  │
│  │   │ Broadcast │   │  │
│  │   └───────────┘   │  │
│  └───────────────────┘  │
└─────────────────────────┘
       │
       ▼ (WebSocket)
┌─────────────────────────┐
│   Connected Clients     │
│   • Browser tabs        │
│   • CLI watchers        │
│   • Integration bots    │
└─────────────────────────┘
```

## Implementation Plan

### Phase 1: Foundation

1. [ ] Create `guts-realtime` crate structure
2. [ ] Implement event types and serialization
3. [ ] Create connection hub for managing clients
4. [ ] Add WebSocket upgrade handler in guts-node
5. [ ] Implement ping/pong keepalive

### Phase 2: Subscriptions

1. [ ] Implement subscription management
2. [ ] Add channel parsing and validation
3. [ ] Create subscription storage per client
4. [ ] Implement unsubscribe logic
5. [ ] Add subscription limits and rate limiting

### Phase 3: Event Broadcasting

1. [ ] Integrate with existing webhook events
2. [ ] Add event emission in API handlers
3. [ ] Implement broadcast to subscribed clients
4. [ ] Add event filtering by channel
5. [ ] Implement event buffering for reconnection

### Phase 4: Web UI Integration

1. [ ] Add WebSocket client JavaScript
2. [ ] Update templates with real-time containers
3. [ ] Implement live update animations
4. [ ] Add connection status indicator
5. [ ] Handle reconnection gracefully

### Phase 5: Notifications

1. [ ] Create notification storage
2. [ ] Implement notification preferences
3. [ ] Add notification badge to UI
4. [ ] Create notification dropdown/panel
5. [ ] Implement mark-as-read functionality

## API Integration

### Event Emission

```rust
// In API handlers, emit events after mutations
async fn create_pull_request(
    State(state): State<AppState>,
    Json(req): Json<CreatePRRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Create the PR
    let pr = state.collaboration.create_pr(...)?;

    // Emit real-time event
    state.realtime.emit(Event::PullRequestOpened {
        repo_key: repo_key.clone(),
        pr_number: pr.number,
        title: pr.title.clone(),
        author: pr.author.clone(),
    });

    Ok(Json(pr))
}
```

### WebSocket Handler

```rust
// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.realtime.clone()))
}

async fn handle_socket(socket: WebSocket, hub: Arc<EventHub>) {
    let (sender, mut receiver) = socket.split();
    let client = hub.connect(sender).await;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    hub.handle_command(&client, cmd).await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    hub.disconnect(&client).await;
}
```

## Success Criteria

- [ ] WebSocket connections work reliably
- [ ] Events broadcast within 100ms of mutation
- [ ] UI updates without page refresh
- [ ] Reconnection handles gracefully (with backoff)
- [ ] 1000+ concurrent connections supported
- [ ] No memory leaks with connection churn
- [ ] Notifications persist across sessions
- [ ] Mobile-friendly notification display

## Dependencies

- `tokio-tungstenite`: WebSocket implementation
- `futures-util`: Stream utilities
- `tokio::sync::broadcast`: Event broadcasting
- `guts-auth`: Webhook event types (reused)
- `guts-collaboration`: PR/Issue types
- `serde_json`: Message serialization

## Security Considerations

1. **Rate Limiting**: Limit subscription count and message rate per client
2. **Channel Authorization**: Verify user has access to subscribed channels
3. **Message Validation**: Sanitize all incoming messages
4. **Connection Limits**: Cap concurrent connections per IP/user
5. **TLS**: Require WSS in production

## Performance Considerations

1. **Connection Pooling**: Efficient memory use per connection
2. **Broadcast Channels**: Use tokio broadcast for O(1) distribution
3. **Message Batching**: Combine rapid events into batches
4. **Lazy Serialization**: Serialize events once, send to many
5. **Backpressure**: Handle slow clients without blocking

## Future Enhancements

These features are out of scope for Milestone 6 but planned:

1. **Push Notifications**: Mobile/desktop push via service workers
2. **Email Digests**: Periodic email summaries of notifications
3. **Mention Detection**: Parse @mentions in comments
4. **Watch Lists**: Follow specific PRs/Issues
5. **Notification Rules**: Custom filtering rules

## References

- [RFC 6455: WebSocket Protocol](https://tools.ietf.org/html/rfc6455)
- [GitHub Real-time Events](https://docs.github.com/en/developers/webhooks-and-events)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [Axum WebSocket Example](https://github.com/tokio-rs/axum/tree/main/examples/websockets)

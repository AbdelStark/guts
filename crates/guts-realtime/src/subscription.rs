//! Subscription management for real-time channels.

use crate::error::RealtimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Maximum subscriptions per client.
pub const MAX_SUBSCRIPTIONS_PER_CLIENT: usize = 100;

/// A channel that clients can subscribe to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Channel {
    /// Channel type.
    pub channel_type: ChannelType,
    /// Channel identifier.
    pub identifier: String,
    /// Optional sub-channel filter.
    pub filter: Option<String>,
}

impl Channel {
    /// Parse a channel string into a Channel.
    ///
    /// Formats:
    /// - `repo:owner/name` - All events for a repository
    /// - `repo:owner/name/prs` - PR events only
    /// - `repo:owner/name/issues` - Issue events only
    /// - `user:username` - User notifications
    /// - `org:orgname` - Organization events
    pub fn parse(s: &str) -> Result<Self, RealtimeError> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(RealtimeError::InvalidChannel(format!(
                "missing channel type prefix: {}",
                s
            )));
        }

        let channel_type = match parts[0] {
            "repo" => ChannelType::Repository,
            "user" => ChannelType::User,
            "org" => ChannelType::Organization,
            _ => {
                return Err(RealtimeError::InvalidChannel(format!(
                    "unknown channel type: {}",
                    parts[0]
                )))
            }
        };

        let identifier_parts: Vec<&str> = parts[1].splitn(3, '/').collect();

        let (identifier, filter) = match channel_type {
            ChannelType::Repository => {
                if identifier_parts.len() < 2 {
                    return Err(RealtimeError::InvalidChannel(format!(
                        "repository channel requires owner/name format: {}",
                        parts[1]
                    )));
                }
                let id = format!("{}/{}", identifier_parts[0], identifier_parts[1]);
                let filter = if identifier_parts.len() > 2 {
                    Some(identifier_parts[2].to_string())
                } else {
                    None
                };
                (id, filter)
            }
            ChannelType::User | ChannelType::Organization => {
                if identifier_parts.is_empty() || identifier_parts[0].is_empty() {
                    return Err(RealtimeError::InvalidChannel(format!(
                        "channel identifier cannot be empty: {}",
                        s
                    )));
                }
                (identifier_parts[0].to_string(), None)
            }
        };

        Ok(Channel {
            channel_type,
            identifier,
            filter,
        })
    }

    /// Check if an event channel matches this subscription channel.
    pub fn matches(&self, event_channel: &str) -> bool {
        let event_chan = match Channel::parse(event_channel) {
            Ok(c) => c,
            Err(_) => return false,
        };

        if self.channel_type != event_chan.channel_type {
            return false;
        }

        if self.identifier != event_chan.identifier {
            return false;
        }

        // If subscription has no filter, it matches all sub-channels
        if self.filter.is_none() {
            return true;
        }

        // If subscription has a filter, event must match exactly
        self.filter == event_chan.filter
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.channel_type {
            ChannelType::Repository => "repo",
            ChannelType::User => "user",
            ChannelType::Organization => "org",
        };

        match &self.filter {
            Some(filter) => write!(f, "{}:{}/{}", prefix, self.identifier, filter),
            None => write!(f, "{}:{}", prefix, self.identifier),
        }
    }
}

/// Channel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// Repository channel (e.g., repo:owner/name).
    Repository,
    /// User notification channel (e.g., user:alice).
    User,
    /// Organization channel (e.g., org:acme).
    Organization,
}

/// Manages subscriptions for a single client.
#[derive(Debug, Default)]
pub struct ClientSubscriptions {
    /// Set of subscribed channels.
    channels: HashSet<Channel>,
}

impl ClientSubscriptions {
    /// Create a new subscription manager.
    pub fn new() -> Self {
        Self {
            channels: HashSet::new(),
        }
    }

    /// Subscribe to a channel.
    pub fn subscribe(&mut self, channel: Channel) -> Result<bool, RealtimeError> {
        if self.channels.len() >= MAX_SUBSCRIPTIONS_PER_CLIENT {
            return Err(RealtimeError::SubscriptionLimit(
                MAX_SUBSCRIPTIONS_PER_CLIENT,
            ));
        }

        Ok(self.channels.insert(channel))
    }

    /// Unsubscribe from a channel.
    pub fn unsubscribe(&mut self, channel: &Channel) -> bool {
        self.channels.remove(channel)
    }

    /// Check if subscribed to a channel.
    pub fn is_subscribed(&self, channel: &Channel) -> bool {
        self.channels.contains(channel)
    }

    /// Check if any subscription matches the event channel.
    pub fn matches_event(&self, event_channel: &str) -> bool {
        self.channels.iter().any(|c| c.matches(event_channel))
    }

    /// Get all subscribed channels.
    pub fn channels(&self) -> impl Iterator<Item = &Channel> {
        self.channels.iter()
    }

    /// Get subscription count.
    pub fn count(&self) -> usize {
        self.channels.len()
    }

    /// Clear all subscriptions.
    pub fn clear(&mut self) {
        self.channels.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_parse_repo() {
        let channel = Channel::parse("repo:alice/myrepo").unwrap();
        assert_eq!(channel.channel_type, ChannelType::Repository);
        assert_eq!(channel.identifier, "alice/myrepo");
        assert_eq!(channel.filter, None);
    }

    #[test]
    fn test_channel_parse_repo_with_filter() {
        let channel = Channel::parse("repo:alice/myrepo/prs").unwrap();
        assert_eq!(channel.channel_type, ChannelType::Repository);
        assert_eq!(channel.identifier, "alice/myrepo");
        assert_eq!(channel.filter, Some("prs".to_string()));
    }

    #[test]
    fn test_channel_parse_user() {
        let channel = Channel::parse("user:alice").unwrap();
        assert_eq!(channel.channel_type, ChannelType::User);
        assert_eq!(channel.identifier, "alice");
        assert_eq!(channel.filter, None);
    }

    #[test]
    fn test_channel_parse_org() {
        let channel = Channel::parse("org:acme").unwrap();
        assert_eq!(channel.channel_type, ChannelType::Organization);
        assert_eq!(channel.identifier, "acme");
        assert_eq!(channel.filter, None);
    }

    #[test]
    fn test_channel_parse_invalid() {
        assert!(Channel::parse("invalid").is_err());
        assert!(Channel::parse("unknown:test").is_err());
        assert!(Channel::parse("repo:").is_err());
        assert!(Channel::parse("repo:onlyname").is_err());
    }

    #[test]
    fn test_channel_to_string() {
        let channel = Channel {
            channel_type: ChannelType::Repository,
            identifier: "alice/myrepo".to_string(),
            filter: None,
        };
        assert_eq!(channel.to_string(), "repo:alice/myrepo");

        let channel_with_filter = Channel {
            channel_type: ChannelType::Repository,
            identifier: "alice/myrepo".to_string(),
            filter: Some("prs".to_string()),
        };
        assert_eq!(channel_with_filter.to_string(), "repo:alice/myrepo/prs");
    }

    #[test]
    fn test_channel_matches() {
        let subscription = Channel::parse("repo:alice/myrepo").unwrap();

        // Should match exact and sub-channels
        assert!(subscription.matches("repo:alice/myrepo"));
        assert!(subscription.matches("repo:alice/myrepo/prs"));
        assert!(subscription.matches("repo:alice/myrepo/issues"));

        // Should not match different repos
        assert!(!subscription.matches("repo:bob/otherrepo"));
        assert!(!subscription.matches("user:alice"));
    }

    #[test]
    fn test_channel_matches_with_filter() {
        let subscription = Channel::parse("repo:alice/myrepo/prs").unwrap();

        // Should only match prs channel
        assert!(subscription.matches("repo:alice/myrepo/prs"));

        // Should not match other channels
        assert!(!subscription.matches("repo:alice/myrepo"));
        assert!(!subscription.matches("repo:alice/myrepo/issues"));
    }

    #[test]
    fn test_client_subscriptions() {
        let mut subs = ClientSubscriptions::new();

        let channel = Channel::parse("repo:alice/myrepo").unwrap();
        assert!(subs.subscribe(channel.clone()).unwrap());
        assert!(subs.is_subscribed(&channel));
        assert_eq!(subs.count(), 1);

        // Duplicate subscription returns false
        assert!(!subs.subscribe(channel.clone()).unwrap());
        assert_eq!(subs.count(), 1);

        assert!(subs.unsubscribe(&channel));
        assert!(!subs.is_subscribed(&channel));
        assert_eq!(subs.count(), 0);
    }

    #[test]
    fn test_client_subscriptions_limit() {
        let mut subs = ClientSubscriptions::new();

        for i in 0..MAX_SUBSCRIPTIONS_PER_CLIENT {
            let channel = Channel::parse(&format!("user:user{}", i)).unwrap();
            subs.subscribe(channel).unwrap();
        }

        let extra = Channel::parse("user:extra").unwrap();
        assert!(matches!(
            subs.subscribe(extra),
            Err(RealtimeError::SubscriptionLimit(_))
        ));
    }

    #[test]
    fn test_matches_event() {
        let mut subs = ClientSubscriptions::new();
        subs.subscribe(Channel::parse("repo:alice/myrepo").unwrap())
            .unwrap();
        subs.subscribe(Channel::parse("user:alice").unwrap())
            .unwrap();

        assert!(subs.matches_event("repo:alice/myrepo"));
        assert!(subs.matches_event("repo:alice/myrepo/prs"));
        assert!(subs.matches_event("user:alice"));

        assert!(!subs.matches_event("repo:bob/otherrepo"));
        assert!(!subs.matches_event("user:bob"));
    }
}

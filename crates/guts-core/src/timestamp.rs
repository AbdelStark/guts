//! Timestamp types for Guts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A Unix timestamp with millisecond precision.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a new `Timestamp` from milliseconds since the Unix epoch.
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Returns the current time as a `Timestamp`.
    #[must_use]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Self(duration.as_millis() as i64)
    }

    /// Returns the timestamp value in milliseconds since the Unix epoch.
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.0
    }

    /// Returns the timestamp value in seconds since the Unix epoch.
    #[must_use]
    pub const fn as_secs(&self) -> i64 {
        self.0 / 1000
    }

    /// Converts this timestamp to a `DateTime<Utc>`.
    #[must_use]
    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.0)
    }

    /// Returns the Unix epoch (1970-01-01 00:00:00 UTC).
    #[must_use]
    pub const fn epoch() -> Self {
        Self(0)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt.timestamp_millis())
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(dt) = self.to_datetime() {
            write!(f, "Timestamp({})", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
        } else {
            write!(f, "Timestamp({})", self.0)
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(dt) = self.to_datetime() {
            write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%SZ"))
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_now() {
        let ts = Timestamp::now();
        assert!(ts.as_millis() > 0);
    }

    #[test]
    fn timestamp_epoch() {
        let ts = Timestamp::epoch();
        assert_eq!(ts.as_millis(), 0);
        assert_eq!(ts.as_secs(), 0);
    }

    #[test]
    fn timestamp_to_datetime() {
        let ts = Timestamp::from_millis(1_700_000_000_000);
        let dt = ts.to_datetime().unwrap();
        assert_eq!(dt.timestamp_millis(), 1_700_000_000_000);
    }
}

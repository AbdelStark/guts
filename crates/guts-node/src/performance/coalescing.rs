//! Request coalescing to deduplicate concurrent identical requests.
//!
//! When multiple clients request the same resource simultaneously,
//! this module ensures only one actual request is made, with the
//! result shared among all waiters.

use futures::future::{BoxFuture, Shared};
use futures::FutureExt;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Configuration for request coalescing.
#[derive(Debug, Clone)]
pub struct CoalescerConfig {
    /// Maximum time to wait for a coalesced request.
    pub max_wait: Duration,
    /// Maximum number of in-flight requests to track.
    pub max_in_flight: usize,
    /// TTL for cached results (if caching enabled).
    pub cache_ttl: Duration,
}

impl Default for CoalescerConfig {
    fn default() -> Self {
        Self {
            max_wait: Duration::from_secs(30),
            max_in_flight: 10_000,
            cache_ttl: Duration::from_secs(5),
        }
    }
}

/// Statistics for request coalescing.
#[derive(Debug, Clone, Default)]
pub struct CoalescerStats {
    /// Total requests received.
    pub total_requests: u64,
    /// Requests that were coalesced.
    pub coalesced_requests: u64,
    /// Requests that triggered new fetches.
    pub new_fetches: u64,
    /// Current number of in-flight requests.
    pub in_flight: usize,
}

impl CoalescerStats {
    /// Returns the coalescing ratio.
    pub fn coalescing_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.coalesced_requests as f64 / self.total_requests as f64
        }
    }
}

struct InFlight<V> {
    future: Shared<BoxFuture<'static, V>>,
    created_at: Instant,
}

/// Request coalescer for deduplicating concurrent requests.
pub struct RequestCoalescer<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone + Send + 'static,
{
    in_flight: Mutex<HashMap<K, InFlight<V>>>,
    config: CoalescerConfig,
    stats: CoalescerStatsInner,
}

struct CoalescerStatsInner {
    total_requests: AtomicU64,
    coalesced_requests: AtomicU64,
    new_fetches: AtomicU64,
}

impl<K, V> RequestCoalescer<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + 'static,
{
    /// Creates a new request coalescer.
    pub fn new(config: CoalescerConfig) -> Self {
        Self {
            in_flight: Mutex::new(HashMap::new()),
            config,
            stats: CoalescerStatsInner {
                total_requests: AtomicU64::new(0),
                coalesced_requests: AtomicU64::new(0),
                new_fetches: AtomicU64::new(0),
            },
        }
    }

    /// Creates a coalescer with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CoalescerConfig::default())
    }

    /// Gets a value, coalescing with any in-flight request for the same key.
    pub async fn get_or_fetch<F, Fut>(&self, key: K, fetch: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = V> + Send + 'static,
    {
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check for existing in-flight request (scope the lock)
        let existing = {
            let in_flight = self.in_flight.lock().await;
            if let Some(entry) = in_flight.get(&key) {
                if entry.created_at.elapsed() < self.config.max_wait {
                    self.stats
                        .coalesced_requests
                        .fetch_add(1, Ordering::Relaxed);
                    Some(entry.future.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If we found an existing request, await it
        if let Some(future) = existing {
            return future.await;
        }

        // Check capacity (scope the lock)
        let at_capacity = {
            let in_flight = self.in_flight.lock().await;
            in_flight.len() >= self.config.max_in_flight
        };

        // If at capacity, just execute directly
        if at_capacity {
            self.stats.new_fetches.fetch_add(1, Ordering::Relaxed);
            return fetch().await;
        }

        // Create new request
        self.stats.new_fetches.fetch_add(1, Ordering::Relaxed);

        let future = fetch().boxed().shared();
        let entry = InFlight {
            future: future.clone(),
            created_at: Instant::now(),
        };

        // Insert into map (scope the lock)
        {
            let mut in_flight = self.in_flight.lock().await;
            in_flight.insert(key.clone(), entry);
        }

        let result = future.await;

        // Clean up (scope the lock)
        {
            let mut in_flight = self.in_flight.lock().await;
            in_flight.remove(&key);
        }

        result
    }

    /// Returns statistics.
    pub fn stats(&self) -> CoalescerStats {
        // Can't get in_flight count without async, return cached stats
        CoalescerStats {
            total_requests: self.stats.total_requests.load(Ordering::Relaxed),
            coalesced_requests: self.stats.coalesced_requests.load(Ordering::Relaxed),
            new_fetches: self.stats.new_fetches.load(Ordering::Relaxed),
            in_flight: 0, // Can't get this synchronously
        }
    }

    /// Returns statistics with in-flight count.
    pub async fn stats_async(&self) -> CoalescerStats {
        let in_flight = self.in_flight.lock().await;
        CoalescerStats {
            total_requests: self.stats.total_requests.load(Ordering::Relaxed),
            coalesced_requests: self.stats.coalesced_requests.load(Ordering::Relaxed),
            new_fetches: self.stats.new_fetches.load(Ordering::Relaxed),
            in_flight: in_flight.len(),
        }
    }

    /// Clears all in-flight requests.
    pub async fn clear(&self) {
        self.in_flight.lock().await.clear();
    }

    /// Removes stale entries (older than max_wait).
    pub async fn cleanup_stale(&self) {
        let mut in_flight = self.in_flight.lock().await;
        in_flight.retain(|_, entry| entry.created_at.elapsed() < self.config.max_wait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_coalescing_basic() {
        let coalescer = RequestCoalescer::<String, u32>::with_defaults();
        let counter = Arc::new(AtomicU32::new(0));

        let key = "test".to_string();

        // First request
        let c1 = Arc::clone(&counter);
        let result = coalescer
            .get_or_fetch(key.clone(), move || async move {
                c1.fetch_add(1, Ordering::Relaxed);
                42
            })
            .await;

        assert_eq!(result, 42);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_concurrent_coalescing() {
        let coalescer = Arc::new(RequestCoalescer::<String, u32>::with_defaults());
        let fetch_count = Arc::new(AtomicU32::new(0));

        let key = "shared".to_string();
        let mut handles = Vec::new();

        // Spawn 10 concurrent requests for the same key
        for _ in 0..10 {
            let coalescer = Arc::clone(&coalescer);
            let key = key.clone();
            let fetch_count = Arc::clone(&fetch_count);

            handles.push(tokio::spawn(async move {
                coalescer
                    .get_or_fetch(key, move || {
                        let fc = Arc::clone(&fetch_count);
                        async move {
                            fc.fetch_add(1, Ordering::Relaxed);
                            sleep(Duration::from_millis(100)).await;
                            42
                        }
                    })
                    .await
            }));
        }

        // Wait for all
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All should get the same result
        assert!(results.iter().all(|&r| r == 42));

        // Only one fetch should have been made
        assert_eq!(fetch_count.load(Ordering::Relaxed), 1);

        let stats = coalescer.stats();
        assert!(stats.coalesced_requests > 0);
    }

    #[tokio::test]
    async fn test_different_keys_not_coalesced() {
        let coalescer = RequestCoalescer::<String, u32>::with_defaults();
        let counter = Arc::new(AtomicU32::new(0));

        for i in 0..3 {
            let key = format!("key-{}", i);
            let c = Arc::clone(&counter);
            coalescer
                .get_or_fetch(key, move || async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    i
                })
                .await;
        }

        // Each key should trigger a fetch
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_stats() {
        let coalescer = RequestCoalescer::<String, u32>::with_defaults();

        let _ = coalescer
            .get_or_fetch("a".to_string(), || async { 1 })
            .await;
        let _ = coalescer
            .get_or_fetch("b".to_string(), || async { 2 })
            .await;

        let stats = coalescer.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.new_fetches, 2);
    }
}

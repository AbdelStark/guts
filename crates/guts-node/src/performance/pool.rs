//! Connection pooling for efficient resource management.
//!
//! Provides a generic connection pool that can be used for
//! database connections, HTTP clients, and other resources.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, SemaphorePermit};

/// Connection pool configuration.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections.
    pub max_connections: usize,
    /// Minimum number of idle connections to maintain.
    pub min_idle: usize,
    /// Connection timeout.
    pub acquire_timeout: Duration,
    /// Idle timeout before a connection is closed.
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection.
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_idle: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

/// Statistics for a connection pool.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Current number of active connections.
    pub active: usize,
    /// Current number of idle connections.
    pub idle: usize,
    /// Total number of connections created.
    pub total_created: u64,
    /// Total number of connections closed.
    pub total_closed: u64,
    /// Number of acquire timeouts.
    pub timeouts: u64,
    /// Number of waiters.
    pub waiters: usize,
}

/// A pooled connection wrapper.
pub struct PooledConnection<T> {
    conn: Option<T>,
    pool: Arc<ConnectionPoolInner<T>>,
    created_at: Instant,
    last_used: Instant,
}

impl<T> PooledConnection<T> {
    /// Returns a reference to the underlying connection.
    pub fn get(&self) -> &T {
        self.conn.as_ref().unwrap()
    }

    /// Returns a mutable reference to the underlying connection.
    pub fn get_mut(&mut self) -> &mut T {
        self.conn.as_mut().unwrap()
    }

    /// Returns how long this connection has been alive.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Returns how long since this connection was last used.
    pub fn idle_time(&self) -> Duration {
        self.last_used.elapsed()
    }
}

impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = Arc::clone(&self.pool);
            let created_at = self.created_at;

            // Check if connection should be returned or discarded
            if created_at.elapsed() < pool.config.max_lifetime {
                pool.return_connection(conn, created_at);
            } else {
                pool.discard_connection();
            }
        }
    }
}

impl<T> std::ops::Deref for PooledConnection<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> std::ops::DerefMut for PooledConnection<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

struct PoolEntry<T> {
    conn: T,
    created_at: Instant,
}

struct ConnectionPoolInner<T> {
    config: PoolConfig,
    available: Mutex<VecDeque<PoolEntry<T>>>,
    semaphore: Semaphore,
    active_count: AtomicUsize,
    total_created: AtomicUsize,
    total_closed: AtomicUsize,
    timeouts: AtomicUsize,
}

impl<T> ConnectionPoolInner<T> {
    fn return_connection(&self, conn: T, created_at: Instant) {
        let mut available = self.available.lock();

        // Check idle timeout and max lifetime
        if created_at.elapsed() < self.config.max_lifetime {
            available.push_back(PoolEntry { conn, created_at });
        } else {
            self.total_closed.fetch_add(1, Ordering::Relaxed);
        }

        drop(available);
        self.active_count.fetch_sub(1, Ordering::Relaxed);
        self.semaphore.add_permits(1);
    }

    fn discard_connection(&self) {
        self.active_count.fetch_sub(1, Ordering::Relaxed);
        self.total_closed.fetch_add(1, Ordering::Relaxed);
        self.semaphore.add_permits(1);
    }
}

/// A generic connection pool.
pub struct ConnectionPool<T, F>
where
    F: Fn() -> T + Send + Sync,
{
    inner: Arc<ConnectionPoolInner<T>>,
    factory: F,
}

impl<T, F> ConnectionPool<T, F>
where
    T: Send + 'static,
    F: Fn() -> T + Send + Sync,
{
    /// Creates a new connection pool.
    pub fn new(factory: F, config: PoolConfig) -> Self {
        let inner = Arc::new(ConnectionPoolInner {
            semaphore: Semaphore::new(config.max_connections),
            config,
            available: Mutex::new(VecDeque::new()),
            active_count: AtomicUsize::new(0),
            total_created: AtomicUsize::new(0),
            total_closed: AtomicUsize::new(0),
            timeouts: AtomicUsize::new(0),
        });

        Self { inner, factory }
    }

    /// Creates a pool with default configuration.
    pub fn with_defaults(factory: F) -> Self {
        Self::new(factory, PoolConfig::default())
    }

    /// Acquires a connection from the pool.
    pub async fn acquire(&self) -> Result<PooledConnection<T>, PoolError> {
        // Try to acquire a permit
        let permit = tokio::time::timeout(
            self.inner.config.acquire_timeout,
            self.inner.semaphore.acquire(),
        )
        .await
        .map_err(|_| {
            self.inner.timeouts.fetch_add(1, Ordering::Relaxed);
            PoolError::Timeout
        })?
        .map_err(|_| PoolError::Closed)?;

        // Forget the permit - we'll add it back when the connection is returned
        permit.forget();

        // Try to get an existing connection
        let entry = {
            let mut available = self.inner.available.lock();

            // Find a valid connection (not expired)
            loop {
                match available.pop_front() {
                    Some(entry) => {
                        if entry.created_at.elapsed() < self.inner.config.max_lifetime {
                            break Some(entry);
                        } else {
                            // Connection expired, try next
                            self.inner.total_closed.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    None => break None,
                }
            }
        };

        let (conn, created_at) = match entry {
            Some(entry) => (entry.conn, entry.created_at),
            None => {
                // Create new connection
                self.inner.total_created.fetch_add(1, Ordering::Relaxed);
                ((self.factory)(), Instant::now())
            }
        };

        self.inner.active_count.fetch_add(1, Ordering::Relaxed);

        Ok(PooledConnection {
            conn: Some(conn),
            pool: Arc::clone(&self.inner),
            created_at,
            last_used: Instant::now(),
        })
    }

    /// Returns pool statistics.
    pub fn stats(&self) -> PoolStats {
        let available = self.inner.available.lock();
        PoolStats {
            active: self.inner.active_count.load(Ordering::Relaxed),
            idle: available.len(),
            total_created: self.inner.total_created.load(Ordering::Relaxed) as u64,
            total_closed: self.inner.total_closed.load(Ordering::Relaxed) as u64,
            timeouts: self.inner.timeouts.load(Ordering::Relaxed) as u64,
            waiters: self.inner.config.max_connections - self.inner.semaphore.available_permits(),
        }
    }

    /// Clears all idle connections.
    pub fn clear_idle(&self) {
        let mut available = self.inner.available.lock();
        let count = available.len();
        available.clear();
        self.inner.total_closed.fetch_add(count, Ordering::Relaxed);
    }
}

/// Pool error types.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PoolError {
    #[error("connection acquisition timed out")]
    Timeout,
    #[error("pool is closed")]
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_pool_acquire_release() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let pool = ConnectionPool::new(
            move || counter_clone.fetch_add(1, Ordering::Relaxed),
            PoolConfig {
                max_connections: 2,
                ..Default::default()
            },
        );

        // Acquire first connection
        let conn1 = pool.acquire().await.unwrap();
        assert_eq!(*conn1, 0);

        // Acquire second connection
        let conn2 = pool.acquire().await.unwrap();
        assert_eq!(*conn2, 1);

        let stats = pool.stats();
        assert_eq!(stats.active, 2);
        assert_eq!(stats.idle, 0);

        // Release first connection
        drop(conn1);

        let stats = pool.stats();
        assert_eq!(stats.active, 1);
        assert_eq!(stats.idle, 1);

        // Acquire should reuse
        let conn3 = pool.acquire().await.unwrap();
        assert_eq!(*conn3, 0); // Reused first connection

        assert_eq!(counter.load(Ordering::Relaxed), 2); // Only 2 created
    }

    #[tokio::test]
    async fn test_pool_timeout() {
        let pool = ConnectionPool::new(
            || 42,
            PoolConfig {
                max_connections: 1,
                acquire_timeout: Duration::from_millis(50),
                ..Default::default()
            },
        );

        // Acquire the only connection
        let _conn = pool.acquire().await.unwrap();

        // Second acquire should timeout
        let result = pool.acquire().await;
        assert!(matches!(result, Err(PoolError::Timeout)));
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ConnectionPool::new(|| (), PoolConfig::default());

        let conn = pool.acquire().await.unwrap();
        let stats = pool.stats();

        assert_eq!(stats.active, 1);
        assert_eq!(stats.total_created, 1);

        drop(conn);

        let stats = pool.stats();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.idle, 1);
    }
}

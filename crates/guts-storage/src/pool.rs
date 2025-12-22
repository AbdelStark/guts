//! Object pooling for reduced allocations.
//!
//! Provides reusable buffer pools to reduce allocation overhead
//! in hot paths like pack file generation and network I/O.

use parking_lot::Mutex;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A pool of reusable objects.
pub struct ObjectPool<T> {
    /// Available objects.
    available: Mutex<Vec<T>>,
    /// Factory function to create new objects.
    factory: Box<dyn Fn() -> T + Send + Sync>,
    /// Maximum pool size.
    max_size: usize,
    /// Number of objects currently borrowed.
    borrowed: AtomicUsize,
    /// Total number of objects created.
    created: AtomicUsize,
}

impl<T> ObjectPool<T> {
    /// Creates a new object pool with the given factory function.
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            available: Mutex::new(Vec::with_capacity(max_size)),
            factory: Box::new(factory),
            max_size,
            borrowed: AtomicUsize::new(0),
            created: AtomicUsize::new(0),
        }
    }

    /// Acquires an object from the pool.
    pub fn acquire(&self) -> PooledObject<'_, T> {
        let obj = self.available.lock().pop().unwrap_or_else(|| {
            self.created.fetch_add(1, Ordering::Relaxed);
            (self.factory)()
        });

        self.borrowed.fetch_add(1, Ordering::Relaxed);

        PooledObject {
            obj: Some(obj),
            pool: self,
        }
    }

    /// Returns an object to the pool.
    fn release(&self, obj: T) {
        self.borrowed.fetch_sub(1, Ordering::Relaxed);

        let mut available = self.available.lock();
        if available.len() < self.max_size {
            available.push(obj);
        }
        // If pool is full, object is dropped
    }

    /// Returns the number of available objects.
    pub fn available(&self) -> usize {
        self.available.lock().len()
    }

    /// Returns the number of borrowed objects.
    pub fn borrowed(&self) -> usize {
        self.borrowed.load(Ordering::Relaxed)
    }

    /// Returns the total number of objects created.
    pub fn created(&self) -> usize {
        self.created.load(Ordering::Relaxed)
    }
}

/// A pooled object that returns to the pool on drop.
pub struct PooledObject<'a, T> {
    obj: Option<T>,
    pool: &'a ObjectPool<T>,
}

impl<T> Deref for PooledObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<T> DerefMut for PooledObject<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<'_, T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            self.pool.release(obj);
        }
    }
}

/// A pooled buffer specifically for byte vectors.
pub type PooledBuffer<'a> = PooledObject<'a, Vec<u8>>;

/// Global buffer pool for pack file generation.
pub static PACK_BUFFER_POOL: std::sync::LazyLock<ObjectPool<Vec<u8>>> =
    std::sync::LazyLock::new(|| {
        ObjectPool::new(
            || Vec::with_capacity(1024 * 1024), // 1 MB buffers
            100,
        )
    });

/// Global buffer pool for network I/O.
pub static IO_BUFFER_POOL: std::sync::LazyLock<ObjectPool<Vec<u8>>> =
    std::sync::LazyLock::new(|| {
        ObjectPool::new(
            || Vec::with_capacity(64 * 1024), // 64 KB buffers
            200,
        )
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_release() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(1024), 10);

        {
            let mut buf = pool.acquire();
            buf.extend_from_slice(b"hello");
            assert_eq!(pool.borrowed(), 1);
        }

        assert_eq!(pool.borrowed(), 0);
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn test_pool_reuse() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(1024), 10);

        {
            let mut buf = pool.acquire();
            buf.extend_from_slice(b"first");
        }

        {
            let buf = pool.acquire();
            // Buffer should be reused (may contain previous data based on Vec behavior)
            assert!(buf.capacity() >= 1024);
        }

        assert_eq!(pool.created(), 1); // Only one buffer was created
    }

    #[test]
    fn test_pool_max_size() {
        let pool: ObjectPool<u32> = ObjectPool::new(|| 0, 2);

        // Acquire 3 objects
        let _a = pool.acquire();
        let _b = pool.acquire();
        let _c = pool.acquire();

        assert_eq!(pool.borrowed(), 3);
        assert_eq!(pool.created(), 3);

        drop(_a);
        drop(_b);
        drop(_c);

        // Only 2 should be pooled
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_pooled_object_deref() {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(Vec::new, 10);

        let mut buf = pool.acquire();
        buf.push(1);
        buf.push(2);

        assert_eq!(buf.len(), 2);
        assert_eq!(&*buf, &[1, 2]);
    }

    #[test]
    fn test_global_pools() {
        let pack_buf = PACK_BUFFER_POOL.acquire();
        assert!(pack_buf.capacity() >= 1024 * 1024);

        let io_buf = IO_BUFFER_POOL.acquire();
        assert!(io_buf.capacity() >= 64 * 1024);
    }
}

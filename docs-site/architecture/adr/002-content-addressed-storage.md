# ADR-002: Content-Addressed Storage for Git Objects

## Status

Accepted

## Date

2025-12-20

## Context

Guts needs to store Git objects (blobs, trees, commits, tags) in a way that:

1. **Deduplicates data**: Identical content should be stored once
2. **Verifies integrity**: Data corruption must be detectable
3. **Enables efficient replication**: Nodes should sync only missing objects
4. **Scales horizontally**: Storage should grow with the network

Git itself uses content-addressed storage where objects are identified by SHA-1 hashes of their content.

## Decision

We will implement content-addressed storage in `guts-storage` crate:

```rust
pub trait ObjectStore: Send + Sync {
    /// Store an object, returning its content hash
    async fn store(&self, data: &[u8]) -> Result<ObjectId>;

    /// Retrieve an object by its hash
    async fn get(&self, id: &ObjectId) -> Result<Option<Vec<u8>>>;

    /// Check if an object exists
    async fn exists(&self, id: &ObjectId) -> Result<bool>;
}
```

Key design choices:

1. **SHA-1 for Git compatibility**: Object IDs use Git's SHA-1 hashing (migration path to SHA-256)
2. **Async interface**: All storage operations are async for non-blocking I/O
3. **Trait-based**: Abstract interface allows multiple backends
4. **Immutable objects**: Once stored, objects are never modified

## Consequences

### Positive

- **Automatic deduplication**: Same content = same hash = stored once
- **Integrity verification**: Re-hash on read catches corruption
- **Simple replication**: "Do you have this hash?" protocol
- **Git compatibility**: Direct mapping to Git object model
- **Cache-friendly**: Objects can be cached indefinitely

### Negative

- **No partial updates**: Changing one byte creates a new object
- **Garbage collection needed**: Unreferenced objects accumulate
- **Hash collisions**: Theoretical risk (mitigated by moving to SHA-256)

### Neutral

- Storage overhead for object headers
- Must track references separately from objects

## Implementation

The current implementation uses in-memory storage with a `HashMap`:

```rust
pub struct InMemoryObjectStore {
    objects: RwLock<HashMap<ObjectId, Vec<u8>>>,
}
```

Future implementations will add:
- Disk-based persistence (likely using RocksDB)
- Network-based storage (fetch from peers on miss)
- Tiered storage (hot/cold separation)

## Alternatives Considered

### Traditional Database

Use PostgreSQL or similar for object storage.

**Rejected because:**
- Overhead for immutable data
- Complex replication setup
- No natural content addressing

### IPFS

Use IPFS as the storage layer.

**Rejected because:**
- Additional runtime dependency
- Different content addressing scheme
- Less control over data locality

### Git Object Format Directly

Store Git pack files as-is.

**Rejected because:**
- Complex delta reconstruction
- Harder to query individual objects
- Still need index for lookups

## References

- [Git Internals - Git Objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [Content-Addressable Storage](https://en.wikipedia.org/wiki/Content-addressable_storage)
- [SHA-1 to SHA-256 Migration](https://git-scm.com/docs/hash-function-transition)

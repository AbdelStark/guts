# ADR-003: Custom Git Smart HTTP Protocol Implementation

## Status

Accepted

## Date

2025-12-20

## Context

Guts must support standard Git operations (clone, push, pull, fetch) to be compatible with existing developer workflows. Git supports multiple transport protocols:

1. **Local protocol**: Direct filesystem access
2. **HTTP(S) Smart protocol**: RESTful HTTP with pack files
3. **SSH protocol**: Secure shell with pack streaming
4. **Git protocol**: TCP-based (legacy, no authentication)

We need to choose and implement a transport protocol that works with decentralized nodes.

## Decision

We will implement the **Git Smart HTTP protocol** in the `guts-git` crate:

### Endpoints

```
GET  /repos/{owner}/{repo}/info/refs?service=git-upload-pack
POST /repos/{owner}/{repo}/git-upload-pack
POST /repos/{owner}/{repo}/git-receive-pack
```

### Key Components

1. **Pktline format**: Git's packet-line framing protocol
2. **Pack files**: Compressed object streams
3. **Reference negotiation**: Capability and wants/haves exchange
4. **Delta compression**: Optional object deltas for efficiency

### Implementation Structure

```rust
// guts-git/src/lib.rs
pub mod pktline;     // Packet-line encoding/decoding
pub mod pack;        // Pack file parsing and generation
pub mod protocol;    // Smart HTTP handlers
pub mod objects;     // Git object types (blob, tree, commit, tag)
```

## Consequences

### Positive

- **Git client compatibility**: Works with `git clone`, `git push`, `git pull`
- **HTTP-based**: Easy to proxy, cache, and secure with TLS
- **Firewall-friendly**: Uses standard HTTP/HTTPS ports
- **Stateless**: Each request is independent (good for load balancing)
- **Well-documented**: Extensive Git protocol documentation

### Negative

- **Complexity**: Pack file format is intricate
- **HTTP overhead**: More bytes than raw git protocol
- **No streaming for HTTP/1.1**: Large pushes buffer in memory

### Neutral

- Requires implementing low-level binary protocols
- Must handle capability negotiation correctly

## Implementation Details

### Pktline Format

```rust
/// Encode data as a pktline (4-byte hex length + data)
pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len() + 4;
    format!("{:04x}", len).into_bytes()
        .into_iter()
        .chain(data.iter().copied())
        .collect()
}

/// Special pktlines
pub const FLUSH: &[u8] = b"0000";      // End of section
pub const DELIM: &[u8] = b"0001";      // Delimiter
pub const RESPONSE_END: &[u8] = b"0002"; // End of response
```

### Pack File Structure

```
PACK signature (4 bytes: "PACK")
Version number (4 bytes, network order)
Object count (4 bytes, network order)
Objects (compressed, possibly deltified)
SHA-1 checksum (20 bytes)
```

## Alternatives Considered

### SSH Protocol

Use SSH for all Git operations.

**Rejected because:**
- Requires SSH key management per node
- Complex to integrate with decentralized identity
- Harder to proxy through web infrastructure

### Git Protocol (TCP 9418)

Use native Git protocol.

**Rejected because:**
- No authentication mechanism
- Non-standard port blocked by firewalls
- Deprecated in favor of Smart HTTP

### Use gitoxide Library

Leverage the gitoxide Rust library for protocol handling.

**Partially adopted:**
- We reference gitoxide patterns
- Custom implementation for tighter control
- May integrate gitoxide in future for specific features

## References

- [Git HTTP Protocol Documentation](https://git-scm.com/docs/http-protocol)
- [Git Pack Protocol](https://git-scm.com/docs/pack-protocol)
- [Git Protocol Capabilities](https://git-scm.com/docs/protocol-capabilities)
- [gitoxide](https://github.com/Byron/gitoxide)

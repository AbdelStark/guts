# Guts Cryptographic Primitives

> Documentation of all cryptographic primitives, their usage, and security properties.

## Cryptographic Inventory

### Overview

| Primitive | Library | Version | Purpose | Status |
|-----------|---------|---------|---------|--------|
| Ed25519 | commonware-cryptography | 0.0.63 | Digital signatures | Production |
| SHA-256 | sha2 | 0.10 | Content addressing | Production |
| Argon2id | argon2 | 0.5 | Token hashing | Production |
| BLAKE3 | blake3 | - | Fast hashing | Optional |
| TLS 1.3 | rustls | 0.21+ | Transport security | Production |
| Noise | snow | - | P2P encryption | Production |

## Digital Signatures (Ed25519)

### Overview

All cryptographic signatures in Guts use Ed25519, a Schnorr signature scheme using the Edwards form of Curve25519.

**Security Properties**:
- 128-bit security level
- Deterministic signatures (no RNG required for signing)
- Fast verification (~15,000 verifications/second)
- Compact signatures (64 bytes) and keys (32 bytes)

### Implementation

```rust
use commonware_cryptography::ed25519::{PrivateKey, PublicKey, Signature};

// Key generation
let private_key = PrivateKey::random(&mut rng);
let public_key = private_key.public_key();

// Signing with domain separation
const NAMESPACE: &[u8] = b"_GUTS";
fn sign_message(key: &PrivateKey, message: &[u8]) -> Signature {
    let prefixed = [NAMESPACE, message].concat();
    key.sign(&prefixed)
}

// Verification
fn verify_signature(
    public_key: &PublicKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    let prefixed = [NAMESPACE, message].concat();
    public_key.verify(&prefixed, signature)
}
```

### Usage Contexts

| Context | Signing Key | Verification |
|---------|-------------|--------------|
| Git commits | User key | Commit verification |
| Node identity | Node key | P2P authentication |
| Consensus messages | Node key | BFT protocol |
| Audit log entries | Node key | Tamper evidence |
| Webhook signatures | Node key | Payload verification |

### Domain Separation

All signatures include domain separation to prevent cross-context replay:

```rust
// Namespace prevents using a signature from one context in another
pub const NAMESPACE: &[u8] = b"_GUTS";
pub const EPOCH: u64 = 0;  // Additional context for consensus
```

### Key Storage

| Environment | Storage Method | Protection |
|-------------|---------------|------------|
| Development | Environment variable | File permissions |
| Production | HSM or Vault | Hardware isolation |
| Backup | Encrypted file | Passphrase + KDF |

## Content Addressing (SHA-256)

### Overview

Git objects are stored using content-addressed storage with SHA-256 hashes (via SHA-1 compatibility mode for Git, transitioning to SHA-256).

**Security Properties**:
- Collision resistance: ~128-bit security
- Preimage resistance: 256-bit security
- Immutability: content cannot be modified without changing hash

### Implementation

```rust
use sha2::{Sha256, Digest};

// Hash git object
fn hash_git_object(object_type: &str, content: &[u8]) -> [u8; 32] {
    let header = format!("{} {}\0", object_type, content.len());

    let mut hasher = Sha256::new();
    hasher.update(header.as_bytes());
    hasher.update(content);
    hasher.finalize().into()
}

// Verify object integrity
fn verify_object(hash: &[u8; 32], content: &[u8]) -> bool {
    let computed = hash_git_object("blob", content);
    constant_time_eq(hash, &computed)
}
```

### Git Object Types

| Type | Header Format | Content |
|------|--------------|---------|
| blob | `blob <size>\0` | File content |
| tree | `tree <size>\0` | Directory entries |
| commit | `commit <size>\0` | Commit metadata + tree |
| tag | `tag <size>\0` | Tag metadata |

## Token Hashing (Argon2id)

### Overview

API tokens are hashed using Argon2id, a memory-hard password hashing function resistant to GPU/ASIC attacks.

**Security Properties**:
- Memory-hard: Requires significant memory to compute
- Time-hard: Configurable iteration count
- Side-channel resistant: Argon2id variant
- Salt: Unique per-token salt

### Implementation

```rust
use argon2::{self, Config, Variant, Version};

// Token hashing parameters (OWASP recommended)
const ARGON2_CONFIG: Config = Config {
    variant: Variant::Argon2id,
    version: Version::Version13,
    mem_cost: 65536,     // 64 MB
    time_cost: 3,        // 3 iterations
    lanes: 4,            // 4 parallel lanes
    secret: &[],
    ad: &[],
    hash_length: 32,
};

// Hash a new token
fn hash_token(token: &str) -> String {
    let salt = generate_random_salt(16);
    argon2::hash_encoded(
        token.as_bytes(),
        &salt,
        &ARGON2_CONFIG,
    ).expect("hashing failed")
}

// Verify token (constant-time)
fn verify_token(token: &str, hash: &str) -> bool {
    argon2::verify_encoded(hash, token.as_bytes())
        .unwrap_or(false)
}
```

### Parameter Recommendations

| Environment | Memory | Time | Parallelism | Rationale |
|-------------|--------|------|-------------|-----------|
| Interactive | 64 MB | 3 | 4 | Balance UX and security |
| Background | 256 MB | 4 | 8 | Higher security for batch ops |
| High security | 1 GB | 5 | 8 | Maximum security |

### Token Lifecycle

```rust
pub struct ApiToken {
    pub id: String,
    pub name: String,
    pub hash: String,        // Argon2id hash
    pub scopes: Vec<Scope>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub last_used_at: Option<u64>,
}
```

## Transport Security (TLS 1.3)

### Overview

All external communications use TLS 1.3 with safe cipher suites.

**Security Properties**:
- Forward secrecy via ephemeral key exchange
- AEAD encryption (AES-256-GCM or ChaCha20-Poly1305)
- Zero round-trip time resumption (0-RTT) disabled for security

### Configuration

```rust
use rustls::{ServerConfig, cipher_suite, version};

fn configure_tls() -> ServerConfig {
    ServerConfig::builder()
        .with_cipher_suites(&[
            cipher_suite::TLS13_AES_256_GCM_SHA384,
            cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
        ])
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&version::TLS13])
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .expect("TLS configuration failed")
}
```

### Certificate Requirements

| Requirement | Specification |
|-------------|--------------|
| Key algorithm | ECDSA P-256 or Ed25519 |
| Key size | 256-bit minimum |
| Signature | SHA-256 or better |
| Validity | 90 days maximum (Let's Encrypt) |
| OCSP | Must staple |

## P2P Encryption (Noise)

### Overview

Node-to-node communication uses the Noise Protocol Framework for authenticated encryption.

**Security Properties**:
- Mutual authentication
- Forward secrecy
- Identity hiding (optional)
- Replay protection

### Handshake Pattern

```
Noise_XX:
  <- s
  ...
  -> e, es, s, ss
  <- e, ee, se

Legend:
  e = ephemeral key
  s = static key
  es/ee/se/ss = DH operations
```

### Implementation

```rust
use snow::{Builder, params::NoiseParams};

const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

fn create_initiator(private_key: &[u8]) -> snow::HandshakeState {
    let params: NoiseParams = NOISE_PARAMS.parse().unwrap();

    Builder::new(params)
        .local_private_key(private_key)
        .build_initiator()
        .expect("failed to build initiator")
}

fn create_responder(private_key: &[u8]) -> snow::HandshakeState {
    let params: NoiseParams = NOISE_PARAMS.parse().unwrap();

    Builder::new(params)
        .local_private_key(private_key)
        .build_responder()
        .expect("failed to build responder")
}
```

## Random Number Generation

### Overview

Cryptographic randomness is provided by the operating system via `getrandom`.

```rust
use rand::{rngs::OsRng, RngCore};

// Generate cryptographic random bytes
fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

// Generate random token
fn generate_token() -> String {
    let bytes = random_bytes::<32>();
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}
```

### Entropy Sources

| Platform | Source | Blocking Behavior |
|----------|--------|-------------------|
| Linux | /dev/urandom | Non-blocking after init |
| macOS | getentropy | Non-blocking |
| Windows | BCryptGenRandom | Non-blocking |

## Key Derivation

### Overview

When deriving keys from passwords or other secrets, use HKDF or Argon2.

```rust
use hkdf::Hkdf;
use sha2::Sha256;

// Derive subkeys from master key
fn derive_key(
    master_key: &[u8],
    context: &[u8],
    output_len: usize,
) -> Vec<u8> {
    let hkdf = Hkdf::<Sha256>::new(None, master_key);

    let mut output = vec![0u8; output_len];
    hkdf.expand(context, &mut output)
        .expect("HKDF expansion failed");
    output
}

// Context strings for key derivation
const ENCRYPTION_CONTEXT: &[u8] = b"guts-encryption-v1";
const SIGNING_CONTEXT: &[u8] = b"guts-signing-v1";
```

## Constant-Time Operations

### Overview

Sensitive comparisons use constant-time operations to prevent timing attacks.

```rust
use subtle::ConstantTimeEq;

// Constant-time byte comparison
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

// Token verification (constant-time)
fn verify_token_constant(token: &str, expected_hash: &str) -> bool {
    // Argon2 verify is internally constant-time
    argon2::verify_encoded(expected_hash, token.as_bytes())
        .unwrap_or(false)
}
```

## Security Recommendations

### Key Management

1. **Store private keys in HSM** for production deployments
2. **Rotate keys every 90 days** with 7-day overlap
3. **Use separate keys** for different purposes (signing, encryption, identity)
4. **Never log or expose** private keys in errors or debug output

### Implementation Guidelines

1. **Always use domain separation** for signatures
2. **Prefer high-level APIs** over low-level crypto primitives
3. **Validate all inputs** before cryptographic operations
4. **Handle errors gracefully** without leaking timing information

### Audit Checklist

- [ ] All signatures use domain separation
- [ ] No custom cryptographic implementations
- [ ] Constant-time comparisons for secrets
- [ ] RNG properly seeded from OS
- [ ] TLS 1.3 only, safe cipher suites
- [ ] Keys rotated according to policy
- [ ] Argon2id with OWASP parameters
- [ ] No secret logging

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-12-21 | Initial cryptographic inventory |

---

*This document is reviewed quarterly and updated with any cryptographic changes.*

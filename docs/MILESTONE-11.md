# Milestone 11: True Decentralization

> **Status:** 🚧 Next
> **Priority:** CRITICAL - This is the essence of the project

## Executive Summary

This milestone transforms Guts from a replicated multi-node system into a truly decentralized, Byzantine fault-tolerant network. The current implementation has P2P messaging but lacks proper consensus, node discovery, and the ability for independent operators to join the network.

**This milestone is the most important in the entire project** - without true decentralization, Guts is just a replicated database. With it, Guts becomes unstoppable infrastructure for code collaboration.

## Current State vs Target State

| Aspect | Current State | Target State |
|--------|---------------|--------------|
| **Consensus** | None (broadcast only) | Simplex BFT consensus |
| **Node Discovery** | Hardcoded bootstrap | Dynamic peer discovery |
| **Validator Set** | Fixed/configured | Permissionless joining |
| **Message Ordering** | None (eventual) | Total ordering via consensus |
| **Fault Tolerance** | None | Tolerates f < n/3 Byzantine nodes |
| **State Agreement** | Optimistic | Cryptographic proof |

## Architecture: Commonware Integration

Based on analysis of the [commonware monorepo](https://github.com/commonwarexyz/monorepo) and the [Alto reference implementation](https://github.com/commonwarexyz/alto), we will integrate the following primitives:

### Core Commonware Crates

| Crate | Purpose | Usage in Guts |
|-------|---------|---------------|
| `commonware-consensus` | BFT message ordering | Order all state-changing operations |
| `commonware-p2p` | Authenticated encrypted networking | Node-to-node communication |
| `commonware-broadcast` | Message dissemination | Gossip for pending transactions |
| `commonware-cryptography` | Ed25519/BLS signatures | Validator identity and signing |
| `commonware-runtime` | Async task execution | Deterministic scheduling |
| `commonware-storage` | Persistent storage | WAL and state persistence |

### Simplex Consensus

The `commonware-consensus::simplex` module implements a BFT consensus algorithm with:

- **2 network hops** for block proposal
- **3 network hops** for finalization
- **f < n/3** Byzantine fault tolerance (3f+1 quorum)
- Partial synchrony model

**Protocol Messages:**
1. `Notarize(c, v)` - Proposals for containers in view v
2. `Nullify(v)` - Votes when leaders are unresponsive
3. `Finalize(c, v)` - Final commitment votes

**Key Traits to Implement:**
```rust
/// Application interface for consensus
trait Application {
    /// Called when a block is finalized
    fn finalized(&mut self, block: Block);
}

/// Automaton drives consensus forward
trait Automaton {
    /// Propose a new payload
    fn propose(&mut self, context: Context) -> Option<Payload>;

    /// Verify a proposed payload
    fn verify(&self, payload: &Payload) -> bool;
}

/// Relay broadcasts payloads to the network
trait Relay {
    /// Broadcast a message to all peers
    fn broadcast(&self, message: Vec<u8>);
}
```

## Implementation Plan

### Phase 1: Consensus Engine Foundation (Week 1-2)

#### 1.1 Create `guts-consensus` Crate

```
crates/guts-consensus/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API
    ├── engine.rs           # Main consensus engine
    ├── application.rs      # Application trait implementation
    ├── automaton.rs        # Block proposal/verification
    ├── block.rs            # Block structure
    ├── transaction.rs      # Transaction types
    ├── validator.rs        # Validator set management
    └── config.rs           # Configuration
```

#### 1.2 Define Transaction Types

All state-changing operations become consensus transactions:

```rust
/// Transactions that require consensus ordering
#[derive(Clone, Serialize, Deserialize)]
pub enum Transaction {
    // Git operations
    GitPush {
        repo_key: String,
        ref_name: String,
        old_oid: ObjectId,
        new_oid: ObjectId,
        objects: Vec<ObjectId>,
        signature: Signature,
    },

    // Repository management
    CreateRepository {
        owner: String,
        name: String,
        creator: PublicKey,
        signature: Signature,
    },

    // Collaboration
    CreatePullRequest {
        repo_key: String,
        pr: SerializablePullRequest,
        signature: Signature,
    },
    UpdatePullRequest {
        repo_key: String,
        pr_number: u64,
        update: PullRequestUpdate,
        signature: Signature,
    },
    MergePullRequest {
        repo_key: String,
        pr_number: u64,
        merge_commit: ObjectId,
        signature: Signature,
    },

    CreateIssue {
        repo_key: String,
        issue: SerializableIssue,
        signature: Signature,
    },
    UpdateIssue {
        repo_key: String,
        issue_number: u64,
        update: IssueUpdate,
        signature: Signature,
    },

    CreateComment {
        repo_key: String,
        target: CommentTarget,
        comment: SerializableComment,
        signature: Signature,
    },

    CreateReview {
        repo_key: String,
        pr_number: u64,
        review: SerializableReview,
        signature: Signature,
    },

    // Governance
    CreateOrganization {
        org: SerializableOrg,
        signature: Signature,
    },
    UpdateOrganization {
        org_id: String,
        update: OrgUpdate,
        signature: Signature,
    },

    CreateTeam {
        org_id: String,
        team: SerializableTeam,
        signature: Signature,
    },

    UpdatePermissions {
        repo_key: String,
        user: String,
        permission: Permission,
        signature: Signature,
    },

    // Branch protection
    SetBranchProtection {
        repo_key: String,
        branch: String,
        protection: BranchProtection,
        signature: Signature,
    },
}
```

#### 1.3 Block Structure

```rust
/// A block in the Guts consensus chain
#[derive(Clone, Serialize, Deserialize)]
pub struct GutsBlock {
    /// Block height
    pub height: u64,

    /// Previous block hash
    pub parent: [u8; 32],

    /// Block producer (validator public key)
    pub producer: PublicKey,

    /// Timestamp (unix millis)
    pub timestamp: u64,

    /// Ordered transactions
    pub transactions: Vec<Transaction>,

    /// Merkle root of transactions
    pub tx_root: [u8; 32],

    /// State root after applying transactions
    pub state_root: [u8; 32],
}
```

#### 1.4 Implement Application Trait

```rust
/// Guts application implementing consensus interface
pub struct GutsApplication {
    /// Current state (repositories, collaboration, auth)
    state: Arc<RwLock<AppState>>,

    /// Pending transaction pool
    mempool: Arc<RwLock<Mempool>>,

    /// Block storage
    blocks: Arc<dyn BlockStore>,

    /// Event broadcaster for real-time updates
    events: broadcast::Sender<Event>,
}

impl Application for GutsApplication {
    fn finalized(&mut self, block: Block) {
        // Apply each transaction in order
        let mut state = self.state.write();

        for tx in block.transactions() {
            match self.apply_transaction(&mut state, tx) {
                Ok(events) => {
                    // Broadcast events for real-time updates
                    for event in events {
                        let _ = self.events.send(event);
                    }
                }
                Err(e) => {
                    // Transaction failed - this shouldn't happen
                    // if verification was correct
                    tracing::error!(?tx, ?e, "Transaction failed");
                }
            }
        }

        // Persist block
        self.blocks.put(block);
    }
}
```

### Phase 2: P2P Network Layer (Week 2-3)

#### 2.1 Enhance `guts-p2p` with Commonware Authenticated Networking

```rust
use commonware_p2p::authenticated::{Config, Network};
use commonware_cryptography::ed25519::Keypair;

/// P2P network manager
pub struct P2PNetwork {
    /// Network instance
    network: Network,

    /// Our validator keypair
    keypair: Keypair,

    /// Connected peers
    peers: Arc<RwLock<HashMap<PublicKey, PeerInfo>>>,

    /// Channel senders for different message types
    channels: Channels,
}

/// Communication channels
pub struct Channels {
    /// Consensus messages (highest priority)
    pub consensus: Channel,

    /// Block/transaction broadcasts
    pub broadcast: Channel,

    /// Object sync requests/responses
    pub sync: Channel,

    /// Peer discovery
    pub discovery: Channel,
}

impl P2PNetwork {
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        let keypair = Keypair::from_seed(&config.seed)?;

        let network_config = Config {
            keypair: keypair.clone(),
            listen_addr: config.listen_addr,
            max_peers: config.max_peers,
            ..Default::default()
        };

        let network = Network::new(network_config).await?;

        Ok(Self {
            network,
            keypair,
            peers: Arc::new(RwLock::new(HashMap::new())),
            channels: Channels::new(),
        })
    }

    /// Bootstrap by connecting to known nodes
    pub async fn bootstrap(&mut self, bootstrap_nodes: &[SocketAddr]) -> Result<()> {
        for addr in bootstrap_nodes {
            match self.network.connect(*addr).await {
                Ok(peer_id) => {
                    tracing::info!(?addr, ?peer_id, "Connected to bootstrap node");
                    self.request_peers(peer_id).await?;
                }
                Err(e) => {
                    tracing::warn!(?addr, ?e, "Failed to connect to bootstrap node");
                }
            }
        }
        Ok(())
    }

    /// Request peer list from a connected node
    async fn request_peers(&mut self, peer: PublicKey) -> Result<Vec<PeerInfo>> {
        let request = DiscoveryMessage::GetPeers;
        let response = self.network.request(&peer, request.encode()).await?;
        let peers: Vec<PeerInfo> = DiscoveryMessage::decode(&response)?.into_peers()?;

        // Connect to new peers
        for info in &peers {
            if !self.peers.read().contains_key(&info.pubkey) {
                self.network.connect(info.addr).await?;
            }
        }

        Ok(peers)
    }
}
```

#### 2.2 Bootstrap Node Discovery

```rust
/// Bootstrap configuration
pub struct BootstrapConfig {
    /// Well-known bootstrap nodes (DNS or IP)
    pub bootstrap_nodes: Vec<String>,

    /// Local network discovery (mDNS-like)
    pub enable_local_discovery: bool,

    /// Maximum peers to maintain
    pub max_peers: usize,

    /// Peer exchange interval
    pub peer_exchange_interval: Duration,
}

/// Peer discovery service
pub struct Discovery {
    network: Arc<P2PNetwork>,
    config: BootstrapConfig,
    known_peers: Arc<RwLock<HashSet<PeerInfo>>>,
}

impl Discovery {
    /// Run the discovery loop
    pub async fn run(&self) -> Result<()> {
        // Initial bootstrap
        self.bootstrap().await?;

        // Periodic peer exchange
        let mut interval = tokio::time::interval(self.config.peer_exchange_interval);

        loop {
            interval.tick().await;

            // Exchange peers with connected nodes
            let peers: Vec<_> = self.network.peers.read().keys().cloned().collect();
            for peer in peers {
                if let Ok(new_peers) = self.network.request_peers(peer).await {
                    for info in new_peers {
                        self.known_peers.write().insert(info);
                    }
                }
            }

            // Ensure minimum peer count
            self.maintain_connections().await?;
        }
    }

    async fn maintain_connections(&self) -> Result<()> {
        let current_count = self.network.peers.read().len();

        if current_count < self.config.max_peers / 2 {
            // Need more peers
            let known = self.known_peers.read().clone();
            for info in known {
                if !self.network.peers.read().contains_key(&info.pubkey) {
                    if let Ok(_) = self.network.connect(info.addr).await {
                        if self.network.peers.read().len() >= self.config.max_peers {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
```

### Phase 3: Validator Set Management (Week 3-4)

#### 3.1 Validator Registration

For the initial implementation, we use a simple permissioned validator set that can be expanded later:

```rust
/// Validator set management
pub struct ValidatorSet {
    /// Current validators
    validators: Vec<Validator>,

    /// Epoch number (changes when validator set changes)
    epoch: u64,

    /// Configuration
    config: ValidatorConfig,
}

#[derive(Clone)]
pub struct Validator {
    /// Validator public key
    pub pubkey: PublicKey,

    /// Voting weight
    pub weight: u64,

    /// Network address
    pub addr: SocketAddr,

    /// Joined at epoch
    pub joined_epoch: u64,

    /// Is active (participating in consensus)
    pub active: bool,
}

#[derive(Clone)]
pub struct ValidatorConfig {
    /// Minimum validators for network operation
    pub min_validators: usize,

    /// Maximum validators
    pub max_validators: usize,

    /// Quorum threshold (2/3 + 1)
    pub quorum_threshold: f64,

    /// Block time target
    pub block_time: Duration,
}

impl ValidatorSet {
    /// Create genesis validator set
    pub fn genesis(validators: Vec<Validator>) -> Self {
        Self {
            validators,
            epoch: 0,
            config: ValidatorConfig::default(),
        }
    }

    /// Get quorum weight required for consensus
    pub fn quorum_weight(&self) -> u64 {
        let total: u64 = self.validators.iter().map(|v| v.weight).sum();
        (total * 2 / 3) + 1
    }

    /// Get validators for current epoch
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Check if a public key is a validator
    pub fn is_validator(&self, pubkey: &PublicKey) -> bool {
        self.validators.iter().any(|v| v.pubkey == *pubkey)
    }

    /// Get leader for a given view (round-robin initially)
    pub fn leader_for_view(&self, view: u64) -> &Validator {
        let active: Vec<_> = self.validators.iter().filter(|v| v.active).collect();
        let idx = (view as usize) % active.len();
        active[idx]
    }
}
```

#### 3.2 Genesis Configuration

```rust
/// Genesis configuration for the network
#[derive(Clone, Serialize, Deserialize)]
pub struct Genesis {
    /// Network identifier
    pub chain_id: String,

    /// Genesis timestamp
    pub timestamp: u64,

    /// Initial validators
    pub validators: Vec<GenesisValidator>,

    /// Initial repositories (optional, for testnet)
    pub repositories: Vec<GenesisRepository>,

    /// Consensus parameters
    pub consensus: ConsensusParams,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    pub name: String,
    pub pubkey: String,  // hex-encoded
    pub weight: u64,
    pub addr: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConsensusParams {
    /// Target block time in milliseconds
    pub block_time_ms: u64,

    /// Maximum transactions per block
    pub max_txs_per_block: usize,

    /// Maximum block size in bytes
    pub max_block_size: usize,

    /// View timeout multiplier
    pub view_timeout_multiplier: f64,
}

impl Genesis {
    /// Load from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let genesis: Genesis = serde_json::from_str(&content)?;
        genesis.validate()?;
        Ok(genesis)
    }

    /// Validate genesis configuration
    pub fn validate(&self) -> Result<()> {
        if self.validators.is_empty() {
            return Err(Error::InvalidGenesis("No validators"));
        }

        if self.validators.len() < 4 {
            return Err(Error::InvalidGenesis("Need at least 4 validators for BFT"));
        }

        // Verify all public keys are valid
        for v in &self.validators {
            PublicKey::from_hex(&v.pubkey)?;
        }

        Ok(())
    }
}
```

### Phase 4: Consensus Integration (Week 4-5)

#### 4.1 Wire Up Consensus Engine

```rust
/// Main node with consensus
pub struct ConsensusNode {
    /// Network layer
    network: Arc<P2PNetwork>,

    /// Consensus engine
    consensus: SimplexConsensus,

    /// Application state
    application: GutsApplication,

    /// Validator set
    validators: Arc<RwLock<ValidatorSet>>,

    /// Transaction mempool
    mempool: Arc<RwLock<Mempool>>,

    /// HTTP API server
    api: ApiServer,
}

impl ConsensusNode {
    pub async fn new(config: NodeConfig) -> Result<Self> {
        // Load genesis
        let genesis = Genesis::load(&config.genesis_path)?;

        // Initialize network
        let network = Arc::new(P2PNetwork::new(config.network).await?);

        // Initialize validator set from genesis
        let validators = Arc::new(RwLock::new(
            ValidatorSet::from_genesis(&genesis)?
        ));

        // Initialize application state
        let application = GutsApplication::new(config.storage).await?;

        // Initialize mempool
        let mempool = Arc::new(RwLock::new(Mempool::new(config.mempool)));

        // Initialize consensus
        let consensus = SimplexConsensus::new(
            config.consensus,
            network.clone(),
            validators.clone(),
        )?;

        // Initialize API server
        let api = ApiServer::new(config.api, application.clone(), mempool.clone())?;

        Ok(Self {
            network,
            consensus,
            application,
            validators,
            mempool,
            api,
        })
    }

    /// Run the node
    pub async fn run(&mut self) -> Result<()> {
        // Bootstrap network
        self.network.bootstrap(&self.config.bootstrap_nodes).await?;

        // Wait for minimum peers
        self.wait_for_peers().await?;

        // Start all components
        tokio::select! {
            result = self.consensus.run() => {
                tracing::error!(?result, "Consensus exited");
            }
            result = self.network.run() => {
                tracing::error!(?result, "Network exited");
            }
            result = self.api.run() => {
                tracing::error!(?result, "API exited");
            }
        }

        Ok(())
    }
}
```

#### 4.2 Transaction Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Client     │────▶│   API Layer  │────▶│   Mempool    │
│  (git push)  │     │  (validate)  │     │  (pending)   │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                     ┌────────────────────────────┘
                     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Broadcast   │◀────│   Leader     │◀────│  Consensus   │
│  to Peers    │     │  Proposes    │     │   Selects    │
└──────────────┘     │   Block      │     │    Leader    │
                     └──────┬───────┘     └──────────────┘
                            │
                            ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Validators  │────▶│  Notarize    │────▶│   Finalize   │
│    Vote      │     │  (2f+1)      │     │   (2f+1)     │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                                  ▼
                     ┌──────────────────────────────────┐
                     │         Apply to State           │
                     │  (git refs, PRs, issues, etc.)   │
                     └──────────────────────────────────┘
```

### Phase 5: Testing & Validation (Week 5-6)

#### 5.1 Consensus Tests

```rust
#[tokio::test]
async fn test_consensus_finalization() {
    // Start 4 validators (tolerates 1 Byzantine)
    let network = TestNetwork::new(4).await;

    // Submit a transaction
    let tx = Transaction::CreateRepository {
        owner: "alice".to_string(),
        name: "test-repo".to_string(),
        creator: network.validator(0).pubkey(),
        signature: network.validator(0).sign(/* ... */),
    };

    network.submit_transaction(tx).await;

    // Wait for finalization
    let block = network.wait_for_block(1, Duration::from_secs(10)).await;

    // Verify all nodes have the same state
    for i in 0..4 {
        let state = network.node(i).state().await;
        assert!(state.has_repository("alice/test-repo"));
    }
}

#[tokio::test]
async fn test_byzantine_tolerance() {
    // Start 4 validators
    let network = TestNetwork::new(4).await;

    // Make one validator Byzantine (doesn't vote)
    network.set_byzantine(3, ByzantineBehavior::Silent);

    // Consensus should still work with 3 honest validators
    let tx = Transaction::CreateRepository { /* ... */ };
    network.submit_transaction(tx).await;

    // Should finalize despite Byzantine node
    let block = network.wait_for_block(1, Duration::from_secs(15)).await;
    assert!(block.is_some());
}

#[tokio::test]
async fn test_leader_rotation() {
    let network = TestNetwork::new(4).await;

    // Generate multiple blocks
    for i in 0..10 {
        let tx = Transaction::CreateRepository {
            name: format!("repo-{}", i),
            /* ... */
        };
        network.submit_transaction(tx).await;
    }

    // Wait for blocks
    network.wait_for_block(10, Duration::from_secs(30)).await;

    // Verify different leaders proposed blocks
    let blocks: Vec<_> = network.node(0).blocks(0..10).await;
    let leaders: HashSet<_> = blocks.iter().map(|b| b.producer).collect();

    // Should have multiple different leaders
    assert!(leaders.len() > 1);
}
```

#### 5.2 Network Partition Tests

```rust
#[tokio::test]
async fn test_network_partition_recovery() {
    let network = TestNetwork::new(7).await;

    // Create some state
    network.submit_transaction(/* ... */).await;
    network.wait_for_block(1, Duration::from_secs(10)).await;

    // Partition network: [0,1,2,3] and [4,5,6]
    network.partition(vec![vec![0,1,2,3], vec![4,5,6]]);

    // Neither partition has quorum (need 5 of 7)
    // No new blocks should finalize

    // Heal partition
    network.heal_partition();

    // Submit new transaction
    network.submit_transaction(/* ... */).await;

    // Should eventually finalize
    let block = network.wait_for_block(2, Duration::from_secs(30)).await;
    assert!(block.is_some());
}
```

### Phase 6: DevNet & Documentation (Week 6-7)

#### 6.1 DevNet Configuration

```yaml
# infra/devnet/genesis.yaml
chain_id: "guts-devnet-1"
timestamp: 1703980800000

validators:
  - name: "validator-1"
    pubkey: "ed25519:abc123..."
    weight: 100
    addr: "validator-1:9000"

  - name: "validator-2"
    pubkey: "ed25519:def456..."
    weight: 100
    addr: "validator-2:9000"

  - name: "validator-3"
    pubkey: "ed25519:ghi789..."
    weight: 100
    addr: "validator-3:9000"

  - name: "validator-4"
    pubkey: "ed25519:jkl012..."
    weight: 100
    addr: "validator-4:9000"

consensus:
  block_time_ms: 2000
  max_txs_per_block: 1000
  max_block_size: 10485760  # 10 MB
  view_timeout_multiplier: 2.0
```

#### 6.2 Docker Compose for DevNet

```yaml
# infra/docker/docker-compose.consensus.yml
version: '3.8'

services:
  validator-1:
    build: ../..
    command:
      - guts-node
      - --genesis=/config/genesis.yaml
      - --validator-key=/keys/validator-1.key
      - --api-addr=0.0.0.0:8080
      - --p2p-addr=0.0.0.0:9000
    volumes:
      - ./config:/config
      - ./keys:/keys
      - validator-1-data:/data
    ports:
      - "8081:8080"
      - "9001:9000"
    networks:
      - guts-network

  validator-2:
    build: ../..
    command:
      - guts-node
      - --genesis=/config/genesis.yaml
      - --validator-key=/keys/validator-2.key
      - --bootstrap=validator-1:9000
      - --api-addr=0.0.0.0:8080
      - --p2p-addr=0.0.0.0:9000
    volumes:
      - ./config:/config
      - ./keys:/keys
      - validator-2-data:/data
    ports:
      - "8082:8080"
      - "9002:9000"
    networks:
      - guts-network
    depends_on:
      - validator-1

  # ... validators 3 and 4 similar

volumes:
  validator-1-data:
  validator-2-data:
  validator-3-data:
  validator-4-data:

networks:
  guts-network:
    driver: bridge
```

## Success Criteria

### Must Have (P0)

- [ ] Simplex BFT consensus integrated and working
- [ ] 4+ node devnet with consensus achieving finality
- [ ] Git push/pull works through consensus
- [ ] PRs, issues, comments ordered by consensus
- [ ] Network tolerates 1 Byzantine node (in 4-node setup)
- [ ] Nodes can bootstrap from peers
- [ ] State is consistent across all honest nodes

### Should Have (P1)

- [ ] 7+ node devnet for better fault tolerance
- [ ] Node can sync from scratch (catch up to current state)
- [ ] Metrics for consensus (block time, finality latency)
- [ ] Prometheus dashboards for monitoring
- [ ] E2E tests for consensus scenarios

### Nice to Have (P2)

- [ ] Dynamic validator set changes
- [ ] Testnet with 10+ independent operators
- [ ] Geographic distribution testing
- [ ] Chaos testing with network partitions

## Timeline

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | Foundation | `guts-consensus` crate skeleton, transaction types |
| 2 | Consensus | Simplex integration, block structure |
| 3 | Networking | P2P authentication, bootstrap, peer discovery |
| 4 | Validators | Validator set, genesis, leader election |
| 5 | Integration | Wire up API -> mempool -> consensus -> application |
| 6 | Testing | Consensus tests, Byzantine tests, partition tests |
| 7 | DevNet | Docker compose, monitoring, documentation |

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Commonware API changes | Medium | High | Pin to specific version, wrap primitives |
| Performance issues | Medium | Medium | Benchmark early, optimize block size |
| Consensus bugs | Low | Critical | Extensive testing, formal verification later |
| Network complexity | High | Medium | Start simple, iterate on discovery |

## Dependencies

- `commonware-consensus` v0.0.64+
- `commonware-p2p` v0.0.64+
- `commonware-broadcast` v0.0.64+
- `commonware-cryptography` v0.0.64+
- `commonware-runtime` v0.0.64+

## References

- [Simplex Consensus Paper](https://eprint.iacr.org/2023/463) - Original BFT algorithm
- [Alto Blockchain](https://github.com/commonwarexyz/alto) - Reference implementation
- [Commonware Docs](https://docs.rs/commonware-consensus) - API documentation
- [ADR-001](adr/001-commonware-primitives.md) - Decision to use Commonware

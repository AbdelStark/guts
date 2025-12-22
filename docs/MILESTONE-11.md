# Milestone 11: True Decentralization

> **Status:** Planned
> **Target:** Q2 2025
> **Priority:** Critical

## Overview

Milestone 12 transforms Guts from a replicated system into a truly decentralized, permissionless network. Currently, Guts requires pre-configured bootstrap nodes and has no mechanism for independent operators to join the network. This milestone implements trustless peer discovery, validator governance, Sybil resistance, and launches a public testnet with independent operators.

## Goals

1. **Permissionless Node Discovery**: DHT-based peer discovery without centralized bootstrap
2. **Validator Governance**: On-chain mechanisms for validator set management
3. **Sybil Resistance**: Stake-based or proof-of-work based protection
4. **Gossip Protocol**: Efficient message propagation at scale
5. **Network Partitioning**: Graceful handling and recovery from network splits
6. **Public Testnet**: Launch with 20+ independent operators
7. **Multi-Region**: Demonstrate global network operation

## Current Limitations

| Aspect | Current State | Target State |
|--------|---------------|--------------|
| Node Discovery | Static bootstrap list | DHT-based discovery |
| Validator Set | Fixed configuration | Dynamic governance |
| Sybil Protection | None | Stake or PoW |
| Message Propagation | Direct broadcast | Gossip protocol |
| Partition Handling | Untested | Automatic recovery |
| Network Type | Private/permissioned | Public/permissionless |

## Architecture

### New Components

```
crates/guts-p2p/
├── src/
│   ├── discovery/
│   │   ├── mod.rs           # Discovery module
│   │   ├── dht.rs           # Kademlia DHT implementation
│   │   ├── bootstrap.rs     # Bootstrap node handling
│   │   └── mdns.rs          # Local network discovery
│   ├── gossip/
│   │   ├── mod.rs           # Gossip module
│   │   ├── epidemic.rs      # Epidemic broadcast
│   │   ├── plumtree.rs      # Plumtree hybrid gossip
│   │   └── mesh.rs          # Mesh-based gossip
│   ├── governance/
│   │   ├── mod.rs           # Governance module
│   │   ├── validator_set.rs # Validator set management
│   │   ├── staking.rs       # Stake management
│   │   └── voting.rs        # On-chain voting
│   └── sybil/
│       ├── mod.rs           # Sybil resistance
│       ├── stake.rs         # Proof of stake
│       └── pow.rs           # Proof of work (optional)
```

### Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Guts Decentralized Network                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐  │
│   │ Region 1│     │ Region 2│     │ Region 3│     │ Region 4│  │
│   │ (US-E)  │     │ (EU-W)  │     │ (APAC)  │     │ (SA)    │  │
│   └────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘  │
│        │               │               │               │        │
│        └───────────────┼───────────────┼───────────────┘        │
│                        │               │                         │
│              ┌─────────┴───────────────┴─────────┐              │
│              │       Kademlia DHT Network         │              │
│              │   (Peer Discovery & Routing)       │              │
│              └─────────────────────────────────────┘              │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                   Gossip Layer                           │   │
│   │   Plumtree for consensus messages, epidemic for data    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │              BFT Consensus (Validators Only)            │   │
│   │        Stake-weighted voting, dynamic validator set     │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Detailed Implementation

### Phase 1: DHT-Based Peer Discovery

#### 1.1 Kademlia Implementation

```rust
// crates/guts-p2p/src/discovery/dht.rs

use libp2p::kad::{Kademlia, KademliaConfig, QueryResult};

pub struct DhtDiscovery {
    /// Kademlia DHT instance
    kademlia: Kademlia<MemoryStore>,

    /// Our peer ID
    local_peer_id: PeerId,

    /// Bootstrap nodes (used only on first start)
    bootstrap_nodes: Vec<Multiaddr>,

    /// Discovery configuration
    config: DhtConfig,
}

pub struct DhtConfig {
    /// Bucket size (k parameter)
    pub k: usize,

    /// Parallelism factor (alpha parameter)
    pub alpha: usize,

    /// Record TTL
    pub record_ttl: Duration,

    /// Provider record TTL
    pub provider_ttl: Duration,

    /// Replication interval
    pub replication_interval: Duration,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            k: 20,
            alpha: 3,
            record_ttl: Duration::from_secs(36 * 3600),  // 36 hours
            provider_ttl: Duration::from_secs(24 * 3600), // 24 hours
            replication_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl DhtDiscovery {
    pub async fn new(
        local_key: identity::Keypair,
        bootstrap_nodes: Vec<Multiaddr>,
        config: DhtConfig,
    ) -> Result<Self> {
        let local_peer_id = PeerId::from(local_key.public());

        let mut kad_config = KademliaConfig::default();
        kad_config.set_kbucket_inserts(KademliaBucketInserts::OnConnected);
        kad_config.set_record_ttl(Some(config.record_ttl));
        kad_config.set_provider_record_ttl(Some(config.provider_ttl));
        kad_config.set_replication_interval(Some(config.replication_interval));

        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::with_config(local_peer_id, store, kad_config);

        Ok(Self {
            kademlia,
            local_peer_id,
            bootstrap_nodes,
            config,
        })
    }

    /// Bootstrap the DHT by connecting to known nodes
    pub async fn bootstrap(&mut self) -> Result<()> {
        // Add bootstrap nodes to routing table
        for addr in &self.bootstrap_nodes {
            if let Some(peer_id) = extract_peer_id(addr) {
                self.kademlia.add_address(&peer_id, addr.clone());
            }
        }

        // Perform bootstrap query
        self.kademlia.bootstrap()?;

        Ok(())
    }

    /// Announce ourselves as a Guts node
    pub async fn announce(&mut self) -> Result<()> {
        // Announce as provider for "guts-network" key
        let key = Key::new(&b"guts-network");
        self.kademlia.start_providing(key)?;

        // Announce our services
        self.announce_services().await?;

        Ok(())
    }

    /// Find other Guts nodes
    pub async fn find_peers(&mut self) -> Result<Vec<PeerInfo>> {
        let key = Key::new(&b"guts-network");
        self.kademlia.get_providers(key);

        // Wait for providers
        let providers = self.collect_providers().await?;

        Ok(providers)
    }

    /// Find nodes providing a specific repository
    pub async fn find_repo_providers(&mut self, repo_key: &RepoKey) -> Result<Vec<PeerInfo>> {
        let key = Key::new(repo_key.as_bytes());
        self.kademlia.get_providers(key);

        self.collect_providers().await
    }

    /// Announce that we have a repository
    pub async fn announce_repo(&mut self, repo_key: &RepoKey) -> Result<()> {
        let key = Key::new(repo_key.as_bytes());
        self.kademlia.start_providing(key)?;
        Ok(())
    }
}
```

#### 1.2 Multi-Address Support

```rust
/// Support multiple network transports
pub struct MultiTransport {
    /// TCP transport
    tcp: TcpTransport,

    /// QUIC transport (preferred)
    quic: QuicTransport,

    /// WebSocket transport (for browser nodes)
    websocket: WebSocketTransport,
}

impl MultiTransport {
    pub fn listen_addresses(&self) -> Vec<Multiaddr> {
        vec![
            // TCP
            format!("/ip4/0.0.0.0/tcp/{}", self.tcp_port).parse().unwrap(),
            // QUIC
            format!("/ip4/0.0.0.0/udp/{}/quic-v1", self.quic_port).parse().unwrap(),
            // WebSocket
            format!("/ip4/0.0.0.0/tcp/{}/ws", self.ws_port).parse().unwrap(),
        ]
    }
}
```

#### 1.3 Local Network Discovery

```rust
/// mDNS for local network discovery
pub struct MdnsDiscovery {
    mdns: Mdns,
    service_name: String,
}

impl MdnsDiscovery {
    pub async fn new() -> Result<Self> {
        let mdns = Mdns::new(MdnsConfig::default())?;

        Ok(Self {
            mdns,
            service_name: "_guts._tcp.local".to_string(),
        })
    }

    /// Discover peers on local network
    pub fn discovered_peers(&self) -> impl Stream<Item = PeerInfo> {
        self.mdns.discovered()
    }
}
```

### Phase 2: Gossip Protocol

#### 2.1 Plumtree Implementation

```rust
// crates/guts-p2p/src/gossip/plumtree.rs

/// Plumtree: Epidemic Broadcast Trees
/// Combines eager push (tree) with lazy push (gossip)
pub struct Plumtree {
    /// Eager peers (tree edges)
    eager_peers: HashSet<PeerId>,

    /// Lazy peers (non-tree edges)
    lazy_peers: HashSet<PeerId>,

    /// Missing messages (for pull requests)
    missing: HashMap<MessageId, HashSet<PeerId>>,

    /// Received messages (dedup)
    received: LruCache<MessageId, ()>,

    /// Configuration
    config: PlumtreeConfig,
}

pub struct PlumtreeConfig {
    /// Threshold for moving peer to lazy
    pub graft_threshold: Duration,

    /// Missing message timeout
    pub ihave_timeout: Duration,

    /// Optimization interval
    pub optimization_interval: Duration,
}

impl Plumtree {
    /// Broadcast a message
    pub async fn broadcast(&mut self, msg: GossipMessage) -> Result<()> {
        let msg_id = msg.id();

        // Already seen?
        if self.received.contains(&msg_id) {
            return Ok(());
        }
        self.received.put(msg_id.clone(), ());

        // Eager push to tree neighbors
        for peer in &self.eager_peers {
            self.send_eager(peer, &msg).await?;
        }

        // Lazy push (IHAVE) to others
        for peer in &self.lazy_peers {
            self.send_ihave(peer, &msg_id).await?;
        }

        Ok(())
    }

    /// Handle received message
    pub async fn on_message(&mut self, from: PeerId, msg: GossipMessage) -> Result<()> {
        let msg_id = msg.id();

        // Already seen?
        if self.received.contains(&msg_id) {
            // Prune: demote sender to lazy
            self.eager_peers.remove(&from);
            self.lazy_peers.insert(from);
            self.send_prune(&from, &msg_id).await?;
            return Ok(());
        }

        // New message: sender becomes eager
        self.received.put(msg_id.clone(), ());
        self.lazy_peers.remove(&from);
        self.eager_peers.insert(from);

        // Cancel any pending requests
        self.missing.remove(&msg_id);

        // Forward to other eager peers
        for peer in &self.eager_peers {
            if *peer != from {
                self.send_eager(peer, &msg).await?;
            }
        }

        // Lazy announce to lazy peers
        for peer in &self.lazy_peers {
            self.send_ihave(peer, &msg_id).await?;
        }

        Ok(())
    }

    /// Handle IHAVE announcement
    pub async fn on_ihave(&mut self, from: PeerId, msg_id: MessageId) -> Result<()> {
        // Already have it?
        if self.received.contains(&msg_id) {
            return Ok(());
        }

        // Track who has this message
        self.missing
            .entry(msg_id.clone())
            .or_default()
            .insert(from);

        // Start timer for pull request
        self.schedule_graft(msg_id).await;

        Ok(())
    }

    /// Handle GRAFT request
    pub async fn on_graft(&mut self, from: PeerId, msg_id: MessageId) -> Result<()> {
        // Promote peer to eager
        self.lazy_peers.remove(&from);
        self.eager_peers.insert(from);

        // Send the message if we have it
        if let Some(msg) = self.get_message(&msg_id) {
            self.send_eager(&from, &msg).await?;
        }

        Ok(())
    }
}
```

#### 2.2 Message Types

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// New commit/ref update
    RefUpdate {
        repo_key: RepoKey,
        ref_name: String,
        old_oid: Option<ObjectId>,
        new_oid: ObjectId,
        commit_chain: Vec<ObjectId>,
    },

    /// New collaboration item
    CollaborationEvent {
        repo_key: RepoKey,
        event_type: CollabEventType,
        item_id: Uuid,
        content_hash: [u8; 32],
    },

    /// Consensus proposal
    ConsensusMessage {
        epoch: u64,
        slot: u64,
        message_type: ConsensusMessageType,
        payload: Vec<u8>,
        signature: Signature,
    },

    /// Validator set change
    ValidatorChange {
        epoch: u64,
        change: ValidatorSetChange,
        proofs: Vec<ValidatorProof>,
    },
}
```

### Phase 3: Validator Governance

#### 3.1 Stake-Based Validator Set

```rust
// crates/guts-p2p/src/governance/staking.rs

pub struct StakingModule {
    /// Current validator set
    validators: ValidatorSet,

    /// Pending stake changes
    pending_changes: Vec<StakeChange>,

    /// Stake lock period
    lock_period: Duration,

    /// Minimum stake to become validator
    min_stake: u64,

    /// Maximum validators
    max_validators: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator public key
    pub pubkey: PublicKey,

    /// Staked amount
    pub stake: u64,

    /// Commission rate (basis points)
    pub commission: u16,

    /// Joined at epoch
    pub joined_epoch: u64,

    /// Performance metrics
    pub uptime: f64,
    pub blocks_proposed: u64,
    pub blocks_missed: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum StakeChange {
    /// Add stake (becomes validator if above minimum)
    Stake {
        validator: PublicKey,
        amount: u64,
        proof: StakeProof,
    },

    /// Remove stake (exits validator set if below minimum)
    Unstake {
        validator: PublicKey,
        amount: u64,
    },

    /// Slash validator for misbehavior
    Slash {
        validator: PublicKey,
        reason: SlashReason,
        amount: u64,
        evidence: SlashEvidence,
    },
}

impl StakingModule {
    /// Process stake deposit
    pub async fn stake(
        &mut self,
        validator: PublicKey,
        amount: u64,
        proof: StakeProof,
    ) -> Result<()> {
        // Verify stake proof
        self.verify_stake_proof(&proof)?;

        // Add to pending changes
        self.pending_changes.push(StakeChange::Stake {
            validator,
            amount,
            proof,
        });

        Ok(())
    }

    /// Process stake withdrawal
    pub async fn unstake(&mut self, validator: PublicKey, amount: u64) -> Result<()> {
        // Check if validator has enough stake
        let current_stake = self.validators.get_stake(&validator)?;
        if current_stake < amount {
            return Err(Error::InsufficientStake);
        }

        // Check lock period
        if !self.can_unstake(&validator)? {
            return Err(Error::StakeLocked);
        }

        self.pending_changes.push(StakeChange::Unstake { validator, amount });

        Ok(())
    }

    /// Apply pending changes at epoch boundary
    pub async fn end_epoch(&mut self, epoch: u64) -> Result<ValidatorSetChange> {
        let mut changes = ValidatorSetChange::default();

        for change in std::mem::take(&mut self.pending_changes) {
            match change {
                StakeChange::Stake { validator, amount, .. } => {
                    let new_stake = self.validators.add_stake(&validator, amount);

                    if new_stake >= self.min_stake {
                        if !self.validators.is_validator(&validator) {
                            changes.added.push(validator);
                        }
                    }
                }
                StakeChange::Unstake { validator, amount } => {
                    let new_stake = self.validators.remove_stake(&validator, amount);

                    if new_stake < self.min_stake {
                        if self.validators.is_validator(&validator) {
                            changes.removed.push(validator);
                        }
                    }
                }
                StakeChange::Slash { validator, amount, .. } => {
                    self.validators.slash(&validator, amount);
                    changes.slashed.push(validator);
                }
            }
        }

        // Update validator set
        self.validators.apply_changes(&changes);

        Ok(changes)
    }
}
```

#### 3.2 Validator Voting

```rust
// crates/guts-p2p/src/governance/voting.rs

pub struct GovernanceVoting {
    /// Active proposals
    proposals: HashMap<ProposalId, Proposal>,

    /// Votes by proposal
    votes: HashMap<ProposalId, HashMap<PublicKey, Vote>>,

    /// Voting parameters
    params: VotingParams,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: PublicKey,
    pub proposal_type: ProposalType,
    pub description: String,
    pub created_at: u64,
    pub voting_ends: u64,
    pub execution_delay: u64,
    pub status: ProposalStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change protocol parameters
    ParameterChange {
        parameter: String,
        old_value: Value,
        new_value: Value,
    },

    /// Upgrade protocol version
    ProtocolUpgrade {
        version: String,
        activation_epoch: u64,
    },

    /// Emergency action
    Emergency {
        action: EmergencyAction,
        justification: String,
    },

    /// Spend from treasury
    TreasurySpend {
        recipient: String,
        amount: u64,
        purpose: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

impl GovernanceVoting {
    /// Submit a new proposal
    pub async fn submit_proposal(
        &mut self,
        proposer: PublicKey,
        proposal_type: ProposalType,
        description: String,
    ) -> Result<ProposalId> {
        // Check proposer is a validator
        if !self.is_validator(&proposer) {
            return Err(Error::NotValidator);
        }

        // Check proposer has enough stake for proposal
        if self.get_stake(&proposer) < self.params.proposal_threshold {
            return Err(Error::InsufficientStakeForProposal);
        }

        let proposal = Proposal {
            id: ProposalId::new(),
            proposer,
            proposal_type,
            description,
            created_at: current_epoch(),
            voting_ends: current_epoch() + self.params.voting_period,
            execution_delay: self.params.execution_delay,
            status: ProposalStatus::Active,
        };

        let id = proposal.id.clone();
        self.proposals.insert(id.clone(), proposal);

        Ok(id)
    }

    /// Cast a vote
    pub async fn vote(
        &mut self,
        proposal_id: &ProposalId,
        voter: PublicKey,
        vote: Vote,
    ) -> Result<()> {
        // Check proposal exists and is active
        let proposal = self.proposals.get(proposal_id)
            .ok_or(Error::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalNotActive);
        }

        if current_epoch() > proposal.voting_ends {
            return Err(Error::VotingEnded);
        }

        // Check voter is a validator
        if !self.is_validator(&voter) {
            return Err(Error::NotValidator);
        }

        // Record vote
        self.votes
            .entry(proposal_id.clone())
            .or_default()
            .insert(voter, vote);

        Ok(())
    }

    /// Tally votes and update proposal status
    pub async fn tally(&mut self, proposal_id: &ProposalId) -> Result<ProposalStatus> {
        let proposal = self.proposals.get(proposal_id)
            .ok_or(Error::ProposalNotFound)?;

        let votes = self.votes.get(proposal_id)
            .map(|v| v.clone())
            .unwrap_or_default();

        // Calculate stake-weighted votes
        let mut yes_stake = 0u64;
        let mut no_stake = 0u64;
        let mut abstain_stake = 0u64;

        for (voter, vote) in votes {
            let stake = self.get_stake(&voter);
            match vote {
                Vote::Yes => yes_stake += stake,
                Vote::No => no_stake += stake,
                Vote::Abstain => abstain_stake += stake,
            }
        }

        let total_stake = self.total_stake();
        let participation = (yes_stake + no_stake + abstain_stake) as f64 / total_stake as f64;

        // Check quorum
        if participation < self.params.quorum_threshold {
            return Ok(ProposalStatus::Rejected { reason: "Quorum not reached".to_string() });
        }

        // Check approval
        let approval = yes_stake as f64 / (yes_stake + no_stake) as f64;
        if approval >= self.params.approval_threshold {
            Ok(ProposalStatus::Passed {
                execution_epoch: current_epoch() + proposal.execution_delay,
            })
        } else {
            Ok(ProposalStatus::Rejected { reason: "Not enough support".to_string() })
        }
    }
}
```

### Phase 4: Sybil Resistance

#### 4.1 Stake-Based Protection

```rust
// crates/guts-p2p/src/sybil/stake.rs

pub struct StakeBasedSybilResistance {
    /// Minimum stake for various actions
    min_stake: StakeRequirements,

    /// Reputation scores
    reputation: HashMap<PublicKey, ReputationScore>,
}

#[derive(Clone)]
pub struct StakeRequirements {
    /// Minimum to join as validator
    pub validator: u64,

    /// Minimum to create repository
    pub create_repo: u64,

    /// Minimum to push to any repo
    pub push: u64,

    /// Minimum for governance participation
    pub governance: u64,
}

impl StakeBasedSybilResistance {
    /// Check if identity can perform action
    pub fn can_perform(&self, identity: &PublicKey, action: Action) -> Result<()> {
        let stake = self.get_stake(identity);
        let reputation = self.get_reputation(identity);

        let required = match action {
            Action::CreateRepository => self.min_stake.create_repo,
            Action::Push { repo, .. } => {
                if self.is_repo_owner(repo, identity) {
                    0  // Owners can always push
                } else {
                    self.min_stake.push
                }
            }
            Action::BecomeValidator => self.min_stake.validator,
            Action::Vote => self.min_stake.governance,
        };

        // Apply reputation discount
        let effective_required = (required as f64 * (1.0 - reputation.discount())) as u64;

        if stake >= effective_required {
            Ok(())
        } else {
            Err(Error::InsufficientStake {
                required: effective_required,
                have: stake,
            })
        }
    }
}
```

#### 4.2 Proof of Work (Optional Alternative)

```rust
// crates/guts-p2p/src/sybil/pow.rs

pub struct PowSybilResistance {
    /// Difficulty for various actions
    difficulty: PowDifficulty,

    /// Recent PoW cache
    recent_pow: LruCache<[u8; 32], ()>,
}

#[derive(Clone)]
pub struct PowDifficulty {
    /// Bits of work for repository creation
    pub create_repo: u8,

    /// Bits of work per push
    pub push: u8,

    /// Bits of work for issue/PR creation
    pub collaboration: u8,
}

impl PowSybilResistance {
    /// Verify proof of work
    pub fn verify_pow(&self, challenge: &[u8], nonce: u64, difficulty: u8) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(challenge);
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        // Check leading zero bits
        leading_zeros(&hash) >= difficulty
    }

    /// Generate challenge for action
    pub fn generate_challenge(&self, action: &Action) -> ([u8; 32], u8) {
        let challenge = rand::random();
        let difficulty = self.difficulty_for(action);
        (challenge, difficulty)
    }
}
```

### Phase 5: Network Partition Handling

#### 5.1 Partition Detection

```rust
pub struct PartitionDetector {
    /// Known peer connectivity
    peer_connectivity: HashMap<PeerId, PeerStatus>,

    /// Partition detection threshold
    threshold: f64,

    /// Check interval
    check_interval: Duration,
}

impl PartitionDetector {
    /// Check for network partition
    pub async fn check_partition(&self) -> PartitionStatus {
        let total_peers = self.peer_connectivity.len();
        let connected_peers = self.peer_connectivity.values()
            .filter(|s| s.is_connected())
            .count();

        let connectivity = connected_peers as f64 / total_peers as f64;

        if connectivity < self.threshold {
            PartitionStatus::Partitioned {
                connected: connected_peers,
                total: total_peers,
            }
        } else {
            PartitionStatus::Connected
        }
    }

    /// Handle detected partition
    pub async fn on_partition(&mut self) -> Result<()> {
        // Pause consensus participation
        self.pause_consensus().await?;

        // Continue serving read-only requests
        self.enable_read_only_mode().await?;

        // Start partition healing
        self.start_healing().await?;

        Ok(())
    }
}
```

#### 5.2 Partition Recovery

```rust
pub struct PartitionRecovery {
    /// State snapshot before partition
    pre_partition_state: Option<StateSnapshot>,

    /// Operations during partition
    partition_operations: Vec<Operation>,

    /// Recovery strategy
    strategy: RecoveryStrategy,
}

#[derive(Clone)]
pub enum RecoveryStrategy {
    /// Longest chain wins
    LongestChain,

    /// Most stake-weighted support wins
    MostStake,

    /// Manual resolution required
    Manual,
}

impl PartitionRecovery {
    /// Recover from partition
    pub async fn recover(&mut self, other_partition: &PartitionState) -> Result<()> {
        match self.strategy {
            RecoveryStrategy::LongestChain => {
                self.recover_longest_chain(other_partition).await
            }
            RecoveryStrategy::MostStake => {
                self.recover_most_stake(other_partition).await
            }
            RecoveryStrategy::Manual => {
                self.require_manual_resolution(other_partition).await
            }
        }
    }

    async fn recover_longest_chain(&mut self, other: &PartitionState) -> Result<()> {
        // Compare chain lengths
        let our_height = self.partition_operations.len();
        let their_height = other.operations.len();

        if their_height > our_height {
            // Rollback and apply their state
            self.rollback_to(self.pre_partition_state.clone().unwrap()).await?;
            self.apply_operations(&other.operations).await?;
        }

        // Resume normal operation
        self.resume_consensus().await?;

        Ok(())
    }
}
```

### Phase 6: Public Testnet

#### 6.1 Testnet Configuration

```yaml
# infra/testnet/config.yml
network:
  name: "guts-testnet-1"
  chain_id: "guts-testnet-1"

genesis:
  validators:
    - pubkey: "ed25519:..."
      stake: 1000000
      name: "validator-1"
    - pubkey: "ed25519:..."
      stake: 1000000
      name: "validator-2"
    # ... 20+ validators

  parameters:
    min_stake: 100000
    max_validators: 100
    epoch_duration: 3600  # 1 hour
    voting_period: 86400  # 24 hours

bootstrap_nodes:
  - "/dns4/bootstrap1.testnet.guts.network/tcp/9000/p2p/..."
  - "/dns4/bootstrap2.testnet.guts.network/tcp/9000/p2p/..."
  - "/dns4/bootstrap3.testnet.guts.network/tcp/9000/p2p/..."

regions:
  - name: "us-east"
    nodes: 5
  - name: "eu-west"
    nodes: 5
  - name: "ap-southeast"
    nodes: 5
  - name: "sa-east"
    nodes: 5
```

#### 6.2 Testnet Faucet

```rust
pub struct TestnetFaucet {
    /// Available balance
    balance: AtomicU64,

    /// Per-identity limit
    limit_per_identity: u64,

    /// Cooldown period
    cooldown: Duration,

    /// Recent requests
    recent_requests: Mutex<HashMap<PublicKey, Instant>>,
}

impl TestnetFaucet {
    /// Request testnet tokens
    pub async fn request(&self, identity: PublicKey, amount: u64) -> Result<()> {
        // Check cooldown
        if let Some(last) = self.recent_requests.lock().await.get(&identity) {
            if last.elapsed() < self.cooldown {
                return Err(Error::CooldownActive);
            }
        }

        // Check limit
        if amount > self.limit_per_identity {
            return Err(Error::ExceedsLimit);
        }

        // Check balance
        let current = self.balance.load(Ordering::Relaxed);
        if current < amount {
            return Err(Error::FaucetEmpty);
        }

        // Dispense tokens
        self.balance.fetch_sub(amount, Ordering::Relaxed);
        self.recent_requests.lock().await.insert(identity, Instant::now());

        // Transfer tokens
        self.transfer_tokens(identity, amount).await?;

        Ok(())
    }
}
```

## Implementation Plan

### Phase 1: DHT Discovery (Week 1-3)
- [ ] Integrate libp2p Kademlia
- [ ] Implement multi-address support
- [ ] Add mDNS for local discovery
- [ ] Create bootstrap node infrastructure
- [ ] Test peer discovery in isolation

### Phase 2: Gossip Protocol (Week 3-5)
- [ ] Implement Plumtree gossip
- [ ] Define gossip message types
- [ ] Integrate with consensus layer
- [ ] Benchmark message propagation
- [ ] Tune gossip parameters

### Phase 3: Validator Governance (Week 5-7)
- [ ] Implement staking module
- [ ] Add validator set management
- [ ] Implement governance voting
- [ ] Add proposal types
- [ ] Test epoch transitions

### Phase 4: Sybil Resistance (Week 7-8)
- [ ] Implement stake-based protection
- [ ] Add optional PoW fallback
- [ ] Integrate with action authorization
- [ ] Test attack scenarios

### Phase 5: Partition Handling (Week 8-9)
- [ ] Implement partition detection
- [ ] Add recovery strategies
- [ ] Test partition scenarios
- [ ] Document recovery procedures

### Phase 6: Testnet Launch (Week 9-12)
- [ ] Set up testnet infrastructure
- [ ] Deploy 20+ validator nodes
- [ ] Launch testnet faucet
- [ ] Create onboarding documentation
- [ ] Monitor and iterate

## Success Criteria

- [ ] Nodes discover peers without bootstrap after initial connection
- [ ] Message propagation reaches all nodes within 2 seconds
- [ ] Validator set changes propagate correctly
- [ ] Network survives 30% node failures
- [ ] Partition recovery completes within 1 hour
- [ ] 20+ independent operators running validators
- [ ] Testnet stable for 30+ days
- [ ] Geographic distribution across 4+ regions
- [ ] Documentation complete for operators

## Security Considerations

1. **Eclipse Attacks**: Implement peer diversity requirements
2. **Sybil Attacks**: Require stake or PoW for all actions
3. **Long-Range Attacks**: Implement checkpointing
4. **Partition Attacks**: Require supermajority for consensus
5. **Governance Attacks**: Time-lock and veto mechanisms

## Dependencies

- libp2p for networking primitives
- External stake/token bridge (if using external token)
- Multi-region infrastructure (AWS, GCP, Azure)
- Independent validator operators

## References

- [Kademlia Paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)
- [Plumtree Paper](https://asc.di.fct.unl.pt/~jleitao/pdf/srds07-leitao.pdf)
- [libp2p Specifications](https://github.com/libp2p/specs)
- [Cosmos SDK Staking](https://docs.cosmos.network/main/build/modules/staking)

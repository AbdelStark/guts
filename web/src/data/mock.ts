// Mock data for GUTS Web MVP

export interface Repository {
  id: string;
  owner: string;
  name: string;
  description: string;
  visibility: "public" | "private";
  consensusStatus: "verified" | "pending" | "conflicted";
  stats: {
    commits: number;
    branches: number;
    issues: number;
    pullRequests: number;
  };
  lastUpdated: string;
  language?: string;
}

export interface PullRequest {
  id: string;
  number: number;
  title: string;
  description: string;
  author: string;
  authorAvatar?: string;
  state: "open" | "closed" | "merged";
  sourceBranch: string;
  targetBranch: string;
  createdAt: string;
  updatedAt: string;
  reviewers: string[];
  labels: string[];
  consensusStatus: "verified" | "pending" | "conflicted";
  commits: number;
  additions: number;
  deletions: number;
}

export interface Issue {
  id: string;
  number: number;
  title: string;
  description: string;
  author: string;
  authorAvatar?: string;
  state: "open" | "closed";
  createdAt: string;
  updatedAt: string;
  labels: string[];
  assignees: string[];
  comments: number;
}

export interface Node {
  id: string;
  publicKey: string;
  address: string;
  status: "online" | "syncing" | "offline";
  version: string;
  lastSeen: string;
  peers: number;
  consensusRole: "validator" | "observer";
  location?: string;
}

export interface Activity {
  id: string;
  type: "commit" | "pr" | "issue" | "merge" | "review";
  title: string;
  description: string;
  author: string;
  repo: string;
  timestamp: string;
}

export const mockRepositories: Repository[] = [
  {
    id: "1",
    owner: "guts-org",
    name: "guts",
    description: "Decentralized code collaboration platform - a sovereign alternative to GitHub",
    visibility: "public",
    consensusStatus: "verified",
    stats: { commits: 1247, branches: 12, issues: 34, pullRequests: 8 },
    lastUpdated: "2 hours ago",
    language: "Rust",
  },
  {
    id: "2",
    owner: "guts-org",
    name: "sdk",
    description: "Client SDK for interacting with the GUTS network",
    visibility: "public",
    consensusStatus: "verified",
    stats: { commits: 456, branches: 5, issues: 12, pullRequests: 3 },
    lastUpdated: "5 hours ago",
    language: "TypeScript",
  },
  {
    id: "3",
    owner: "guts-org",
    name: "docs",
    description: "Documentation and guides for GUTS",
    visibility: "public",
    consensusStatus: "verified",
    stats: { commits: 234, branches: 3, issues: 8, pullRequests: 2 },
    lastUpdated: "1 day ago",
    language: "MDX",
  },
  {
    id: "4",
    owner: "satoshi",
    name: "bitcoin-whitepaper",
    description: "A Peer-to-Peer Electronic Cash System",
    visibility: "public",
    consensusStatus: "verified",
    stats: { commits: 1, branches: 1, issues: 0, pullRequests: 0 },
    lastUpdated: "16 years ago",
    language: "LaTeX",
  },
  {
    id: "5",
    owner: "alice",
    name: "cryptographic-protocols",
    description: "Research on cryptographic protocols for decentralized systems",
    visibility: "public",
    consensusStatus: "pending",
    stats: { commits: 89, branches: 4, issues: 5, pullRequests: 1 },
    lastUpdated: "3 days ago",
    language: "Rust",
  },
  {
    id: "6",
    owner: "bob",
    name: "p2p-experiments",
    description: "Experimental P2P networking implementations",
    visibility: "private",
    consensusStatus: "conflicted",
    stats: { commits: 156, branches: 7, issues: 23, pullRequests: 4 },
    lastUpdated: "12 hours ago",
    language: "Go",
  },
];

export const mockPullRequests: PullRequest[] = [
  {
    id: "pr-1",
    number: 42,
    title: "feat(consensus): implement Simplex BFT consensus engine",
    description: "This PR adds the core Simplex BFT consensus engine with support for 4-validator devnet.",
    author: "satoshi",
    state: "open",
    sourceBranch: "feat/simplex-bft",
    targetBranch: "main",
    createdAt: "2 days ago",
    updatedAt: "1 hour ago",
    reviewers: ["alice", "bob"],
    labels: ["enhancement", "consensus"],
    consensusStatus: "pending",
    commits: 15,
    additions: 2847,
    deletions: 156,
  },
  {
    id: "pr-2",
    number: 41,
    title: "fix(p2p): resolve connection timeout issues",
    description: "Fixes sporadic connection timeouts in high-latency networks.",
    author: "alice",
    state: "merged",
    sourceBranch: "fix/p2p-timeout",
    targetBranch: "main",
    createdAt: "5 days ago",
    updatedAt: "3 days ago",
    reviewers: ["satoshi"],
    labels: ["bug", "p2p"],
    consensusStatus: "verified",
    commits: 3,
    additions: 45,
    deletions: 12,
  },
  {
    id: "pr-3",
    number: 40,
    title: "docs: update installation guide for new CLI",
    description: "Updates the installation guide to reflect the new CLI structure.",
    author: "bob",
    state: "open",
    sourceBranch: "docs/cli-update",
    targetBranch: "main",
    createdAt: "1 day ago",
    updatedAt: "1 day ago",
    reviewers: [],
    labels: ["documentation"],
    consensusStatus: "pending",
    commits: 2,
    additions: 89,
    deletions: 34,
  },
  {
    id: "pr-4",
    number: 39,
    title: "refactor(storage): migrate to RocksDB backend",
    description: "Replaces the in-memory storage with RocksDB for persistence.",
    author: "satoshi",
    state: "closed",
    sourceBranch: "refactor/rocksdb",
    targetBranch: "main",
    createdAt: "1 week ago",
    updatedAt: "6 days ago",
    reviewers: ["alice", "bob"],
    labels: ["enhancement", "storage"],
    consensusStatus: "verified",
    commits: 8,
    additions: 567,
    deletions: 234,
  },
];

export const mockIssues: Issue[] = [
  {
    id: "issue-1",
    number: 156,
    title: "Node crashes when processing large pack files",
    description: "When cloning a repository with a pack file larger than 100MB, the node crashes with an out-of-memory error.",
    author: "alice",
    state: "open",
    createdAt: "3 hours ago",
    updatedAt: "1 hour ago",
    labels: ["bug", "critical"],
    assignees: ["satoshi"],
    comments: 5,
  },
  {
    id: "issue-2",
    number: 155,
    title: "Add support for signed commits verification",
    description: "We should verify commit signatures during the consensus process to ensure authenticity.",
    author: "bob",
    state: "open",
    createdAt: "1 day ago",
    updatedAt: "12 hours ago",
    labels: ["enhancement", "security"],
    assignees: [],
    comments: 8,
  },
  {
    id: "issue-3",
    number: 154,
    title: "Improve error messages for CLI",
    description: "The current error messages are too cryptic. We should provide more helpful messages.",
    author: "satoshi",
    state: "open",
    createdAt: "2 days ago",
    updatedAt: "2 days ago",
    labels: ["enhancement", "ux"],
    assignees: ["bob"],
    comments: 3,
  },
  {
    id: "issue-4",
    number: 153,
    title: "Documentation needs update for API v2",
    description: "The API documentation is outdated and doesn't reflect the latest changes.",
    author: "alice",
    state: "closed",
    createdAt: "1 week ago",
    updatedAt: "3 days ago",
    labels: ["documentation"],
    assignees: ["alice"],
    comments: 2,
  },
];

export const mockNodes: Node[] = [
  {
    id: "node-1",
    publicKey: "7f3a9b2c4e5d6f8a1b3c5e7d9f1a3b5c7e9d1f3a5b7c9e1d3f5a7b9c1e3d5f7a9b",
    address: "135.181.42.156:9000",
    status: "online",
    version: "0.1.0",
    lastSeen: "Just now",
    peers: 24,
    consensusRole: "validator",
    location: "Helsinki, FI",
  },
  {
    id: "node-2",
    publicKey: "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
    address: "95.216.89.23:9000",
    status: "online",
    version: "0.1.0",
    lastSeen: "2 seconds ago",
    peers: 22,
    consensusRole: "validator",
    location: "Frankfurt, DE",
  },
  {
    id: "node-3",
    publicKey: "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
    address: "138.201.156.78:9000",
    status: "syncing",
    version: "0.1.0",
    lastSeen: "15 seconds ago",
    peers: 18,
    consensusRole: "observer",
    location: "Ashburn, US",
  },
  {
    id: "node-4",
    publicKey: "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6",
    address: "5.9.61.42:9000",
    status: "online",
    version: "0.1.0",
    lastSeen: "5 seconds ago",
    peers: 21,
    consensusRole: "validator",
    location: "Singapore, SG",
  },
  {
    id: "node-5",
    publicKey: "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7",
    address: "78.47.82.19:9000",
    status: "offline",
    version: "0.0.9",
    lastSeen: "2 hours ago",
    peers: 0,
    consensusRole: "observer",
    location: "Tokyo, JP",
  },
];

export const mockActivity: Activity[] = [
  {
    id: "act-1",
    type: "commit",
    title: "fix: resolve memory leak in pack file parser",
    description: "Pushed to main",
    author: "satoshi",
    repo: "guts-org/guts",
    timestamp: "15 minutes ago",
  },
  {
    id: "act-2",
    type: "pr",
    title: "feat(consensus): implement Simplex BFT",
    description: "Opened pull request #42",
    author: "satoshi",
    repo: "guts-org/guts",
    timestamp: "2 hours ago",
  },
  {
    id: "act-3",
    type: "review",
    title: "Approved PR #41",
    description: "fix(p2p): resolve connection timeout issues",
    author: "alice",
    repo: "guts-org/guts",
    timestamp: "3 hours ago",
  },
  {
    id: "act-4",
    type: "merge",
    title: "Merged PR #41",
    description: "fix(p2p): resolve connection timeout issues",
    author: "satoshi",
    repo: "guts-org/guts",
    timestamp: "3 hours ago",
  },
  {
    id: "act-5",
    type: "issue",
    title: "Node crashes when processing large pack files",
    description: "Opened issue #156",
    author: "alice",
    repo: "guts-org/guts",
    timestamp: "3 hours ago",
  },
];

export const mockFileTree = [
  {
    name: "src",
    type: "directory" as const,
    children: [
      {
        name: "main.rs",
        type: "file" as const,
        language: "rust",
      },
      {
        name: "lib.rs",
        type: "file" as const,
        language: "rust",
      },
      {
        name: "consensus",
        type: "directory" as const,
        children: [
          { name: "mod.rs", type: "file" as const, language: "rust" },
          { name: "simplex.rs", type: "file" as const, language: "rust" },
          { name: "types.rs", type: "file" as const, language: "rust" },
        ],
      },
      {
        name: "p2p",
        type: "directory" as const,
        children: [
          { name: "mod.rs", type: "file" as const, language: "rust" },
          { name: "network.rs", type: "file" as const, language: "rust" },
        ],
      },
    ],
  },
  {
    name: "Cargo.toml",
    type: "file" as const,
    language: "toml",
  },
  {
    name: "README.md",
    type: "file" as const,
    language: "markdown",
  },
];

export const mockCode = `use std::collections::HashMap;
use tokio::sync::mpsc;
use thiserror::Error;

/// Core consensus engine implementing Simplex BFT
pub struct ConsensusEngine {
    /// Current view number
    view: u64,
    /// Validator set
    validators: Vec<PublicKey>,
    /// Pending transactions
    mempool: Vec<Transaction>,
    /// Finalized blocks
    chain: Vec<Block>,
}

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Quorum not reached")]
    QuorumNotReached,
    #[error("Block not found: {0}")]
    BlockNotFound(u64),
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new(validators: Vec<PublicKey>) -> Self {
        Self {
            view: 0,
            validators,
            mempool: Vec::new(),
            chain: Vec::new(),
        }
    }

    /// Process a new transaction
    pub async fn submit_transaction(&mut self, tx: Transaction) -> Result<(), ConsensusError> {
        // Verify transaction signature
        if !tx.verify() {
            return Err(ConsensusError::InvalidSignature);
        }

        self.mempool.push(tx);
        Ok(())
    }

    /// Propose a new block
    pub async fn propose_block(&mut self) -> Result<Block, ConsensusError> {
        let transactions = self.mempool.drain(..).collect();
        let block = Block::new(self.view, transactions);
        Ok(block)
    }

    /// Finalize a block after reaching consensus
    pub async fn finalize_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        self.chain.push(block);
        self.view += 1;
        Ok(())
    }
}`;

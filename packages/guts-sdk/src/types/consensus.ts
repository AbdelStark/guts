/**
 * Consensus and network types.
 */

export interface ConsensusStatus {
  /** Current block height */
  height: number;
  /** Current round */
  round: number;
  /** Current phase */
  phase: string;
  /** Whether the node is synced */
  synced: boolean;
  /** Number of connected peers */
  peers_count: number;
  /** Number of validators */
  validators_count: number;
  /** Number of pending transactions */
  mempool_size: number;
  /** Last block time */
  last_block_time?: string;
}

export interface Block {
  /** Block height */
  height: number;
  /** Block hash */
  hash: string;
  /** Parent block hash */
  parent_hash: string;
  /** Block timestamp */
  timestamp: string;
  /** Proposer public key */
  proposer: string;
  /** Number of transactions */
  transactions_count: number;
  /** Block size in bytes */
  size: number;
}

export interface Validator {
  /** Validator public key */
  public_key: string;
  /** Validator address */
  address: string;
  /** Whether the validator is active */
  active: boolean;
  /** Voting power */
  voting_power: number;
  /** Last seen timestamp */
  last_seen?: string;
}

export interface Transaction {
  /** Transaction hash */
  hash: string;
  /** Transaction type */
  type: string;
  /** Transaction payload */
  payload: unknown;
  /** Sender public key */
  sender: string;
  /** Timestamp */
  timestamp: string;
  /** Block height (if included) */
  block_height?: number;
}

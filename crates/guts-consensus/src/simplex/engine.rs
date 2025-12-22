//! Simplex BFT consensus engine.
//!
//! This module provides the main engine that orchestrates the Simplex BFT
//! consensus protocol for the Guts decentralized code collaboration platform.
//!
//! # Architecture
//!
//! The engine integrates with commonware-consensus's Simplex implementation
//! to provide Byzantine fault-tolerant consensus. The key components are:
//!
//! - **Application Actor**: Handles block proposal and verification
//! - **Marshal Actor**: Manages block storage and synchronization
//! - **Buffer Engine**: Buffers broadcast messages for efficient delivery
//! - **Consensus Engine**: Core BFT voting logic (Batcher, Voter, Resolver)
//!
//! # Usage
//!
//! The engine requires P2P channels for:
//! - `pending`: Pending consensus votes
//! - `recovered`: Recovered messages after reconnection
//! - `resolver`: Fetching missing certificates
//! - `broadcast`: Block broadcast messages
//! - `marshal`: Block sync messages

use super::{
    application::{self, Actor as ApplicationActor, Mailbox as ApplicationMailbox},
    block::SimplexBlock,
    types::Scheme,
};
use commonware_broadcast::buffered;
use commonware_consensus::{
    marshal::{self, ingress::handler},
    simplex::{self, Engine as Consensus},
};
use commonware_cryptography::{ed25519::PublicKey, sha256::Digest};
use commonware_p2p::{Blocker, Receiver, Sender};
use commonware_resolver::Resolver;
use commonware_runtime::{buffer::PoolRef, Clock, Handle, Metrics, Spawner, Storage};
use commonware_utils::{set::Ordered, NZUsize, NZU64};
use futures::{channel::mpsc, future::try_join_all};
use governor::clock::Clock as GClock;
use governor::Quota;
use rand::{CryptoRng, Rng};
use std::{marker::PhantomData, num::NonZero, sync::Arc, time::Duration};
use tracing::{error, info, warn};

/// Type alias for the finalization callback.
pub type FinalizedCallback = Arc<dyn Fn(&SimplexBlock) + Send + Sync>;

/// Namespace for Guts consensus messages.
pub const NAMESPACE: &[u8] = b"guts-consensus";

/// Epoch for the consensus instance.
pub const EPOCH: u64 = 0;

/// Epoch length (effectively infinite for single-epoch operation).
pub const EPOCH_LENGTH: u64 = u64::MAX;

// Buffer and storage constants
const PRUNABLE_ITEMS_PER_SECTION: NonZero<u64> = NZU64!(4_096);
const IMMUTABLE_ITEMS_PER_SECTION: NonZero<u64> = NZU64!(262_144);
const FREEZER_TABLE_RESIZE_FREQUENCY: u8 = 4;
const FREEZER_TABLE_RESIZE_CHUNK_SIZE: u32 = 2u32.pow(16);
const FREEZER_JOURNAL_TARGET_SIZE: u64 = 1024 * 1024 * 1024; // 1GB
const FREEZER_JOURNAL_COMPRESSION: Option<u8> = Some(3);
const REPLAY_BUFFER: NonZero<usize> = NZUsize!(8 * 1024 * 1024); // 8MB
const WRITE_BUFFER: NonZero<usize> = NZUsize!(1024 * 1024); // 1MB
const BUFFER_POOL_PAGE_SIZE: NonZero<usize> = NZUsize!(4_096); // 4KB
const BUFFER_POOL_CAPACITY: NonZero<usize> = NZUsize!(8_192); // 32MB
const MAX_REPAIR: u64 = 20;
const SYNCER_ACTIVITY_TIMEOUT_MULTIPLIER: u64 = 10;

/// Reporter type for the consensus engine.
///
/// Uses the marshal mailbox as the activity reporter, with no additional indexer.
type SimpleReporter = marshal::Mailbox<Scheme, SimplexBlock>;

/// Static scheme provider that always returns the same signing scheme.
#[derive(Clone)]
pub struct StaticSchemeProvider(Arc<Scheme>);

impl marshal::SchemeProvider for StaticSchemeProvider {
    type Scheme = Scheme;

    fn scheme(&self, _epoch: u64) -> Option<Arc<Scheme>> {
        Some(self.0.clone())
    }
}

impl From<Scheme> for StaticSchemeProvider {
    fn from(scheme: Scheme) -> Self {
        Self(Arc::new(scheme))
    }
}

/// Configuration for the Simplex engine.
#[derive(Clone)]
pub struct Config<B: Blocker<PublicKey = PublicKey>> {
    /// The blocker for managing peer connections.
    pub blocker: B,

    /// Prefix for storage partitions.
    pub partition_prefix: String,

    /// Initial size for the blocks freezer table.
    pub blocks_freezer_table_initial_size: u32,

    /// Initial size for the finalized freezer table.
    pub finalized_freezer_table_initial_size: u32,

    /// Our public key.
    pub me: PublicKey,

    /// Our private key for signing.
    pub private_key: commonware_cryptography::ed25519::PrivateKey,

    /// The set of participants in consensus.
    pub participants: Ordered<PublicKey>,

    /// Size of mailbox channels.
    pub mailbox_size: usize,

    /// Size of message deques.
    pub deque_size: usize,

    /// Timeout for leader proposal.
    pub leader_timeout: Duration,

    /// Timeout for notarization.
    pub notarization_timeout: Duration,

    /// Retry interval for nullify messages.
    pub nullify_retry: Duration,

    /// Timeout for fetch requests.
    pub fetch_timeout: Duration,

    /// Activity timeout in views.
    pub activity_timeout: u64,

    /// Skip timeout in views.
    pub skip_timeout: u64,

    /// Maximum number of blocks to fetch at once.
    pub max_fetch_count: usize,

    /// Number of concurrent fetch requests.
    pub fetch_concurrent: usize,

    /// Rate limit for fetch requests per peer.
    pub fetch_rate_per_peer: Quota,

    /// Callback for finalized blocks.
    pub on_finalized: Option<FinalizedCallback>,
}

impl<B: Blocker<PublicKey = PublicKey>> Config<B> {
    /// Creates a new configuration with sensible defaults.
    pub fn new(
        blocker: B,
        me: PublicKey,
        private_key: commonware_cryptography::ed25519::PrivateKey,
        participants: Vec<PublicKey>,
    ) -> Self {
        Self {
            blocker,
            partition_prefix: "guts".to_string(),
            blocks_freezer_table_initial_size: 2u32.pow(21), // ~100MB
            finalized_freezer_table_initial_size: 2u32.pow(21),
            me,
            private_key,
            participants: participants.into_iter().collect(),
            mailbox_size: 1024,
            deque_size: 10,
            leader_timeout: Duration::from_secs(1),
            notarization_timeout: Duration::from_secs(2),
            nullify_retry: Duration::from_secs(10),
            fetch_timeout: Duration::from_secs(2),
            activity_timeout: 256,
            skip_timeout: 32,
            max_fetch_count: 16,
            fetch_concurrent: 4,
            fetch_rate_per_peer: Quota::per_second(std::num::NonZeroU32::new(128).unwrap()),
            on_finalized: None,
        }
    }

    /// Sets the callback for finalized blocks.
    pub fn on_finalized<F>(mut self, callback: F) -> Self
    where
        F: Fn(&SimplexBlock) + Send + Sync + 'static,
    {
        self.on_finalized = Some(Arc::new(callback));
        self
    }
}

/// The Simplex BFT consensus engine.
pub struct Engine<E, B>
where
    E: Clock + GClock + Rng + CryptoRng + Spawner + Storage + Metrics + Clone,
    B: Blocker<PublicKey = PublicKey>,
{
    context: E,

    application: ApplicationActor<E>,
    application_mailbox: ApplicationMailbox,
    buffer: buffered::Engine<E, PublicKey, SimplexBlock>,
    buffer_mailbox: buffered::Mailbox<PublicKey, SimplexBlock>,
    marshal: marshal::Actor<E, SimplexBlock, StaticSchemeProvider, Scheme>,
    marshal_mailbox: marshal::Mailbox<Scheme, SimplexBlock>,

    consensus: Consensus<
        E,
        PublicKey,
        Scheme,
        B,
        Digest,
        ApplicationMailbox,
        ApplicationMailbox,
        SimpleReporter,
    >,
}

impl<E, B> Engine<E, B>
where
    E: Clock + GClock + Rng + CryptoRng + Spawner + Storage + Metrics + Clone,
    B: Blocker<PublicKey = PublicKey>,
{
    /// Creates a new Simplex engine.
    pub async fn new(context: E, cfg: Config<B>) -> Self {
        // Create the application actor
        let (application, application_mailbox) = ApplicationActor::new(
            context.clone(),
            application::Config {
                mailbox_size: cfg.mailbox_size,
            },
        );

        // Create the buffer
        let (buffer, buffer_mailbox) = buffered::Engine::new(
            context.clone(),
            buffered::Config {
                public_key: cfg.me.clone(),
                mailbox_size: cfg.mailbox_size,
                deque_size: cfg.deque_size,
                priority: true,
                codec_config: (),
            },
        );

        // Create the buffer pool
        let buffer_pool = PoolRef::new(BUFFER_POOL_PAGE_SIZE, BUFFER_POOL_CAPACITY);

        // Create the signing scheme
        let scheme = Scheme::new(cfg.participants.clone(), cfg.private_key);

        // Create marshal
        let (marshal, marshal_mailbox) = marshal::Actor::init(
            context.clone(),
            marshal::Config {
                scheme_provider: scheme.clone().into(),
                epoch_length: EPOCH_LENGTH,
                partition_prefix: cfg.partition_prefix.clone(),
                mailbox_size: cfg.mailbox_size,
                view_retention_timeout: cfg
                    .activity_timeout
                    .saturating_mul(SYNCER_ACTIVITY_TIMEOUT_MULTIPLIER),
                namespace: NAMESPACE.to_vec(),
                prunable_items_per_section: PRUNABLE_ITEMS_PER_SECTION,
                immutable_items_per_section: IMMUTABLE_ITEMS_PER_SECTION,
                freezer_table_initial_size: cfg.blocks_freezer_table_initial_size,
                freezer_table_resize_frequency: FREEZER_TABLE_RESIZE_FREQUENCY,
                freezer_table_resize_chunk_size: FREEZER_TABLE_RESIZE_CHUNK_SIZE,
                freezer_journal_target_size: FREEZER_JOURNAL_TARGET_SIZE,
                freezer_journal_compression: FREEZER_JOURNAL_COMPRESSION,
                freezer_journal_buffer_pool: buffer_pool.clone(),
                replay_buffer: REPLAY_BUFFER,
                write_buffer: WRITE_BUFFER,
                block_codec_config: (),
                max_repair: MAX_REPAIR,
                _marker: PhantomData,
            },
        )
        .await;

        // Use the marshal mailbox directly as the reporter
        let reporter = marshal_mailbox.clone();

        // Create the consensus engine
        let consensus = Consensus::new(
            context.clone(),
            simplex::Config {
                epoch: EPOCH,
                namespace: NAMESPACE.to_vec(),
                scheme,
                automaton: application_mailbox.clone(),
                relay: application_mailbox.clone(),
                reporter,
                partition: format!("{}-consensus", cfg.partition_prefix),
                mailbox_size: cfg.mailbox_size,
                leader_timeout: cfg.leader_timeout,
                notarization_timeout: cfg.notarization_timeout,
                nullify_retry: cfg.nullify_retry,
                fetch_timeout: cfg.fetch_timeout,
                activity_timeout: cfg.activity_timeout,
                skip_timeout: cfg.skip_timeout,
                max_fetch_count: cfg.max_fetch_count,
                fetch_concurrent: cfg.fetch_concurrent,
                fetch_rate_per_peer: cfg.fetch_rate_per_peer,
                replay_buffer: REPLAY_BUFFER,
                write_buffer: WRITE_BUFFER,
                blocker: cfg.blocker,
                buffer_pool,
            },
        );

        info!(
            participants = cfg.participants.len(),
            "created Simplex BFT consensus engine"
        );

        Self {
            context,
            application,
            application_mailbox,
            buffer,
            buffer_mailbox,
            marshal,
            marshal_mailbox,
            consensus,
        }
    }

    /// Starts the consensus engine.
    ///
    /// This method takes ownership of all the P2P channels and starts the
    /// various actors that make up the consensus engine.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        self,
        pending: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        recovered: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        resolver: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        broadcast: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        marshal: (
            mpsc::Receiver<handler::Message<SimplexBlock>>,
            impl Resolver<Key = handler::Request<SimplexBlock>>,
        ),
    ) -> Handle<()> {
        let context = self.context.clone();
        context.spawn(move |_| async move {
            self.run(pending, recovered, resolver, broadcast, marshal)
                .await;
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn run(
        self,
        pending: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        recovered: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        resolver: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        broadcast: (
            impl Sender<PublicKey = PublicKey>,
            impl Receiver<PublicKey = PublicKey>,
        ),
        marshal: (
            mpsc::Receiver<handler::Message<SimplexBlock>>,
            impl Resolver<Key = handler::Request<SimplexBlock>>,
        ),
    ) {
        // Start the application actor
        let application_handle = self.context.spawn({
            let application = self.application;
            let marshal_mailbox = self.marshal_mailbox.clone();
            move |_| async move {
                application.run(marshal_mailbox).await;
            }
        });

        // Start the buffer
        let buffer_handle = self.buffer.start(broadcast);

        // Start marshal
        let marshal_handle =
            self.marshal
                .start(self.application_mailbox, self.buffer_mailbox, marshal);

        // Start consensus
        let consensus_handle = self.consensus.start(pending, recovered, resolver);

        // Wait for any actor to finish
        if let Err(e) = try_join_all(vec![
            application_handle,
            buffer_handle,
            marshal_handle,
            consensus_handle,
        ])
        .await
        {
            error!(?e, "consensus engine failed");
        } else {
            warn!("consensus engine stopped");
        }
    }
}

/// Metrics from the consensus engine.
#[derive(Debug, Clone, Default)]
pub struct EngineMetrics {
    /// Current view number.
    pub view: u64,
    /// Last finalized height.
    pub finalized_height: u64,
    /// Number of pending transactions.
    pub pending_transactions: usize,
    /// Whether the engine is the current leader.
    pub is_leader: bool,
}

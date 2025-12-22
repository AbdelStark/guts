//! Application actor for Simplex BFT consensus.
//!
//! This module provides the application layer that interfaces between the
//! Simplex consensus engine and the Guts application logic.

use super::block::SimplexBlock;
use super::types::Scheme;
use commonware_consensus::{
    marshal::{self, Update},
    simplex::types::Context,
    types::{Epoch, Round, View},
    Automaton, Relay, Reporter,
};
use commonware_cryptography::{ed25519::PublicKey, sha256::Digest, Digestible, Hasher, Sha256};
use commonware_macros::select;
use commonware_runtime::{Clock, Metrics, Spawner};
use commonware_utils::SystemTimeExt;
use futures::{
    channel::{mpsc, oneshot},
    future::{self, Either},
    SinkExt, StreamExt,
};
use rand::Rng;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Type alias for the finalization callback.
pub type FinalizedCallback = Arc<dyn Fn(&SimplexBlock) + Send + Sync>;

/// Genesis message used during initialization.
const GENESIS: &[u8] = b"guts-genesis";

/// Milliseconds in the future to allow for block timestamps.
const SYNCHRONY_BOUND: u64 = 500;

/// Configuration for the application actor.
pub struct Config {
    /// Size of the mailbox channel.
    pub mailbox_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { mailbox_size: 1024 }
    }
}

/// Messages sent to the application.
pub enum Message {
    /// Request for genesis digest.
    Genesis { response: oneshot::Sender<Digest> },
    /// Request to propose a new block.
    Propose {
        round: Round,
        parent: (View, Digest),
        response: oneshot::Sender<Digest>,
    },
    /// Request to broadcast a block.
    Broadcast { payload: Digest },
    /// Request to verify a proposed block.
    Verify {
        round: Round,
        parent: (View, Digest),
        payload: Digest,
        response: oneshot::Sender<bool>,
    },
    /// A block has been finalized.
    Finalized { block: SimplexBlock },
}

/// Mailbox for the application actor.
#[derive(Clone)]
pub struct Mailbox {
    sender: mpsc::Sender<Message>,
}

impl Mailbox {
    /// Creates a new mailbox.
    pub(super) fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}

impl Automaton for Mailbox {
    type Digest = Digest;
    type Context = Context<Self::Digest, PublicKey>;

    async fn genesis(&mut self, _epoch: Epoch) -> Self::Digest {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Genesis { response })
            .await
            .expect("Failed to send genesis");
        receiver.await.expect("Failed to receive genesis")
    }

    async fn propose(
        &mut self,
        context: Context<Self::Digest, PublicKey>,
    ) -> oneshot::Receiver<Self::Digest> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Propose {
                round: context.round,
                parent: context.parent,
                response,
            })
            .await
            .expect("Failed to send propose");
        receiver
    }

    async fn verify(
        &mut self,
        context: Context<Self::Digest, PublicKey>,
        payload: Self::Digest,
    ) -> oneshot::Receiver<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Verify {
                round: context.round,
                parent: context.parent,
                payload,
                response,
            })
            .await
            .expect("Failed to send verify");
        receiver
    }
}

impl Relay for Mailbox {
    type Digest = Digest;

    async fn broadcast(&mut self, digest: Self::Digest) {
        self.sender
            .send(Message::Broadcast { payload: digest })
            .await
            .expect("Failed to send broadcast");
    }
}

impl Reporter for Mailbox {
    type Activity = Update<SimplexBlock>;

    async fn report(&mut self, update: Self::Activity) {
        let Update::Block(block) = update else {
            return;
        };
        self.sender
            .send(Message::Finalized { block })
            .await
            .expect("Failed to send finalized");
    }
}

/// Application actor that handles consensus callbacks.
pub struct Actor<R: Rng + Spawner + Metrics + Clock> {
    context: R,
    hasher: Sha256,
    mailbox: mpsc::Receiver<Message>,

    /// Callback for when a block is finalized.
    on_finalized: Option<FinalizedCallback>,
}

impl<R: Rng + Spawner + Metrics + Clock> Actor<R> {
    /// Creates a new application actor.
    pub fn new(context: R, config: Config) -> (Self, Mailbox) {
        let (sender, mailbox) = mpsc::channel(config.mailbox_size);
        (
            Self {
                context,
                hasher: Sha256::new(),
                mailbox,
                on_finalized: None,
            },
            Mailbox::new(sender),
        )
    }

    /// Sets the callback for finalized blocks.
    pub fn on_finalized<F>(mut self, callback: F) -> Self
    where
        F: Fn(&SimplexBlock) + Send + Sync + 'static,
    {
        self.on_finalized = Some(Arc::new(callback));
        self
    }

    /// Runs the application actor.
    pub async fn run(mut self, mut marshal: marshal::Mailbox<Scheme, SimplexBlock>) {
        // Compute genesis digest
        self.hasher.update(GENESIS);
        let genesis_parent = self.hasher.finalize();
        let genesis = SimplexBlock::new(genesis_parent, 0, 0, [0u8; 32], 0, [0u8; 32]);
        let genesis_digest = genesis.digest();

        let built: Option<(Round, SimplexBlock)> = None;
        let built = Arc::new(Mutex::new(built));

        while let Some(message) = self.mailbox.next().await {
            match message {
                Message::Genesis { response } => {
                    // Return the digest of the genesis block
                    let _ = response.send(genesis_digest);
                }
                Message::Propose {
                    round,
                    parent,
                    response,
                } => {
                    // Get the parent block
                    let parent_request = if parent.1 == genesis_digest {
                        Either::Left(future::ready(Ok(genesis.clone())))
                    } else {
                        Either::Right(
                            marshal
                                .subscribe(Some(Round::new(round.epoch(), parent.0)), parent.1)
                                .await,
                        )
                    };

                    // Build the new block in a separate task
                    let built_clone = built.clone();
                    let context_clone = self.context.clone();
                    context_clone.clone().spawn(move |_ctx| async move {
                        select! {
                            parent_result = parent_request => {
                                let parent_block = parent_result.unwrap();

                                // Create timestamp
                                let mut current = context_clone.current().epoch_millis();
                                if current <= parent_block.timestamp {
                                    current = parent_block.timestamp + 1;
                                }

                                // Create new block
                                let block = SimplexBlock::new(
                                    parent_block.digest(),
                                    parent_block.height + 1,
                                    current,
                                    [0u8; 32], // State root computed elsewhere
                                    0,         // TX count
                                    [0u8; 32], // TX root
                                );
                                let digest = block.digest();

                                {
                                    let mut built = built_clone.lock().unwrap();
                                    *built = Some((round, block));
                                }

                                let result = response.send(digest);
                                info!(?round, ?digest, success = result.is_ok(), "proposed new block");
                            }
                        }
                    });
                }
                Message::Broadcast { payload } => {
                    // Get the built block and broadcast it
                    let Some(built_block) = built.lock().unwrap().clone() else {
                        warn!(?payload, "missing block to broadcast");
                        continue;
                    };

                    debug!(
                        ?payload,
                        round = ?built_block.0,
                        height = built_block.1.height,
                        "broadcast requested"
                    );
                    marshal.broadcast(built_block.1.clone()).await;
                }
                Message::Verify {
                    round,
                    parent,
                    payload,
                    response,
                } => {
                    // Get parent and verify the block
                    let parent_request = if parent.1 == genesis_digest {
                        Either::Left(future::ready(Ok(genesis.clone())))
                    } else {
                        Either::Right(
                            marshal
                                .subscribe(Some(Round::new(round.epoch(), parent.0)), parent.1)
                                .await,
                        )
                    };

                    let mut marshal_clone = marshal.clone();
                    let context_clone = self.context.clone();
                    context_clone.clone().spawn(move |_ctx| async move {
                        let block_request = marshal_clone.subscribe(None, payload).await;

                        select! {
                            results = futures::future::try_join(parent_request, block_request) => {
                                let (parent_block, block) = results.unwrap();

                                // Verify block
                                if block.height != parent_block.height + 1 {
                                    let _ = response.send(false);
                                    return;
                                }
                                if block.parent != parent_block.digest() {
                                    let _ = response.send(false);
                                    return;
                                }
                                if block.timestamp <= parent_block.timestamp {
                                    let _ = response.send(false);
                                    return;
                                }
                                let current = context_clone.current().epoch_millis();
                                if block.timestamp > current + SYNCHRONY_BOUND {
                                    let _ = response.send(false);
                                    return;
                                }

                                // Mark as verified
                                marshal_clone.verified(round, block).await;

                                let _ = response.send(true);
                            }
                        }
                    });
                }
                Message::Finalized { block } => {
                    info!(
                        height = block.height,
                        digest = ?block.digest(),
                        "processed finalized block"
                    );

                    // Call the finalization callback if set
                    if let Some(ref callback) = self.on_finalized {
                        callback(&block);
                    }
                }
            }
        }
    }
}

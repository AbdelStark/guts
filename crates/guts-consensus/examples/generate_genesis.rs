//! Generate genesis configuration for E2E devnet.
//!
//! This generates a genesis.json file with validators using deterministic
//! keys matching those in docker-compose.e2e.yml (seeds 1, 2, 3, 4).
//!
//! Usage:
//!   cargo run --example generate_genesis > genesis.json

use commonware_cryptography::{ed25519::PrivateKey, PrivateKeyExt, Signer};

fn main() {
    // Generate validators for seeds 1, 2, 3, 4 (matching docker-compose.e2e.yml)
    // Docker uses: GUTS_PRIVATE_KEY: "0100000000000000..." where first byte is the seed
    // IP addresses match docker-compose.e2e.yml network: 172.29.0.{11,12,13,14}
    let validators: Vec<serde_json::Value> = (1..=4u64)
        .map(|seed| {
            let key = PrivateKey::from_seed(seed);
            let pubkey = hex::encode(key.public_key().as_ref());

            serde_json::json!({
                "name": format!("validator{}", seed),
                "pubkey": pubkey,
                "weight": 100,
                "addr": format!("172.29.0.{}:9000", 10 + seed)
            })
        })
        .collect();

    let genesis = serde_json::json!({
        "chain_id": "guts-e2e-devnet",
        "timestamp": 1703116800000_u64,  // Fixed timestamp for reproducibility
        "validators": validators,
        "repositories": [],
        "consensus": {
            "block_time_ms": 1000,
            "max_txs_per_block": 100,
            "max_block_size": 10485760,
            "view_timeout_multiplier": 2.0,
            "min_validators": 4,
            "max_validators": 100
        }
    });

    println!("{}", serde_json::to_string_pretty(&genesis).unwrap());
}

# Fuzz Testing for Guts

This directory contains fuzz testing targets for critical parsing and validation code in the Guts project.

## Prerequisites

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

Note: `cargo-fuzz` requires nightly Rust:

```bash
rustup install nightly
```

## Running Fuzz Tests

Run a specific fuzz target:

```bash
# Fuzz P2P message parsing
cargo +nightly fuzz run fuzz_p2p_message

# Fuzz Git pack file parsing
cargo +nightly fuzz run fuzz_pack_parser

# Fuzz username validation
cargo +nightly fuzz run fuzz_username_validation

# Fuzz Git pkt-line protocol parsing
cargo +nightly fuzz run fuzz_pktline
```

## Available Fuzz Targets

| Target | Description |
|--------|-------------|
| `fuzz_p2p_message` | Tests P2P message encoding/decoding (RepoAnnounce, SyncRequest, ObjectData, RefUpdate) |
| `fuzz_pack_parser` | Tests Git pack file parsing (header validation, object decompression, checksum verification) |
| `fuzz_username_validation` | Tests username validation rules (length, characters, reserved names) |
| `fuzz_pktline` | Tests Git pkt-line protocol parsing (packet reading, flush packets, error handling) |

## Options

```bash
# Run with specific options
cargo +nightly fuzz run fuzz_p2p_message -- \
    -max_len=1024 \           # Maximum input size
    -max_total_time=60 \      # Run for 60 seconds
    -jobs=4                   # Run 4 parallel jobs

# List all targets
cargo +nightly fuzz list

# Run with coverage
cargo +nightly fuzz coverage fuzz_p2p_message
```

## Corpus

The fuzzer automatically saves interesting inputs to the `corpus/` directory. These inputs are used to seed future fuzzing runs for better coverage.

## Crashes

If a crash is found, it will be saved to the `artifacts/` directory. To reproduce:

```bash
cargo +nightly fuzz run fuzz_p2p_message artifacts/fuzz_p2p_message/crash-XXXX
```

## Adding New Fuzz Targets

1. Create a new file in `fuzz_targets/`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing code here
});
```

2. Add entry to `Cargo.toml`:

```toml
[[bin]]
name = "fuzz_new_target"
path = "fuzz_targets/fuzz_new_target.rs"
test = false
doc = false
bench = false
```

## CI Integration

Fuzz tests can be run in CI with a time limit:

```yaml
- name: Fuzz Tests
  run: |
    cargo +nightly fuzz run fuzz_p2p_message -- -max_total_time=60
    cargo +nightly fuzz run fuzz_pack_parser -- -max_total_time=60
```

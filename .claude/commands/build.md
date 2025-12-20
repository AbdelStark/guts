---
name: build
description: Build the entire Guts workspace
---

Build the Guts project:

```bash
cargo build --workspace --all-features
```

Then run clippy for linting:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Report any compilation errors or warnings.

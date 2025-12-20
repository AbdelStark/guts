---
name: test
description: Run all tests with coverage
---

Run the complete test suite:

```bash
cargo test --workspace --all-features
```

If there are failures, analyze them and suggest fixes.

For coverage (if cargo-llvm-cov is installed):

```bash
cargo llvm-cov --workspace --all-features --html
```

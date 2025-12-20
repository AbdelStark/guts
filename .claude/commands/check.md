---
name: check
description: Run all checks (format, lint, test)
---

Run the complete check suite:

1. Format check:
```bash
cargo fmt --all -- --check
```

2. Clippy:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

3. Tests:
```bash
cargo test --workspace --all-features
```

4. Documentation:
```bash
cargo doc --workspace --no-deps
```

Report any issues found.

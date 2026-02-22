# CI Quality Gates

Required gates:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo llvm-cov --all-features --all-targets --workspace --summary-only
```

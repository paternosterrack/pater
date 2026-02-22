# Testing Strategy

Current priorities:
- Integration and E2E coverage for common flows.
- Command-surface checks to catch CLI drift.
- Contract checks for machine-readable outputs.

Run:

```bash
cargo test --all
cargo llvm-cov --all-features --all-targets --workspace --summary-only
```

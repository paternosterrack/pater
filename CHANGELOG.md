# Changelog

All notable changes to this project are tracked here.

## [Unreleased]

### Added
- E2E/integration coverage for common user flows (`tests/e2e_flows.rs`).
- CLI command-surface matrix test (`tests/cli_command_matrix.rs`).
- Shared integration test helpers (`tests/common/mod.rs`).
- CI rustdoc gate (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`).
- CI llvm-cov summary step (`cargo llvm-cov --all-features --all-targets --workspace --summary-only`).

### Changed
- Refactored architecture to keep `main.rs` bootstrap-focused and move runtime logic into layered modules.
- Internal docs moved to rustdoc-first model; module guidance now lives in `//!` comments.

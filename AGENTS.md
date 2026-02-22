# AGENTS.md

Essential rules for contributors and coding agents in this repository.

## Scope

This repo (`pater`) is the CLI/runtime interface for `rack` marketplaces.

## When changing install/discovery/marketplace behavior

1. Run required quality gates:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all`
   - `cargo llvm-cov --all-features --all-targets --workspace --summary-only`
2. Keep command UX aligned with marketplace flow:
   - marketplace add/list/update
   - discover/show/install/remove/installed
   - validate/hooks list
3. Maintain machine-friendly `--json` outputs for automation.
4. When adding or changing a CLI command/option, add/update integration tests in the same change (`tests/cli.rs` and command-surface check in `tests/cli_command_matrix.rs`).
5. CI tests must pass on every PR/push (`.github/workflows/tests.yml`).

## Rack integration contract (mandatory)

Any CLI change that affects plugin resolution/install must be validated against `../rack`:

- `pater --marketplace ../rack validate`
- `pater --marketplace ../rack search <query>`
- `pater --marketplace ../rack install <plugin>@paternoster-rack`

## Versioning discipline

- If CLI behavior changes in user-visible ways, bump `Cargo.toml` version following semver intent.
- Keep `Cargo.lock` committed for this CLI app.

## Documentation policy (mandatory)

1. Rustdoc is the source of truth for internal docs.
   - Keep module-level `//!` docs in sync with code behavior.
   - Root `README.md` remains public overview only.
2. On every behavior-changing PR/push, run doc checks:
   - `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
3. If you change command flow, service boundaries, or domain models, update docs in the same change.
   - No code-only architecture refactors without corresponding doc updates.
4. CI doc checks must pass on every PR/push.

### Doc ownership by layer

- `src/commands/*` docs own: CLI flow, orchestration contracts, command semantics.
- `src/services/*` docs own: behavior, side effects, invariants, failure modes.
- `src/domain/*` docs own: JSON/data schema guarantees and compatibility notes.

### Doc touchpoints checklist (required in behavior-changing PRs)

- Command behavior changed? Update `src/commands/*` rustdoc.
- Service logic/side effects changed? Update `src/services/*` rustdoc.
- JSON shape or shared model changed? Update `src/domain/models.rs` rustdoc + `docs/contracts/*` when relevant.
- Add a short entry to `CHANGELOG.md` under `Unreleased`.

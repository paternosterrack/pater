# AGENTS.md

Essential rules for contributors and coding agents in this repository.

## Scope

This repo (`pater`) is the CLI/runtime interface for `rack` marketplaces.

## When changing install/discovery/marketplace behavior

1. Run required quality gates:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all`
2. Keep command UX aligned with marketplace flow:
   - marketplace add/list/update
   - discover/show/install/remove/installed
   - validate/hooks list
3. Maintain machine-friendly `--json` outputs for automation.

## Rack integration contract (mandatory)

Any CLI change that affects plugin resolution/install must be validated against `../rack`:

- `pater --marketplace ../rack validate`
- `pater --marketplace ../rack discover`
- `pater --marketplace ../rack install <plugin>@paternoster-rack`

## Versioning discipline

- If CLI behavior changes in user-visible ways, bump `Cargo.toml` version following semver intent.
- Keep `Cargo.lock` committed for this CLI app.

## README policy

- Do **not** change root `README.md` unless explicitly requested by the user.

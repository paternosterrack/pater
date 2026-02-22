# Architecture

Cargo-inspired layering:

- `src/main.rs` — bootstrap and wiring.
- `src/commands/*` — CLI orchestration.
- `src/services/*` — business logic and side effects.
- `src/domain/*` — shared models/constants and JSON-shape impact.

Rule: behavior logic belongs in services; command handlers stay thin.

Runtime-first storage is canonical:

- Installed plugin assets: `~/.local/share/pater/runtime/plugins`
- Runtime registry: `~/.local/share/pater/runtime/registry.json`
- Adapter bridge configs: `~/.local/share/pater/runtime/bridges/*.json`

Adapter sync generates bridge/config outputs pointing to runtime paths instead of maintaining duplicated adapter-native plugin trees as the source of truth.

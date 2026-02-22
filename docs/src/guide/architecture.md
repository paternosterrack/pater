# Architecture

Cargo-inspired layering:

- `src/main.rs` — bootstrap and wiring.
- `src/commands/*` — CLI orchestration.
- `src/services/*` — business logic and side effects.
- `src/domain/*` — shared models/constants and JSON-shape impact.

Rule: behavior logic belongs in services; command handlers stay thin.

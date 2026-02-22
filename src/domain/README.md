# Domain module

Shared data models and constants used across commands and services.

## Purpose
- Keep DTOs/report structs in one place.
- Avoid cyclic imports and duplicated type definitions.
- Make JSON output schema changes explicit and reviewable.

## Files
- `models.rs` — state, policy, report/output structs.
- `constants.rs` — stable constants (e.g. official trust key).
- `mod.rs` — module exports.

## Rule of thumb
Domain types should be data-only: no filesystem/network side effects.

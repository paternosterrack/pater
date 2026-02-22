//! Shared data model layer (structs/constants only).
//!
//! ## Purpose
//! - Keep DTO/report structs in one place.
//! - Avoid cyclic imports and duplicated type definitions.
//! - Make JSON output schema changes explicit and reviewable.
//!
//! ## Files
//! - `models.rs` — state, policy, report/output structs.
//! - `constants.rs` — stable constants (e.g. official trust key).
//!
//! ## Rule of thumb
//! Domain types should be data-only: no filesystem/network side effects.
//!
//! ## Compatibility note
//! Changes in these structs can affect `--json` outputs and integration contracts.
//! Keep schema-impacting changes explicit and synchronized with `docs/contracts/*`.

pub mod constants;
pub mod models;

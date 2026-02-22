//! Command handler layer.
//!
//! This module owns CLI-oriented orchestration and output wiring.
//!
//! ## Files
//! - `admin.rs` — trust/rack/author command trees.
//! - `runtime.rs` — search/install/apply/update/remove/list/etc.
//!
//! ## Principles
//! - Parse/match CLI inputs here.
//! - Delegate business logic to `services/*`.
//! - Keep behavior and output schema stable.
//!
//! ## Common flows (CLI examples)
//! ```text
//! pater search typescript
//! pater show commit-commands@paternoster-rack
//! pater install commit-commands@paternoster-rack
//! pater apply commit-commands@paternoster-rack --target-adapter codex
//! pater check
//! ```

pub mod admin;
pub mod runtime;

pub use admin::{handle_author_commands, handle_rack_commands, handle_trust_commands};
pub use runtime::handle_runtime_commands;

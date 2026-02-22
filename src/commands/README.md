# Commands module

CLI command handlers. These functions orchestrate services and output formatting.

## Files
- `admin.rs` — trust/rack/author command trees.
- `runtime.rs` — search/install/apply/update/remove/list/etc.
- `mod.rs` — exports.

## Principles
- Parse/match CLI inputs here.
- Delegate business logic to `services/*`.
- Keep behavior and output schema stable.

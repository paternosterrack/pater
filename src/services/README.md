# Services module

Business logic and side-effect helpers grouped by concern.

## Service map
- `marketplace.rs` — discover/show/recommend/policy enforcement/update flow.
- `adapters.rs` — materialized adapter sync/smoke/doctor.
- `trust.rs` — trusted key storage + signature verification.
- `rack_ops.rs` — rack maintainer workflows (sync/audit/sign).
- `authoring.rs` — plugin/skill/subagent/hook/mcp authoring mutations.
- `storage.rs` — local state/lock persistence + audit log.
- `release_check.rs` — release report assembly.
- `policy.rs` — source canonicalization and allowlist matching.
- `output.rs` — JSON/text output helpers.

## Conventions
- Prefer pure helpers where possible.
- Side effects should be explicit and localized.
- Keep command handlers thin; delegate to services.

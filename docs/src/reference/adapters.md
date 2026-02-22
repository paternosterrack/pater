# Adapter Behavior

`pater adapter sync` materializes installed plugins into adapter-specific locations and writes adapter shims.

Health checks:
- `adapter smoke` verifies plugin materialization + shims.
- `adapter doctor` includes smoke + config/wrapper checks.

# Adapter Behavior

`pater adapter sync` is runtime-first.

It ensures installed plugins are materialized under `~/.local/share/pater/runtime/plugins`,
writes runtime registry artifacts, and generates adapter bridge configs that reference runtime paths.

Health checks:
- `adapter smoke` verifies runtime materialization + shims/bridges.
- `adapter doctor` includes smoke + config/wrapper checks.

Related runtime commands:
- `pater runtime path`
- `pater runtime status`
- `pater runtime sync --target all`

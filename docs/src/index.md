# pater

`pater` helps you find, install, and safely use plugins for AI coding tools.

If you use Claude Code, Codex, or OpenClaw, `pater` gives you one consistent way to:

- **discover** plugins from curated marketplaces,
- **install/apply** them to your tool setup,
- **control risk** with trust and policy checks,
- **keep your setup healthy** with smoke/doctor/check commands.

## Who this is for

- **Developers** who want faster setup and repeatable plugin workflows.
- **Teams** who need policy guardrails and predictable environments.
- **Maintainers** who publish and verify plugin catalogs.

## Start here

1. [Installation](getting-started/installation.md)
2. [First Workflow](getting-started/first-workflow.md)
3. [Lifecycle Commands](commands/lifecycle.md)

## Example: from zero to working plugin

```bash
pater trust init
pater --marketplace ../rack search commit
pater --marketplace ../rack install commit-commands@paternoster-rack
pater adapter doctor
pater check
```

## Documentation map

- **Getting Started** — quickest path to value.
- **Guide** — practical workflows and architecture context.
- **Reference** — exact behavior, constraints, and contracts.
- **Commands** — command-by-command usage.
- **Operations** — CI gates and docs publishing.

For generated internals from code comments, see rustdoc (`/api/` when hosted).

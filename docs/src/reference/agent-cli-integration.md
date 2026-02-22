# Agent CLI Integration

`pater` integrates with Claude, Codex, and OpenClaw using runtime-first bridges.

## Canonical runtime source

`pater` stores installed plugin/runtime assets under:

- `~/.local/share/pater/runtime/plugins`
- `~/.local/share/pater/runtime/registry.json`

Registry artifacts include plugin, skill, hook, subagent, and mcp views.

## Adapter bridge outputs

`pater adapter sync` and `pater runtime sync` generate bridge configs that point to runtime paths.

- Claude: `~/.claude/pater.plugins.json`
- Codex: `~/.codex/pater.plugins.json`
- OpenClaw: `~/.openclaw/workspace/skills/.pater-index.json`

Compatibility wrappers are also maintained in `~/.local/bin`:

- `pater-claude`
- `pater-codex`
- `pater-openclaw`

## Runtime-first behavior

Agent-native directories are not the canonical plugin source.
Bridge/config generation is the primary sync mechanism.

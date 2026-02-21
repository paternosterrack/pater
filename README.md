# pater 🛠️

`pater` is the plugin manager for **Paternoster Rack**.

It is built for agent CLIs (Claude, Codex, OpenClaw) with trust, policy, and adapter sync in one tool.

## Install

### macOS / Linux / WSL

```bash
curl -fsSL https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.sh | sh
```

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.ps1 | iex
```

## For Users

### 1) Bootstrap trust + policy

```bash
pater trust init
mkdir -p ~/.config/pater
cp examples/policy-safe-default.toml ~/.config/pater/policy.toml
```

### 2) Install and verify

```bash
pater search typescript
pater install typescript-lsp@paternoster-rack
pater adapter doctor
pater check
```

### 3) Restart your agent CLI

Restart Claude/Codex/OpenClaw to load newly synced plugins.

## For Developers

`../rack` is **dev-only local path usage** when `pater` and `rack` are cloned side-by-side.

Example:

```bash
# local development against sibling rack repo
pater --marketplace ../rack validate
```

Production/default usage is remote marketplace `paternosterrack/rack`.

## Core Commands

```bash
# remotes
pater remote add <source>
pater remote list
pater remote update

# lifecycle
pater search [query]
pater recommend --context "task context for agent"
pater plan --intent "task" --agent all|claude|codex|openclaw
pater show <plugin[@marketplace]>
pater install <plugin@marketplace> [--scope user|project|local]
pater apply <plugin@marketplace> --target-adapter all|claude|codex|openclaw [--scope user|project|local]
pater ensure --intent "task" --agent all|claude|codex|openclaw
pater update [plugin]
pater remove <plugin>
pater list
pater capabilities --agent all|claude|codex|openclaw
pater policy eval <plugin[@marketplace]> --agent all|claude|codex|openclaw

# authoring (all entities belong to a plugin)
pater author plugin create <plugin> --rack-dir ../rack --description "..."
pater author plugin update <plugin> --rack-dir ../rack [--description "..."] [--version x.y.z]
pater author plugin remove <plugin> --rack-dir ../rack

pater author skill create <plugin> <skill> --rack-dir ../rack --description "..."
pater author skill remove <plugin> <skill> --rack-dir ../rack

pater author subagent create <plugin> <name> --rack-dir ../rack --purpose "..."
pater author subagent remove <plugin> <name> --rack-dir ../rack

pater author hook create <plugin> --rack-dir ../rack --agent codex --event pre-commit --run "cargo test"
pater author hook remove <plugin> --rack-dir ../rack --agent codex --event pre-commit

pater author mcp create <plugin> <name> --rack-dir ../rack --command "mcp-server"
pater author mcp remove <plugin> <name> --rack-dir ../rack

# runtime hook discovery
pater hook list [--agent codex]

# adapters
pater adapter sync --target all|claude|codex|openclaw
pater adapter smoke --target all|claude|codex|openclaw
pater adapter doctor

# trust
pater trust init
pater trust list
pater trust status

# release gate
pater check

# rack maintainer pipeline (replaces old python scripts)
pater rack doctor --rack-dir ../rack --sign-key /path/to/key.pem
pater rack sync --rack-dir ../rack
pater rack license-audit --rack-dir ../rack
pater rack mark-unknown-external --rack-dir ../rack
pater rack sign --rack-dir ../rack --sign-key /path/to/key.pem
# or one-shot
pater rack prepare-release --rack-dir ../rack --sign-key /path/to/key.pem
```

## Policy

Policy file: `~/.config/pater/policy.toml`  
Starter template: `examples/policy-safe-default.toml`

## Machine contracts

Stable JSON contracts for agent integrations:
- `docs/contracts/README.md`
- `docs/contracts/*.schema.json`

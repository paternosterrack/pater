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
pater discover typescript
pater install typescript-lsp@paternoster-rack
pater adapter doctor
pater release-check
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
# marketplace
pater marketplace add <source>
pater marketplace list
pater marketplace update

# lifecycle
pater discover [query]
pater show <plugin[@marketplace]>
pater install <plugin@marketplace> [--scope user|project|local]
pater update [plugin]
pater remove <plugin>
pater installed

# adapters
pater adapter sync --target all|claude|codex|openclaw
pater adapter smoke --target all|claude|codex|openclaw
pater adapter doctor

# trust
pater trust init
pater trust list
pater trust status

# release gate
pater release-check

# rack maintainer pipeline (replaces old python scripts)
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

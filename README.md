# pater 🛠️

CLI for **Paternoster Rack** — an agent-agnostic plugin marketplace for Claude/Codex/OpenClaw-style tooling.

## Current state

Implemented now:
- Marketplace flow: discover/show/install/update/remove
- Default official marketplace (`paternosterrack/rack`) with cache + refresh
- Adapter sync + activation shims for Claude, Codex, OpenClaw
- Install scopes (`user|project|local`)
- Lockfile (`~/.config/pater/pater.lock`)
- Trust bootstrap + signed marketplace verification (`trust init/list/status`)
- Policy gates (permissions, denied plugins, unknown-license + external-reference controls)
- Audit log (`~/.config/pater/audit.jsonl`)
- One-shot health gate: `release-check`

---

## Install

### Recommended (macOS/Linux/WSL)

```bash
curl -fsSL https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.sh | sh
```

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.ps1 | iex
```

---

## Quick start

```bash
pater trust init
pater discover commit
pater install commit-commands@paternoster-rack
pater adapter doctor
pater release-check
```

---

## Key commands

```bash
# marketplace
pater marketplace add <source>
pater marketplace list
pater marketplace update

# plugin lifecycle
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
```

---

## Policy

Policy file: `~/.config/pater/policy.toml`

See starter policy:
- `examples/policy-safe-default.toml`

Notable controls:
- `require_signed_marketplace`
- `block_unknown_licenses`
- `allow_unknown_license_plugins`
- `allow_external_reference_installs`
- `allow_external_reference_plugins`

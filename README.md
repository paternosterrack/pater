# pater 🛠️

CLI for **Paternoster Rack** — a central, agent-agnostic plugin marketplace.

## Install

### Install (Recommended)

#### macOS, Linux, WSL

```bash
curl -fsSL https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.sh | sh
```

#### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.ps1 | iex
```

#### Windows CMD

```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/paternosterrack/pater/main/scripts/install.ps1 | iex"
```

### Homebrew

```bash
brew tap paternosterrack/tap
brew install pater
```

### WinGet

```powershell
winget install PaternosterRack.Pater
```

> Homebrew tap + WinGet package are intended distribution channels; if unavailable yet, use the recommended installer.

## Quick start

```bash
pater --marketplace ../rack discover
pater --marketplace ../rack install commit-commands@paternoster-rack
pater installed
```

## CLI

```bash
pater marketplace add <source>
pater marketplace list
pater marketplace update

pater discover [query]
pater show <plugin[@marketplace]>
pater install <plugin@marketplace>
pater remove <plugin>
pater installed

pater hooks list [--agent codex|claude|openclaw]
pater validate
```

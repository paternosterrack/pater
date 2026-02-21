# Pater Beta Rollout Checklist

## 1) Bootstrap trust and policy

```bash
pater trust init
mkdir -p ~/.config/pater
cp examples/policy-safe-default.toml ~/.config/pater/policy.toml
```

## 2) Install a verified plugin

```bash
pater install typescript-lsp@paternoster-rack
pater adapter sync --target all
pater adapter doctor
```

## 3) Validate release health

```bash
pater release-check
```

## 4) Real-user test loop (2-3 users)

- Restart Claude/Codex/OpenClaw
- Confirm installed plugin is usable
- Capture failures and run:
  - `pater --json adapter doctor`
  - `pater --json release-check`

## Success criteria

- Trust check: signed marketplace verified
- Adapter doctor: all adapters `ok`
- License gate: `ok` or explicit approved exceptions

# Dev and Maintainer Workflows

This page is for local development and release-maintainer tasks.

## Local `rack` development (side-by-side repos)

When `pater` and `rack` are checked out as siblings:

```bash
pater --marketplace ../rack validate
pater --marketplace ../rack search commit
pater --marketplace ../rack install commit-commands@paternoster-rack
```

This is a dev-only workflow. User onboarding should use the default remote marketplace.

## Runtime-first local verification

`pater` now uses a canonical runtime store:

- `~/.local/share/pater/runtime/plugins/*`
- `~/.local/share/pater/runtime/registry.json`
- `~/.local/share/pater/runtime/bridges/*.json`

Useful checks:

```bash
pater runtime path
pater runtime status
pater runtime sync --target all
```

## Rack maintainer release flow

```bash
pater rack doctor --rack-dir ../rack --sign-key /path/to/key.pem
pater rack sync --rack-dir ../rack
pater rack license-audit --rack-dir ../rack
pater rack sign --rack-dir ../rack --sign-key /path/to/key.pem
pater rack prepare-release --rack-dir ../rack --sign-key /path/to/key.pem
```

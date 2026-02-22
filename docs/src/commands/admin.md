# Admin Commands

Trust / Rack / Authoring groups:

```bash
pater trust init
pater trust status

pater rack doctor --rack-dir ../rack --sign-key <key.pem>
pater rack prepare-release --rack-dir ../rack --sign-key <key.pem>

pater author plugin create <name> --rack-dir ../rack --description "..."
```

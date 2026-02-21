# Pater JSON Contracts (v1)

These contracts define the machine-facing response shape for agent integrations.

## Envelope

All `--json` responses follow:

- success: `{"ok": true, "data": <payload>, "meta"?: {...}}`
- failure: `{"ok": false, "error": {code, message, hint, retryable}, "meta"?: {...}}`

## Stable error codes (v1)

- `POLICY_DENY`
- `SIGNATURE_INVALID`
- `NOT_FOUND`
- `PERMISSION_DELTA_BLOCKED`
- `INTERNAL_ERROR`

## Command payload schemas

- `capabilities.schema.json`
- `policy-eval.schema.json`
- `plan.schema.json`
- `ensure.schema.json`
- `recommend.schema.json`

These schemas describe the `data` field for successful responses.

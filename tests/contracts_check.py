#!/usr/bin/env python3
import json
import os
import subprocess
import tempfile
from pathlib import Path

from jsonschema import Draft202012Validator, RefResolver


ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "docs" / "contracts"


def run_json(home: Path, rack: Path, *args: str):
    env = os.environ.copy()
    orig_home = Path(os.environ.get("HOME", ""))
    env["HOME"] = str(home)
    env.setdefault("CARGO_HOME", str(orig_home / ".cargo"))
    env.setdefault("RUSTUP_HOME", str(orig_home / ".rustup"))
    cmd = ["cargo", "run", "--", "--json", "--marketplace", str(rack)] + list(args)
    out = subprocess.check_output(cmd, cwd=ROOT, env=env, text=True)
    return json.loads(out)


def load_schema(name: str):
    schema_path = SCHEMAS / name
    schema = json.loads(schema_path.read_text())

    store = {}
    for f in SCHEMAS.glob("*.schema.json"):
        doc = json.loads(f.read_text())
        store[f.resolve().as_uri()] = doc
        sid = doc.get("$id")
        if sid:
            store[sid] = doc

    resolver = RefResolver(base_uri=schema_path.resolve().as_uri(), referrer=schema, store=store)
    return schema, resolver


def validate_data(schema_name: str, data):
    schema, resolver = load_schema(schema_name)
    Draft202012Validator(schema, resolver=resolver).validate(data)


def make_fixture_rack(base: Path):
    rack = base / "rack"
    plugin = rack / "plugins" / "typescript-lsp"
    (rack / ".pater").mkdir(parents=True, exist_ok=True)
    (plugin / ".claude-plugin").mkdir(parents=True, exist_ok=True)
    (plugin / "skills" / "ts").mkdir(parents=True, exist_ok=True)

    (plugin / "LICENSE").write_text("MIT License\nPermission is hereby granted")
    (plugin / ".claude-plugin" / "plugin.json").write_text(
        json.dumps({"name": "typescript-lsp", "version": "0.1.0"})
    )
    (plugin / "skills" / "ts" / "SKILL.md").write_text("# TS skill\n")

    marketplace = {
        "name": "fixture-rack",
        "owner": {"name": "Fixture"},
        "plugins": [
            {
                "name": "typescript-lsp",
                "source": "./plugins/typescript-lsp",
                "description": "TypeScript language support",
                "version": "0.1.0",
                "permissions": ["filesystem.read"],
                "skills": ["ts"],
                "hooks": [{"agent": "codex", "event": "on-demand", "run": "echo ok"}],
                "subagents": [{"name": "ts-helper", "purpose": "assist ts tasks"}],
            }
        ],
    }
    (rack / ".pater" / "marketplace.json").write_text(json.dumps(marketplace, indent=2))
    return rack


def main():
    with tempfile.TemporaryDirectory() as td:
        temp = Path(td)
        home = temp / "home"
        home.mkdir(parents=True, exist_ok=True)
        rack = make_fixture_rack(temp)

        # recommend
        rec = run_json(home, rack, "recommend", "--context", "typescript")
        assert rec["ok"] is True
        validate_data("recommend.schema.json", rec["data"])

        # plan
        plan = run_json(home, rack, "plan", "--intent", "typescript", "--agent", "all")
        assert plan["ok"] is True
        validate_data("plan.schema.json", plan["data"])

        # capabilities (before install)
        caps = run_json(home, rack, "capabilities", "--agent", "all")
        assert caps["ok"] is True
        validate_data("capabilities.schema.json", caps["data"])

        # policy eval
        pe = run_json(home, rack, "policy", "eval", "typescript-lsp@fixture-rack", "--agent", "all")
        assert pe["ok"] is True
        validate_data("policy-eval.schema.json", pe["data"])

        # ensure
        ens = run_json(home, rack, "ensure", "--intent", "typescript", "--agent", "all")
        assert ens["ok"] is True
        validate_data("ensure.schema.json", ens["data"])

    print("contracts-check: OK")


if __name__ == "__main__":
    main()

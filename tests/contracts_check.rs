use assert_cmd::cargo::cargo_bin_cmd;
use jsonschema::JSONSchema;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn run_json(home: &Path, rack: &Path, args: &[&str]) -> Value {
    let orig_home = std::env::var("HOME").unwrap_or_default();
    let mut cmd = cargo_bin_cmd!("pater");
    cmd.env("HOME", home)
        .env("CARGO_HOME", PathBuf::from(&orig_home).join(".cargo"))
        .env("RUSTUP_HOME", PathBuf::from(&orig_home).join(".rustup"))
        .args(["--json", "--marketplace", rack.to_str().unwrap()])
        .args(args);

    let out = cmd.assert().success().get_output().stdout.clone();
    serde_json::from_slice(&out).expect("valid json output")
}

fn load_schema(name: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read_to_string(root.join("docs/contracts").join(name)).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn validate(schema_name: &str, data: &Value) {
    let schema = load_schema(schema_name);
    let validator = JSONSchema::compile(&schema).expect("compile schema");
    let msgs: Vec<String> = match validator.validate(data) {
        Ok(()) => return,
        Err(errors) => errors.map(|e| e.to_string()).collect(),
    };
    panic!("schema validation failed: {}", msgs.join(" | "));
}

fn make_fixture_rack(base: &Path) -> PathBuf {
    let rack = base.join("rack");
    let plugin = rack.join("plugins/typescript-lsp");

    fs::create_dir_all(rack.join(".pater")).unwrap();
    fs::create_dir_all(plugin.join(".claude-plugin")).unwrap();
    fs::create_dir_all(plugin.join("skills/ts")).unwrap();

    fs::write(
        plugin.join("LICENSE"),
        "MIT License\nPermission is hereby granted",
    )
    .unwrap();
    fs::write(
        plugin.join(".claude-plugin/plugin.json"),
        json!({"name":"typescript-lsp","version":"0.1.0"}).to_string(),
    )
    .unwrap();
    fs::write(plugin.join("skills/ts/SKILL.md"), "# TS\n").unwrap();

    let marketplace = json!({
        "name": "fixture-rack",
        "owner": {"name": "Fixture"},
        "plugins": [{
            "name": "typescript-lsp",
            "source": "./plugins/typescript-lsp",
            "description": "TypeScript language support",
            "version": "0.1.0",
            "permissions": ["filesystem.read"],
            "skills": ["ts"],
            "hooks": [{"agent": "codex", "event": "on-demand", "run": "echo ok"}],
            "subagents": [{"name": "ts-helper", "purpose": "assist ts tasks"}]
        }]
    });
    fs::write(
        rack.join(".pater/marketplace.json"),
        serde_json::to_string_pretty(&marketplace).unwrap(),
    )
    .unwrap();

    rack
}

#[test]
fn contracts_check() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let rack = make_fixture_rack(tmp.path());

    let rec = run_json(&home, &rack, &["recommend", "--context", "typescript"]);
    assert_eq!(rec["ok"], true);
    validate("recommend.schema.json", &rec["data"]);

    let plan = run_json(
        &home,
        &rack,
        &["plan", "--intent", "typescript", "--agent", "all"],
    );
    assert_eq!(plan["ok"], true);
    validate("plan.schema.json", &plan["data"]);

    let caps = run_json(&home, &rack, &["capabilities", "--agent", "all"]);
    assert_eq!(caps["ok"], true);
    validate("capabilities.schema.json", &caps["data"]);

    let pol = run_json(
        &home,
        &rack,
        &[
            "policy",
            "eval",
            "typescript-lsp@fixture-rack",
            "--agent",
            "all",
        ],
    );
    assert_eq!(pol["ok"], true);
    validate("policy-eval.schema.json", &pol["data"]);

    let ensure = run_json(
        &home,
        &rack,
        &["ensure", "--intent", "typescript", "--agent", "all"],
    );
    assert_eq!(ensure["ok"], true);
    validate("ensure.schema.json", &ensure["data"]);
}

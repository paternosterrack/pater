use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TestEnv {
    _tmp: TempDir,
    home: PathBuf,
    rack: PathBuf,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let tmp = TempDir::new().expect("create temp dir");
        let home = tmp.path().join("home");
        fs::create_dir_all(&home).expect("create isolated home");

        let rack = make_fixture_rack(tmp.path());

        let orig_home = std::env::var("HOME").unwrap_or_default();
        let cargo_home = PathBuf::from(&orig_home).join(".cargo");
        let rustup_home = PathBuf::from(&orig_home).join(".rustup");

        Self {
            _tmp: tmp,
            home,
            rack,
            cargo_home,
            rustup_home,
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = cargo_bin_cmd!("pater");
        cmd.env("HOME", &self.home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("RUSTUP_HOME", &self.rustup_home);
        cmd
    }

    fn run_json(&self, args: &[&str]) -> Value {
        let mut cmd = self.cmd();
        let out = cmd
            .arg("--json")
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        serde_json::from_slice(&out).expect("valid json output")
    }

    fn run_json_market(&self, args: &[&str]) -> Value {
        let mut cmd = self.cmd();
        let out = cmd
            .arg("--json")
            .arg("--marketplace")
            .arg(self.rack.to_str().expect("rack path utf8"))
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        serde_json::from_slice(&out).expect("valid json output")
    }
}

fn make_fixture_rack(base: &Path) -> PathBuf {
    let rack = base.join("rack");
    let commit = rack.join("plugins/commit-commands");

    fs::create_dir_all(rack.join(".pater")).expect("create .pater");
    fs::create_dir_all(commit.join(".claude-plugin")).expect("create plugin manifest dir");

    fs::write(
        commit.join("LICENSE"),
        "MIT License\nPermission is hereby granted",
    )
    .expect("write license");
    fs::write(
        commit.join(".claude-plugin/plugin.json"),
        serde_json::json!({
            "name": "commit-commands",
            "version": "1.0.0",
            "license": "MIT"
        })
        .to_string(),
    )
    .expect("write plugin manifest");

    let marketplace = serde_json::json!({
        "name": "fixture-rack",
        "owner": {"name": "Fixture", "email": "fixture@example.com"},
        "plugins": [
            {
                "name": "commit-commands",
                "source": "./plugins/commit-commands",
                "description": "Conventional commit helpers",
                "version": "1.0.0",
                "permissions": ["filesystem.read"],
                "hooks": [{"agent": "codex", "event": "on-demand", "run": "echo ok"}],
                "subagents": [{"name": "commit-helper", "purpose": "assist commit work"}]
            }
        ]
    });
    fs::write(
        rack.join(".pater/marketplace.json"),
        serde_json::to_string_pretty(&marketplace).expect("serialize marketplace"),
    )
    .expect("write marketplace");

    rack
}

#[test]
fn trust_init_then_status_json() {
    let env = TestEnv::new();

    let init = env.run_json(&["trust", "init"]);
    assert_eq!(init["ok"], true);
    assert_eq!(init["data"], "initialized");

    let status = env.run_json(&["trust", "status"]);
    assert_eq!(status["ok"], true);
    assert_eq!(
        status["data"]["default_marketplace"],
        "paternosterrack/rack"
    );
    assert!(status["data"]["trusted_key_count"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn search_and_show_against_local_fixture_marketplace() {
    let env = TestEnv::new();

    let search = env.run_json_market(&["search", "commit"]);
    assert_eq!(search["ok"], true);
    let results = search["data"].as_array().expect("search results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "commit-commands");
    assert_eq!(results[0]["marketplace"], "fixture-rack");

    let show = env.run_json_market(&["show", "commit-commands@fixture-rack"]);
    assert_eq!(show["ok"], true);
    assert_eq!(show["data"]["name"], "commit-commands");
    assert_eq!(show["data"]["permissions"][0], "filesystem.read");
}

#[test]
fn install_list_remove_cycle() {
    let env = TestEnv::new();

    let install =
        env.run_json_market(&["install", "commit-commands@fixture-rack", "--scope", "user"]);
    assert_eq!(install["ok"], true);
    assert_eq!(install["data"]["name"], "commit-commands");

    let list = env.run_json(&["list"]);
    assert_eq!(list["ok"], true);
    let installed = list["data"].as_array().expect("installed array");
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0]["name"], "commit-commands");

    let remove = env.run_json(&["remove", "commit-commands"]);
    assert_eq!(remove["ok"], true);
    assert_eq!(remove["data"], 1);

    let list_after = env.run_json(&["list"]);
    assert_eq!(list_after["ok"], true);
    assert_eq!(
        list_after["data"]
            .as_array()
            .expect("installed array")
            .len(),
        0
    );
}

#[test]
fn apply_and_adapter_smoke_for_codex_target() {
    let env = TestEnv::new();

    let apply = env.run_json_market(&[
        "apply",
        "commit-commands@fixture-rack",
        "--target-adapter",
        "codex",
        "--scope",
        "user",
    ]);
    assert_eq!(apply["ok"], true);
    assert_eq!(apply["data"]["installed"]["name"], "commit-commands");
    assert_eq!(apply["data"]["smoke"][0]["adapter"], "codex");
    assert_eq!(apply["data"]["smoke"][0]["status"], "ok");

    let smoke = env.run_json(&["adapter", "smoke", "--target", "codex"]);
    assert_eq!(smoke["ok"], true);
    assert_eq!(smoke["data"][0]["adapter"], "codex");
    assert_eq!(smoke["data"][0]["status"], "ok");
    assert_eq!(smoke["data"][0]["checked_plugins"], 1);
}

#[test]
fn policy_denies_install_for_blocked_permission() {
    let env = TestEnv::new();

    let policy_path = env.home.join(".config/pater/policy.toml");
    fs::create_dir_all(policy_path.parent().expect("policy parent")).expect("create policy dir");
    fs::write(
        policy_path,
        r#"[general]
blocked_permissions = ["filesystem.read"]
"#,
    )
    .expect("write policy file");

    let mut cmd = env.cmd();
    let out = cmd
        .arg("--json")
        .arg("--marketplace")
        .arg(env.rack.to_str().expect("rack path utf8"))
        .args(["install", "commit-commands@fixture-rack"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let err: Value = serde_json::from_slice(&out).expect("error json output");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "POLICY_DENY");
    let msg = err["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("policy blocked permission in plugin"));
}

#[test]
fn update_reports_up_to_date_when_no_change() {
    let env = TestEnv::new();

    let _install = env.run_json_market(&["install", "commit-commands@fixture-rack"]);

    let update = env.run_json(&["update", "commit-commands"]);
    assert_eq!(update["ok"], true);

    let reports = update["data"].as_array().expect("update reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["name"], "commit-commands");
    assert_eq!(reports[0]["status"], "up_to_date");
}

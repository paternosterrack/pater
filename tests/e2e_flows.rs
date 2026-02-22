mod common;

use common::TestEnv;
use serde_json::Value;
use std::fs;

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
    assert!(results
        .iter()
        .any(|r| { r["name"] == "commit-commands" && r["marketplace"] == "fixture-rack" }));

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

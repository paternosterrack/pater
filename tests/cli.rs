use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn cmd_with_fresh_home() -> (Command, TempDir) {
    let tmp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("pater");
    cmd.env("HOME", tmp.path());
    (cmd, tmp)
}

#[test]
fn validate_marketplace() {
    let (mut cmd, _home) = cmd_with_fresh_home();
    cmd.arg("--marketplace")
        .arg("../rack")
        .arg("validate")
        .assert()
        .success()
        .stdout(contains("marketplace valid"));
}

#[test]
fn discover_json() {
    let (mut cmd, _home) = cmd_with_fresh_home();
    cmd.args(["--marketplace", "../rack", "--json", "discover", "commit"])
        .assert()
        .success()
        .stdout(contains("commit-commands"));
}

#[test]
fn hooks_list_prints() {
    let (mut cmd, _home) = cmd_with_fresh_home();
    cmd.args(["--marketplace", "../rack", "hooks", "list"])
        .assert()
        .success();
}

#[test]
fn install_and_list_installed() {
    let home = TempDir::new().unwrap();

    let mut install = cargo_bin_cmd!("pater");
    install.env("HOME", home.path());
    install
        .args([
            "--marketplace",
            "../rack",
            "install",
            "commit-commands@paternoster-rack",
        ])
        .assert()
        .success();

    let mut list = cargo_bin_cmd!("pater");
    list.env("HOME", home.path());
    list.arg("installed")
        .assert()
        .success()
        .stdout(contains("commit-commands"));
}

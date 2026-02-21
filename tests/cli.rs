use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    cargo_bin_cmd!("pater")
}

#[test]
fn validate_marketplace() {
    cmd()
        .arg("--marketplace")
        .arg("../rack")
        .arg("validate")
        .assert()
        .success()
        .stdout(contains("marketplace valid"));
}

#[test]
fn discover_json() {
    cmd()
        .args(["--marketplace", "../rack", "--json", "discover", "commit"])
        .assert()
        .success()
        .stdout(contains("commit-commands"));
}

#[test]
fn hooks_list_prints() {
    cmd()
        .args(["--marketplace", "../rack", "hooks", "list"])
        .assert()
        .success();
}

#[test]
fn install_and_list_installed() {
    cmd()
        .args([
            "--marketplace",
            "../rack",
            "install",
            "commit-commands@paternoster-rack",
        ])
        .assert()
        .success();

    cmd()
        .arg("installed")
        .assert()
        .success()
        .stdout(contains("commit-commands"));
}

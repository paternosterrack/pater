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
        .args(["--marketplace", "../rack", "--json", "discover", "lint"])
        .assert()
        .success()
        .stdout(contains("lint-tools"));
}

#[test]
fn hooks_filter_agent() {
    cmd()
        .args([
            "--marketplace",
            "../rack",
            "hooks",
            "list",
            "--agent",
            "codex",
        ])
        .assert()
        .success()
        .stdout(contains("codex"));
}

#[test]
fn install_and_list_installed() {
    cmd()
        .args([
            "--marketplace",
            "../rack",
            "install",
            "lint-tools@paternoster-rack",
        ])
        .assert()
        .success();

    cmd()
        .arg("installed")
        .assert()
        .success()
        .stdout(contains("lint-tools"));
}

use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    Command::cargo_bin("pater").unwrap()
}

#[test]
fn validate_index() {
    cmd().arg("--index").arg("../rack/index/skills.json").arg("validate").assert().success().stdout(contains("index valid"));
}

#[test]
fn search_json() {
    cmd()
        .args(["--index", "../rack/index/skills.json", "--json", "search", "lint"])
        .assert()
        .success()
        .stdout(contains("skill.lint"));
}

#[test]
fn hooks_filter_agent() {
    cmd()
        .args(["--index", "../rack/index/skills.json", "hooks", "list", "--agent", "codex"])
        .assert()
        .success()
        .stdout(contains("codex"));
}

use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::TempDir;

fn run_help(home: &TempDir, args: &[&str]) {
    let mut cmd = cargo_bin_cmd!("pater");
    cmd.env("HOME", home.path())
        .args(args)
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn every_cli_command_has_help_path() {
    let home = TempDir::new().expect("temp home");

    // top-level
    run_help(&home, &[]);

    // runtime commands
    run_help(&home, &["search"]);
    run_help(&home, &["recommend"]);
    run_help(&home, &["plan"]);
    run_help(&home, &["show"]);
    run_help(&home, &["install"]);
    run_help(&home, &["apply"]);
    run_help(&home, &["update"]);
    run_help(&home, &["remove"]);
    run_help(&home, &["list"]);
    run_help(&home, &["capabilities"]);
    run_help(&home, &["hook"]);
    run_help(&home, &["validate"]);
    run_help(&home, &["remote"]);
    run_help(&home, &["ensure"]);
    run_help(&home, &["check"]);
    run_help(&home, &["policy"]);
    run_help(&home, &["adapter"]);

    // grouped subcommands
    run_help(&home, &["hook", "list"]);
    run_help(&home, &["remote", "add"]);
    run_help(&home, &["remote", "list"]);
    run_help(&home, &["remote", "update"]);

    run_help(&home, &["policy", "eval"]);

    run_help(&home, &["adapter", "sync"]);
    run_help(&home, &["adapter", "smoke"]);
    run_help(&home, &["adapter", "doctor"]);

    run_help(&home, &["trust"]);
    run_help(&home, &["trust", "init"]);
    run_help(&home, &["trust", "list"]);
    run_help(&home, &["trust", "status"]);

    run_help(&home, &["rack"]);
    run_help(&home, &["rack", "doctor"]);
    run_help(&home, &["rack", "sync"]);
    run_help(&home, &["rack", "mark-unknown-external"]);
    run_help(&home, &["rack", "license-audit"]);
    run_help(&home, &["rack", "sign"]);
    run_help(&home, &["rack", "prepare-release"]);

    run_help(&home, &["author"]);
    run_help(&home, &["author", "plugin"]);
    run_help(&home, &["author", "plugin", "create"]);
    run_help(&home, &["author", "plugin", "update"]);
    run_help(&home, &["author", "plugin", "remove"]);
    run_help(&home, &["author", "skill"]);
    run_help(&home, &["author", "skill", "create"]);
    run_help(&home, &["author", "skill", "remove"]);
    run_help(&home, &["author", "subagent"]);
    run_help(&home, &["author", "subagent", "create"]);
    run_help(&home, &["author", "subagent", "remove"]);
    run_help(&home, &["author", "hook"]);
    run_help(&home, &["author", "hook", "list"]);
    run_help(&home, &["author", "hook", "create"]);
    run_help(&home, &["author", "hook", "remove"]);
    run_help(&home, &["author", "mcp"]);
    run_help(&home, &["author", "mcp", "create"]);
    run_help(&home, &["author", "mcp", "remove"]);
}

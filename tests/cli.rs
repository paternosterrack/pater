use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use tempfile::TempDir;

fn run(home: &TempDir, args: &[&str]) -> String {
    let mut cmd = cargo_bin_cmd!("pater");
    cmd.env("HOME", home.path());
    let out = cmd
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).unwrap()
}

#[test]
fn cli_core_commands_and_options_success_paths() {
    let home = TempDir::new().unwrap();

    // trust commands
    let out = run(&home, &["trust", "init"]);
    assert!(out.contains("trust initialized"));
    let out = run(&home, &["trust", "list"]);
    assert!(!out.trim().is_empty());
    let out = run(&home, &["--json", "trust", "status"]);
    assert!(out.contains("trusted_key_count"));

    // marketplace commands
    let out = run(
        &home,
        &["--marketplace", "../rack", "marketplace", "add", "../rack"],
    );
    assert!(out.contains("added paternoster-rack"));
    let out = run(&home, &["marketplace", "list"]);
    assert!(out.contains("paternoster-rack"));
    let out = run(&home, &["marketplace", "update"]);
    assert!(out.contains("updated"));

    // discovery/show/validate/hooks
    let out = run(&home, &["--marketplace", "../rack", "validate"]);
    assert!(out.contains("marketplace valid"));
    let out = run(
        &home,
        &["--marketplace", "../rack", "--json", "discover", "commit"],
    );
    assert!(out.contains("commit-commands"));
    let out = run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "show",
            "commit-commands@paternoster-rack",
        ],
    );
    assert!(out.contains("name: commit-commands"));
    let _ = run(&home, &["--marketplace", "../rack", "hooks", "list"]);

    // install options (scope)
    run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "install",
            "commit-commands@paternoster-rack",
            "--scope",
            "user",
        ],
    );
    run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "install",
            "typescript-lsp@paternoster-rack",
            "--scope",
            "project",
        ],
    );
    run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "install",
            "pyright-lsp@paternoster-rack",
            "--scope",
            "local",
        ],
    );

    let out = run(&home, &["installed"]);
    assert!(out.contains("commit-commands"));
    assert!(out.contains("typescript-lsp"));
    assert!(out.contains("pyright-lsp"));

    // update command + option
    let out = run(&home, &["update"]);
    assert!(out.contains("up_to_date") || out.contains("updated"));
    let out = run(
        &home,
        &["update", "commit-commands", "--allow-permission-increase"],
    );
    assert!(out.contains("commit-commands"));

    // adapter commands
    let out = run(&home, &["adapter", "sync", "--target", "all"]);
    assert!(out.contains("adapter sync completed"));
    let out = run(&home, &["--json", "adapter", "smoke", "--target", "all"]);
    assert!(out.contains("adapter"));
    let out = run(&home, &["--json", "adapter", "doctor"]);
    assert!(out.contains("overall"));

    // release check command
    let out = run(
        &home,
        &["--marketplace", "../rack", "--json", "release-check"],
    );
    assert!(out.contains("rack_license_audit"));

    // remove command
    let out = run(&home, &["remove", "commit-commands"]);
    assert!(out.contains("removed"));
}

#[test]
fn validate_marketplace_still_works() {
    let home = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("pater");
    cmd.env("HOME", home.path())
        .arg("--marketplace")
        .arg("../rack")
        .arg("validate")
        .assert()
        .success()
        .stdout(contains("marketplace valid"));
}

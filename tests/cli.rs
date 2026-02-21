use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;
use std::process::Command;
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
        &["--marketplace", "../rack", "remote", "add", "../rack"],
    );
    assert!(out.contains("added paternoster-rack"));
    let out = run(&home, &["remote", "list"]);
    assert!(out.contains("paternoster-rack"));
    let out = run(&home, &["remote", "update"]);
    assert!(out.contains("updated"));

    // discovery/recommend/show/validate/hooks
    let out = run(&home, &["--marketplace", "../rack", "validate"]);
    assert!(out.contains("marketplace valid"));
    let out = run(
        &home,
        &["--marketplace", "../rack", "--json", "search", "commit"],
    );
    assert!(out.contains("commit-commands"));
    let out = run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "--json",
            "recommend",
            "--context",
            "typescript",
        ],
    );
    assert!(out.contains("plugin"));
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
    let _ = run(&home, &["--marketplace", "../rack", "hook", "list"]);

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

    run(
        &home,
        &[
            "--marketplace",
            "../rack",
            "apply",
            "commit-commands@paternoster-rack",
            "--target-adapter",
            "all",
            "--scope",
            "user",
        ],
    );

    let out = run(&home, &["list"]);
    assert!(out.contains("commit-commands"));
    assert!(out.contains("typescript-lsp"));
    assert!(out.contains("pyright-lsp"));

    let out = run(&home, &["--json", "capabilities", "--agent", "all"]);
    assert!(out.contains("installed_count"));

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
    let out = run(&home, &["--marketplace", "../rack", "--json", "check"]);
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

#[test]
fn rack_commands_success_paths() {
    let home = TempDir::new().unwrap();
    let rack = TempDir::new().unwrap();

    fs::create_dir_all(
        rack.path()
            .join("_upstreams/claude-plugins-official/.claude-plugin"),
    )
    .unwrap();
    fs::create_dir_all(rack.path().join("_upstreams/claude-code/.claude-plugin")).unwrap();
    fs::create_dir_all(rack.path().join("_upstreams/skills/.claude-plugin")).unwrap();

    let upstream = r#"{
  "plugins": [
    {"name": "demo", "source": "./plugins/demo"}
  ]
}"#;
    fs::write(
        rack.path()
            .join("_upstreams/claude-plugins-official/.claude-plugin/marketplace.json"),
        upstream,
    )
    .unwrap();
    fs::write(
        rack.path()
            .join("_upstreams/claude-code/.claude-plugin/marketplace.json"),
        "{\"plugins\":[]}",
    )
    .unwrap();
    fs::write(
        rack.path()
            .join("_upstreams/skills/.claude-plugin/marketplace.json"),
        "{\"plugins\":[]}",
    )
    .unwrap();

    fs::create_dir_all(rack.path().join("plugins/demo/.claude-plugin")).unwrap();
    fs::write(
        rack.path().join("plugins/demo/LICENSE"),
        "MIT License\nPermission is hereby granted",
    )
    .unwrap();
    fs::write(
        rack.path().join("plugins/demo/.claude-plugin/plugin.json"),
        "{\"name\":\"demo\",\"version\":\"1.0.0\"}",
    )
    .unwrap();

    let key = rack.path().join("test-key.pem");
    let status = Command::new("openssl")
        .args([
            "genpkey",
            "-algorithm",
            "Ed25519",
            "-out",
            key.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    // rack doctor + sync
    let out = run(
        &home,
        &[
            "rack",
            "doctor",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--sign-key",
            key.to_str().unwrap(),
        ],
    );
    assert!(out.contains("rack doctor"));

    let out = run(
        &home,
        &["rack", "sync", "--rack-dir", rack.path().to_str().unwrap()],
    );
    assert!(out.contains("synced"));

    // rack license audit success
    let out = run(
        &home,
        &[
            "rack",
            "license-audit",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );
    assert!(out.contains("license audit"));

    // mark unknown external (should still succeed)
    let _ = run(
        &home,
        &[
            "rack",
            "mark-unknown-external",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );

    // sign + prepare-release
    let out = run(
        &home,
        &[
            "rack",
            "sign",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--sign-key",
            key.to_str().unwrap(),
        ],
    );
    assert!(out.contains("signed"));

    let out = run(
        &home,
        &[
            "rack",
            "prepare-release",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--sign-key",
            key.to_str().unwrap(),
        ],
    );
    assert!(out.contains("rack release prepared"));
}

#[test]
fn authoring_commands_success_paths() {
    let home = TempDir::new().unwrap();
    let rack = TempDir::new().unwrap();

    fs::create_dir_all(rack.path().join(".pater")).unwrap();
    fs::write(
        rack.path().join(".pater/marketplace.json"),
        r#"{"name":"paternoster-rack","plugins":[]}"#,
    )
    .unwrap();

    // plugin create/update/remove
    let out = run(
        &home,
        &[
            "author",
            "plugin",
            "create",
            "demo",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--description",
            "Demo plugin",
        ],
    );
    assert!(out.contains("created"));

    let out = run(
        &home,
        &[
            "author",
            "plugin",
            "update",
            "demo",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--version",
            "0.2.0",
        ],
    );
    assert!(out.contains("updated"));

    // skill create/remove
    let out = run(
        &home,
        &[
            "author",
            "skill",
            "create",
            "demo",
            "review",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--description",
            "Review code",
        ],
    );
    assert!(out.contains("created"));

    let out = run(
        &home,
        &[
            "author",
            "skill",
            "remove",
            "demo",
            "review",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );
    assert!(out.contains("removed"));

    // subagent create/remove
    let out = run(
        &home,
        &[
            "author",
            "subagent",
            "create",
            "demo",
            "planner",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--purpose",
            "Plan tasks",
        ],
    );
    assert!(out.contains("created"));

    let out = run(
        &home,
        &[
            "author",
            "subagent",
            "remove",
            "demo",
            "planner",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );
    assert!(out.contains("removed"));

    // hook create/remove
    let out = run(
        &home,
        &[
            "author",
            "hook",
            "create",
            "demo",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--agent",
            "codex",
            "--event",
            "pre-commit",
            "--run",
            "cargo test",
        ],
    );
    assert!(out.contains("created"));

    let out = run(
        &home,
        &[
            "author",
            "hook",
            "remove",
            "demo",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--agent",
            "codex",
            "--event",
            "pre-commit",
        ],
    );
    assert!(out.contains("removed"));

    // mcp create/remove
    let out = run(
        &home,
        &[
            "author",
            "mcp",
            "create",
            "demo",
            "github",
            "--rack-dir",
            rack.path().to_str().unwrap(),
            "--command",
            "mcp-github",
        ],
    );
    assert!(out.contains("added"));

    let out = run(
        &home,
        &[
            "author",
            "mcp",
            "remove",
            "demo",
            "github",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );
    assert!(out.contains("removed"));

    let out = run(
        &home,
        &[
            "author",
            "plugin",
            "remove",
            "demo",
            "--rack-dir",
            rack.path().to_str().unwrap(),
        ],
    );
    assert!(out.contains("removed"));
}

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestEnv {
    _tmp: TempDir,
    pub home: PathBuf,
    pub rack: PathBuf,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
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

    pub fn cmd(&self) -> Command {
        let mut cmd = cargo_bin_cmd!("pater");
        cmd.env("HOME", &self.home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("RUSTUP_HOME", &self.rustup_home);
        cmd
    }

    pub fn run_json(&self, args: &[&str]) -> Value {
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

    pub fn run_json_market(&self, args: &[&str]) -> Value {
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
    fs::create_dir_all(commit.join("skills/commit-guidelines")).expect("create skill dir");

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
            "license": "MIT",
            "mcps": [
                {"name": "github", "command": "mcp-github"}
            ]
        })
        .to_string(),
    )
    .expect("write plugin manifest");
    fs::write(
        commit.join("skills/commit-guidelines/SKILL.md"),
        "# Commit Guidelines\n",
    )
    .expect("write skill file");

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
                "skills": ["commit-guidelines"],
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

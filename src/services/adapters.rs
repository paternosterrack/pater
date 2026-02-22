use crate::cli::AdapterTarget;
use crate::domain::models::{CheckItem, DoctorReport, SmokeReport, State};
use crate::rack;
use std::collections::HashSet;
use std::path::PathBuf;

fn check_exists(name: &str, path: PathBuf) -> CheckItem {
    CheckItem {
        name: name.to_string(),
        status: if path.exists() { "ok" } else { "missing" }.to_string(),
    }
}

fn adapter_base(target: &AdapterTarget) -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let p = match target {
        AdapterTarget::Claude => PathBuf::from(home).join(".claude/plugins"),
        AdapterTarget::Codex => PathBuf::from(home).join(".codex/plugins"),
        AdapterTarget::Openclaw => PathBuf::from(home).join(".openclaw/workspace/skills"),
        AdapterTarget::All => PathBuf::from(home).join(".pater/adapters"),
    };
    Ok(p)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    std::fs::write(dst.join(".pater-managed"), "managed-by=pater\n")?;
    Ok(())
}

fn sync_target(state: &State, target: AdapterTarget) -> anyhow::Result<()> {
    let base = adapter_base(&target)?;
    std::fs::create_dir_all(&base)?;
    let mut installed_dirs = Vec::new();
    let desired: HashSet<String> = state.installed.iter().map(|p| p.name.clone()).collect();

    for p in &state.installed {
        let mut src = PathBuf::from(&p.local_path);
        if !src.exists() {
            if let Ok(r) = rack::resolve_plugin_path(&p.marketplace_source, &p.source) {
                src = r;
            }
        }
        if !src.exists() {
            continue;
        }
        let dst = base.join(&p.name);
        copy_dir_all(&src, &dst)?;
        installed_dirs.push(dst.to_string_lossy().to_string());
    }

    for entry in std::fs::read_dir(&base)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let managed = path.join(".pater-managed").exists();
        if managed && !desired.contains(&name) {
            std::fs::remove_dir_all(path)?;
        }
    }

    write_activation_shim(&target, &installed_dirs)?;
    Ok(())
}

pub fn sync_installed(state: &State, target: AdapterTarget) -> anyhow::Result<()> {
    match target {
        AdapterTarget::All => {
            sync_target(state, AdapterTarget::Claude)?;
            sync_target(state, AdapterTarget::Codex)?;
            sync_target(state, AdapterTarget::Openclaw)?;
        }
        t => sync_target(state, t)?,
    }
    Ok(())
}

pub fn adapter_smoke(state: &State, target: AdapterTarget) -> anyhow::Result<Vec<SmokeReport>> {
    let targets = match target {
        AdapterTarget::All => vec![
            AdapterTarget::Claude,
            AdapterTarget::Codex,
            AdapterTarget::Openclaw,
        ],
        t => vec![t],
    };

    let mut out = Vec::new();
    let home = std::env::var("HOME").ok();
    for t in targets {
        let base = adapter_base(&t)?;
        let mut missing = Vec::new();
        for p in &state.installed {
            if !base.join(&p.name).exists() {
                missing.push(p.name.clone());
            }
        }

        let shim_ok = if let Some(home) = &home {
            match t {
                AdapterTarget::Claude => PathBuf::from(home).join(".claude/pater.plugins.json"),
                AdapterTarget::Codex => PathBuf::from(home).join(".codex/pater.plugins.json"),
                AdapterTarget::Openclaw => {
                    PathBuf::from(home).join(".openclaw/workspace/skills/.pater-index.json")
                }
                AdapterTarget::All => PathBuf::new(),
            }
            .exists()
                || matches!(t, AdapterTarget::All)
        } else {
            false
        };

        let status = if missing.is_empty() && shim_ok {
            "ok".to_string()
        } else if !shim_ok {
            "missing_shim".to_string()
        } else {
            "missing_plugins".to_string()
        };

        out.push(SmokeReport {
            adapter: format!("{:?}", t).to_lowercase(),
            status,
            checked_plugins: state.installed.len(),
            missing_plugins: missing,
        });
    }
    Ok(out)
}

pub fn adapter_doctor(state: &State) -> anyhow::Result<DoctorReport> {
    let home = std::env::var("HOME")?;
    let smoke = adapter_smoke(state, AdapterTarget::All)?;

    let configs = vec![
        check_exists(
            "claude_settings",
            PathBuf::from(&home).join(".claude/settings.json"),
        ),
        check_exists(
            "codex_config",
            PathBuf::from(&home).join(".codex/config.toml"),
        ),
        check_exists(
            "openclaw_index",
            PathBuf::from(&home).join(".openclaw/workspace/skills/.pater-index.json"),
        ),
    ];

    let wrappers = vec![
        check_exists(
            "pater-claude",
            PathBuf::from(&home).join(".local/bin/pater-claude"),
        ),
        check_exists(
            "pater-codex",
            PathBuf::from(&home).join(".local/bin/pater-codex"),
        ),
        check_exists(
            "pater-openclaw",
            PathBuf::from(&home).join(".local/bin/pater-openclaw"),
        ),
    ];

    let path_has_local_bin = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|p| p == PathBuf::from(&home).join(".local/bin").to_string_lossy());

    let all_ok = smoke.iter().all(|s| s.status == "ok")
        && configs.iter().all(|c| c.status == "ok")
        && wrappers.iter().all(|w| w.status == "ok");

    Ok(DoctorReport {
        overall: if all_ok { "ok" } else { "needs_attention" }.to_string(),
        path_has_local_bin,
        smoke,
        configs,
        wrappers,
    })
}

fn write_activation_shim(target: &AdapterTarget, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    match target {
        AdapterTarget::Claude => {
            let root = PathBuf::from(&home).join(".claude");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "claude",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Load these plugin dirs in Claude Code startup config."
            });
            std::fs::write(
                root.join("pater.plugins.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            patch_claude_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-claude", "claude", plugin_dirs)?;
        }
        AdapterTarget::Codex => {
            let root = PathBuf::from(&home).join(".codex");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "codex",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Use these plugin dirs in Codex startup config."
            });
            std::fs::write(
                root.join("pater.plugins.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            patch_codex_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-codex", "codex", plugin_dirs)?;
        }
        AdapterTarget::Openclaw => {
            let root = PathBuf::from(&home)
                .join(".openclaw")
                .join("workspace")
                .join("skills");
            std::fs::create_dir_all(&root)?;
            let data = serde_json::json!({
                "managedBy": "pater",
                "adapter": "openclaw",
                "plugin_dirs": plugin_dirs,
                "note": "Generated by pater. Skills are materialized under this directory."
            });
            std::fs::write(
                root.join(".pater-index.json"),
                serde_json::to_string_pretty(&data)?,
            )?;
            write_wrapper(&home, "pater-openclaw", "openclaw", plugin_dirs)?;
        }
        AdapterTarget::All => {}
    }
    Ok(())
}

fn patch_claude_config(root: &std::path::Path, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let cfg = root.join("settings.json");
    let mut v = if cfg.exists() {
        serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&cfg)?)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    v["pater"] = serde_json::json!({ "plugin_dirs": plugin_dirs });
    std::fs::write(cfg, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

fn patch_codex_config(root: &std::path::Path, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let cfg = root.join("config.toml");
    let mut content = if cfg.exists() {
        std::fs::read_to_string(&cfg)?
    } else {
        String::new()
    };
    let start = "# >>> pater managed start >>>";
    let end = "# <<< pater managed end <<<";
    if let (Some(s), Some(e)) = (content.find(start), content.find(end)) {
        content.replace_range(s..(e + end.len()), "");
    }
    let dirs = plugin_dirs
        .iter()
        .map(|d| format!("\"{}\"", d.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    content.push_str(&format!(
        "\n{start}\n[pater]\nplugin_dirs = [{dirs}]\n{end}\n"
    ));
    std::fs::write(cfg, content)?;
    Ok(())
}

fn write_wrapper(home: &str, name: &str, cmd: &str, plugin_dirs: &[String]) -> anyhow::Result<()> {
    let bin = PathBuf::from(home).join(".local/bin");
    std::fs::create_dir_all(&bin)?;
    let mut args = String::new();
    for d in plugin_dirs {
        args.push_str(&format!(" --plugin-dir '{}'", d.replace('\'', "'\\''")));
    }
    let script = format!("#!/usr/bin/env sh\nexec {cmd}{args} \"$@\"\n");
    let path = bin.join(name);
    std::fs::write(&path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

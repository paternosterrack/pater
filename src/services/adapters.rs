use crate::cli::AdapterTarget;
use crate::domain::models::{CheckItem, DoctorReport, SmokeReport, State};
use crate::rack;
use crate::services::storage::{
    materialize_plugin, runtime_base_dir, runtime_bridges_dir, runtime_plugins_dir,
    runtime_registry_path,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn check_exists(name: &str, path: PathBuf) -> CheckItem {
    CheckItem {
        name: name.to_string(),
        status: if path.exists() { "ok" } else { "missing" }.to_string(),
    }
}

fn adapter_shim_path(home: &str, target: &AdapterTarget) -> PathBuf {
    match target {
        AdapterTarget::Claude => PathBuf::from(home).join(".claude/pater.plugins.json"),
        AdapterTarget::Codex => PathBuf::from(home).join(".codex/pater.plugins.json"),
        AdapterTarget::Openclaw => {
            PathBuf::from(home).join(".openclaw/workspace/skills/.pater-index.json")
        }
        AdapterTarget::All => PathBuf::new(),
    }
}

fn bridge_file_path(target: &AdapterTarget) -> anyhow::Result<PathBuf> {
    Ok(runtime_bridges_dir()?.join(format!("{:?}.json", target).to_ascii_lowercase()))
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
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

fn ensure_runtime_materialized(
    installed: &crate::domain::models::InstalledPlugin,
) -> Option<PathBuf> {
    let runtime_dir = runtime_plugins_dir().ok()?.join(&installed.name);
    if runtime_dir.exists() {
        return Some(runtime_dir);
    }

    let local_path = PathBuf::from(&installed.local_path);
    if local_path.exists() && local_path.is_dir() && copy_dir_all(&local_path, &runtime_dir).is_ok()
    {
        return Some(runtime_dir);
    }

    if let Ok(src) = rack::resolve_plugin_path(&installed.marketplace_source, &installed.source) {
        if let Ok(dst) = materialize_plugin(&installed.name, &src) {
            return Some(dst);
        }
    }

    None
}

fn load_manifest_mcps(runtime_dir: &Path) -> Vec<serde_json::Value> {
    let manifest = runtime_dir.join(".claude-plugin").join("plugin.json");
    if !manifest.exists() {
        return Vec::new();
    }

    let Ok(raw) = std::fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };

    value
        .get("mcps")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn build_runtime_registry(
    state: &State,
) -> anyhow::Result<(Vec<String>, serde_json::Value, PathBuf)> {
    let base = runtime_base_dir()?;
    std::fs::create_dir_all(&base)?;

    let mut plugin_dirs = Vec::new();
    let mut plugins = Vec::new();
    let mut skills = Vec::new();
    let mut hooks = Vec::new();
    let mut subagents = Vec::new();
    let mut mcps = Vec::new();
    let mut markets_cache: HashMap<String, rack::Marketplace> = HashMap::new();

    for installed in &state.installed {
        let Some(runtime_dir) = ensure_runtime_materialized(installed) else {
            continue;
        };

        let runtime_path = runtime_dir.to_string_lossy().to_string();
        plugin_dirs.push(runtime_path.clone());

        let market = if let Some(cached) = markets_cache.get(&installed.marketplace_source) {
            Some(cached.clone())
        } else if let Ok(loaded) = rack::load_marketplace(&installed.marketplace_source) {
            markets_cache.insert(installed.marketplace_source.clone(), loaded.clone());
            Some(loaded)
        } else {
            None
        };

        let market_plugin = market
            .as_ref()
            .and_then(|m| m.plugins.iter().find(|p| p.name == installed.name));

        let plugin_skills = market_plugin.map(|p| p.skills.clone()).unwrap_or_default();

        let plugin_hooks = market_plugin.map(|p| p.hooks.clone()).unwrap_or_default();

        let plugin_subagents = market_plugin
            .map(|p| p.subagents.clone())
            .unwrap_or_default();

        let plugin_mcps = load_manifest_mcps(&runtime_dir);

        plugins.push(serde_json::json!({
            "name": installed.name,
            "marketplace": installed.marketplace,
            "marketplace_source": installed.marketplace_source,
            "source": installed.source,
            "runtime_path": runtime_path,
            "permissions": installed.permissions,
            "version": installed.version,
            "scope": installed.scope,
        }));

        for skill in &plugin_skills {
            skills.push(serde_json::json!({
                "plugin": installed.name,
                "name": skill,
                "path": runtime_dir.join("skills").join(skill).to_string_lossy(),
            }));
        }

        for hook in plugin_hooks {
            hooks.push(serde_json::json!({
                "plugin": installed.name,
                "agent": hook.agent,
                "event": hook.event,
                "run": hook.run,
            }));
        }

        for subagent in plugin_subagents {
            subagents.push(serde_json::json!({
                "plugin": installed.name,
                "name": subagent.name,
                "purpose": subagent.purpose,
            }));
        }

        for mcp in plugin_mcps {
            mcps.push(serde_json::json!({
                "plugin": installed.name,
                "config": mcp,
            }));
        }
    }

    plugin_dirs.sort();
    plugins.sort_by(|a, b| {
        let an = a.get("name").and_then(|x| x.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|x| x.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    let registry = serde_json::json!({
        "managedBy": "pater",
        "schemaVersion": 1,
        "runtime_base": base,
        "plugins": plugins,
        "skills": skills,
        "hooks": hooks,
        "subagents": subagents,
        "mcps": mcps,
    });

    let registry_path = runtime_registry_path()?;
    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&registry_path, serde_json::to_string_pretty(&registry)?)?;

    Ok((plugin_dirs, registry, registry_path))
}

fn write_bridge(
    target: &AdapterTarget,
    plugin_dirs: &[String],
    registry_path: &Path,
) -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    let shim_path = adapter_shim_path(&home, target);

    let bridge_data = serde_json::json!({
        "managedBy": "pater",
        "adapter": format!("{:?}", target).to_ascii_lowercase(),
        "runtime_registry": registry_path,
        "plugin_dirs": plugin_dirs,
        "note": "Runtime-first bridge generated by pater. Agent config should read runtime paths.",
    });

    if !matches!(target, AdapterTarget::All) {
        if let Some(parent) = shim_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&shim_path, serde_json::to_string_pretty(&bridge_data)?)?;
    }

    let bridge_path = bridge_file_path(target)?;
    if let Some(parent) = bridge_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(bridge_path, serde_json::to_string_pretty(&bridge_data)?)?;

    match target {
        AdapterTarget::Claude => {
            let root = PathBuf::from(&home).join(".claude");
            patch_claude_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-claude", "claude", plugin_dirs)?;
        }
        AdapterTarget::Codex => {
            let root = PathBuf::from(&home).join(".codex");
            patch_codex_config(&root, plugin_dirs)?;
            write_wrapper(&home, "pater-codex", "codex", plugin_dirs)?;
        }
        AdapterTarget::Openclaw => {
            write_wrapper(&home, "pater-openclaw", "openclaw", plugin_dirs)?;
        }
        AdapterTarget::All => {}
    }

    Ok(())
}

fn sync_target(
    target: AdapterTarget,
    plugin_dirs: &[String],
    registry_path: &Path,
) -> anyhow::Result<()> {
    write_bridge(&target, plugin_dirs, registry_path)
}

pub fn sync_installed(state: &State, target: AdapterTarget) -> anyhow::Result<()> {
    let (plugin_dirs, _registry, registry_path) = build_runtime_registry(state)?;

    match target {
        AdapterTarget::All => {
            sync_target(AdapterTarget::Claude, &plugin_dirs, &registry_path)?;
            sync_target(AdapterTarget::Codex, &plugin_dirs, &registry_path)?;
            sync_target(AdapterTarget::Openclaw, &plugin_dirs, &registry_path)?;
        }
        t => sync_target(t, &plugin_dirs, &registry_path)?,
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
    let runtime_plugins = runtime_plugins_dir()?;

    for t in targets {
        let mut missing = Vec::new();
        for p in &state.installed {
            if !runtime_plugins.join(&p.name).exists() {
                missing.push(p.name.clone());
            }
        }

        let shim_ok = if let Some(home) = &home {
            adapter_shim_path(home, &t).exists()
        } else {
            false
        };

        let bridge_ok = bridge_file_path(&t).map(|p| p.exists()).unwrap_or(false);

        let status = if missing.is_empty() && shim_ok && bridge_ok {
            "ok".to_string()
        } else if !shim_ok {
            "missing_shim".to_string()
        } else if !bridge_ok {
            "missing_bridge".to_string()
        } else {
            "missing_runtime_plugins".to_string()
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
        check_exists("runtime_registry", runtime_registry_path()?),
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

fn patch_claude_config(root: &std::path::Path, plugin_dirs: &[String]) -> anyhow::Result<()> {
    std::fs::create_dir_all(root)?;
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
    std::fs::create_dir_all(root)?;
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

use crate::domain::models::{InstalledPlugin, Lockfile, State};
use std::path::PathBuf;

pub fn audit(action: &str, data: serde_json::Value) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let path = PathBuf::from(home).join(".config/pater/audit.jsonl");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let event = serde_json::json!({
        "ts": chrono_like_now(),
        "action": action,
        "data": data
    });
    let line = format!("{}\n", event);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    ts.to_string()
}

pub fn upsert_installed(state: &mut State, entry: InstalledPlugin) {
    if let Some(existing) = state
        .installed
        .iter_mut()
        .find(|i| i.name == entry.name && i.marketplace == entry.marketplace)
    {
        *existing = entry;
    } else {
        state.installed.push(entry);
    }
}

pub fn runtime_base_dir() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("pater")
        .join("runtime"))
}

pub fn runtime_plugins_dir() -> anyhow::Result<PathBuf> {
    Ok(runtime_base_dir()?.join("plugins"))
}

pub fn runtime_registry_path() -> anyhow::Result<PathBuf> {
    Ok(runtime_base_dir()?.join("registry.json"))
}

pub fn runtime_bridges_dir() -> anyhow::Result<PathBuf> {
    Ok(runtime_base_dir()?.join("bridges"))
}

pub fn materialize_plugin(name: &str, source_path: &std::path::Path) -> anyhow::Result<PathBuf> {
    let base = runtime_plugins_dir()?;
    std::fs::create_dir_all(&base)?;
    let dst = base.join(name);
    copy_dir_all(source_path, &dst)?;
    Ok(dst)
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

fn state_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/state.json"))
}

fn lockfile_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/pater.lock"))
}

pub fn load_state() -> anyhow::Result<State> {
    let p = state_path()?;
    if !p.exists() {
        return Ok(State::default());
    }
    let raw = std::fs::read_to_string(p)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_state(s: &State) -> anyhow::Result<()> {
    let p = state_path()?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(s)?)?;
    Ok(())
}

pub fn save_lockfile(state: &State) -> anyhow::Result<()> {
    let lock = Lockfile {
        version: 1,
        plugins: state.installed.clone(),
    };
    let p = lockfile_path()?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(&lock)?)?;
    Ok(())
}

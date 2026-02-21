use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Marketplace {
    pub name: String,
    pub owner: Owner,
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Owner {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub source: String,
    pub description: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<Hook>,
    #[serde(default)]
    pub subagents: Vec<Subagent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hook {
    pub agent: String,
    pub event: String,
    pub run: String,
    #[serde(skip)]
    pub plugin_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subagent {
    pub name: String,
    pub purpose: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RackError {
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("duplicate plugin name: {0}")]
    DuplicatePlugin(String),
}

fn looks_like_github_shorthand(source: &str) -> bool {
    source.split('/').count() == 2 && !source.contains("://") && !source.starts_with('.')
}

fn normalize_source(source: &str) -> String {
    if looks_like_github_shorthand(source) {
        format!(
            "https://raw.githubusercontent.com/{}/main/.pater/marketplace.json",
            source
        )
    } else {
        source.to_string()
    }
}

fn normalize_sig_source(source: &str) -> String {
    if looks_like_github_shorthand(source) {
        format!(
            "https://raw.githubusercontent.com/{}/main/.pater/marketplace.sig",
            source
        )
    } else if source.ends_with("marketplace.json") {
        source.replace("marketplace.json", "marketplace.sig")
    } else if source.starts_with("http://") || source.starts_with("https://") {
        format!("{}/.pater/marketplace.sig", source.trim_end_matches('/'))
    } else {
        source.to_string()
    }
}

fn is_remote(source: &str) -> bool {
    source.starts_with("http://")
        || source.starts_with("https://")
        || looks_like_github_shorthand(source)
}

pub fn resolve_marketplace_file(source: &str) -> PathBuf {
    let p = Path::new(source);
    if p.is_dir() {
        p.join(".pater").join("marketplace.json")
    } else {
        p.to_path_buf()
    }
}

fn cache_path(source: &str) -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let id = hex::encode(hasher.finalize());
    Ok(PathBuf::from(home)
        .join(".cache")
        .join("pater")
        .join("marketplaces")
        .join(format!("{}.json", id)))
}

fn fetch_marketplace_text(source: &str, timeout_ms: u64) -> anyhow::Result<String> {
    let url = normalize_source(source);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()?;
    let resp = client.get(url).send()?.error_for_status()?;
    Ok(resp.text()?)
}

fn fetch_signature_text(source: &str, timeout_ms: u64) -> anyhow::Result<String> {
    let url = normalize_sig_source(source);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()?;
    let resp = client.get(url).send()?.error_for_status()?;
    Ok(resp.text()?)
}

pub fn refresh_marketplace(source: &str) -> anyhow::Result<()> {
    if !is_remote(source) {
        return Ok(());
    }
    let body = fetch_marketplace_text(source, 3000)?;
    let cache = cache_path(source)?;
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(cache, body)?;
    Ok(())
}

pub fn load_marketplace_raw(source: &str) -> anyhow::Result<String> {
    if is_remote(source) {
        let cache = cache_path(source)?;
        match fetch_marketplace_text(source, 2500) {
            Ok(body) => {
                if let Some(parent) = cache.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&cache, &body)?;
                return Ok(body);
            }
            Err(_) if cache.exists() => {
                let raw = std::fs::read_to_string(cache)?;
                return Ok(raw);
            }
            Err(e) => return Err(e),
        }
    }

    let file = resolve_marketplace_file(source);
    Ok(std::fs::read_to_string(file)?)
}

pub fn load_marketplace_signature(source: &str) -> anyhow::Result<String> {
    if is_remote(source) {
        return fetch_signature_text(source, 2500);
    }

    let p = Path::new(source);
    let sig_path = if p.is_dir() {
        p.join(".pater").join("marketplace.sig")
    } else {
        p.parent().unwrap_or(Path::new(".")).join("marketplace.sig")
    };
    Ok(std::fs::read_to_string(sig_path)?)
}

pub fn load_marketplace(source: &str) -> anyhow::Result<Marketplace> {
    let raw = load_marketplace_raw(source)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn discover<'a>(m: &'a Marketplace, query: Option<&str>) -> Vec<&'a Plugin> {
    match query {
        None => m.plugins.iter().collect(),
        Some(q) => {
            let q = q.to_ascii_lowercase();
            m.plugins
                .iter()
                .filter(|p| {
                    p.name.to_ascii_lowercase().contains(&q)
                        || p.description
                            .as_ref()
                            .map(|d| d.to_ascii_lowercase().contains(&q))
                            .unwrap_or(false)
                })
                .collect()
        }
    }
}

pub fn show<'a>(m: &'a Marketplace, id: &str) -> anyhow::Result<&'a Plugin> {
    m.plugins
        .iter()
        .find(|p| p.name == id)
        .ok_or_else(|| RackError::PluginNotFound(id.to_string()).into())
}

pub fn list_hooks(m: &Marketplace, agent: Option<&str>) -> Vec<Hook> {
    let mut out = Vec::new();
    for p in &m.plugins {
        for h in &p.hooks {
            if agent.map(|a| a == h.agent).unwrap_or(true) {
                let mut h2 = h.clone();
                h2.plugin_name = p.name.clone();
                out.push(h2);
            }
        }
    }
    out
}

pub fn validate(m: &Marketplace) -> anyhow::Result<()> {
    let mut seen = HashSet::new();
    for p in &m.plugins {
        if !seen.insert(&p.name) {
            return Err(RackError::DuplicatePlugin(p.name.clone()).into());
        }
    }
    Ok(())
}

fn repo_cache_path(source: &str) -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let id = hex::encode(hasher.finalize());
    Ok(PathBuf::from(home)
        .join(".cache")
        .join("pater")
        .join("repos")
        .join(id))
}

fn ensure_repo(source: &str) -> anyhow::Result<PathBuf> {
    let cache = repo_cache_path(source)?;
    if cache.exists() {
        let _ = Command::new("git")
            .args(["-C", cache.to_string_lossy().as_ref(), "pull", "--ff-only"])
            .output();
        return Ok(cache);
    }

    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let repo_url = if looks_like_github_shorthand(source) {
        format!("https://github.com/{}.git", source)
    } else {
        source.to_string()
    };

    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &repo_url,
            cache.to_string_lossy().as_ref(),
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to clone marketplace repo: {}", repo_url);
    }
    Ok(cache)
}

pub fn resolve_plugin_path(market_source: &str, plugin_source: &str) -> anyhow::Result<PathBuf> {
    if plugin_source.starts_with("./") {
        let mpath = Path::new(market_source);
        if mpath.exists() {
            let base = if mpath.is_dir() {
                mpath.to_path_buf()
            } else {
                mpath.parent().unwrap_or(Path::new(".")).to_path_buf()
            };
            return Ok(base.join(plugin_source.trim_start_matches("./")));
        }

        if is_remote(market_source) {
            let repo = ensure_repo(market_source)?;
            return Ok(repo.join(plugin_source.trim_start_matches("./")));
        }
    }

    if plugin_source.starts_with("http://")
        || plugin_source.starts_with("https://")
        || plugin_source.starts_with("git@")
        || looks_like_github_shorthand(plugin_source)
    {
        return ensure_repo(plugin_source);
    }

    Ok(PathBuf::from(plugin_source))
}

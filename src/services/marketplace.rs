use crate::cli::{AdapterTarget, DEFAULT_MARKETPLACE_SOURCE};
use crate::domain::models::{
    DiscoverItem, MarketRef, PolicyEvalReport, PolicyFile, PolicyGeneral, Recommendation, State,
    UpdateReport,
};
use crate::rack;
use crate::services::policy::source_matches_allowed;
use crate::services::rack_ops::rack_license_audit_readonly;
use crate::services::storage::{materialize_plugin, save_state};
use crate::services::trust::verify_marketplace_signature;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn load_policy() -> anyhow::Result<PolicyFile> {
    let home = std::env::var("HOME")?;
    let path = PathBuf::from(home).join(".config/pater/policy.toml");
    if !path.exists() {
        return Ok(PolicyFile {
            general: PolicyGeneral::default(),
        });
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}

pub fn update_plugins(
    state: &mut State,
    markets: &[MarketRef],
    only: Option<&str>,
    allow_permission_increase: bool,
    policy: &PolicyFile,
) -> anyhow::Result<Vec<UpdateReport>> {
    let mut reports = Vec::new();
    for installed in &mut state.installed {
        if only.map(|o| o != installed.name).unwrap_or(false) {
            continue;
        }
        let latest = show_plugin(
            markets,
            &installed.name,
            Some(&installed.marketplace),
            policy,
        )?;
        enforce_policy_for_plugin(policy, &latest)?;
        let old_permissions: HashSet<_> = installed.permissions.iter().cloned().collect();
        let new_permissions: HashSet<_> = latest.permissions.iter().cloned().collect();
        let added_permissions: Vec<String> = new_permissions
            .difference(&old_permissions)
            .cloned()
            .collect();

        if !added_permissions.is_empty() && !allow_permission_increase {
            reports.push(UpdateReport {
                name: installed.name.clone(),
                status: "blocked_permission_increase".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions,
            });
            continue;
        }

        let changed =
            installed.version != latest.version || installed.permissions != latest.permissions;
        if changed {
            let report = UpdateReport {
                name: installed.name.clone(),
                status: "updated".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions,
            };
            installed.version = latest.version.clone();
            installed.permissions = latest.permissions.clone();
            installed.source = latest.source.clone();
            installed.marketplace_source = latest.marketplace_source.clone();
            if let Ok(src) = rack::resolve_plugin_path(&latest.marketplace_source, &latest.source) {
                if let Ok(p) = materialize_plugin(&installed.name, &src) {
                    installed.local_path = p.to_string_lossy().to_string();
                }
            }
            reports.push(report);
        } else {
            reports.push(UpdateReport {
                name: installed.name.clone(),
                status: "up_to_date".to_string(),
                old_version: installed.version.clone(),
                new_version: latest.version.clone(),
                added_permissions: vec![],
            });
        }
    }
    Ok(reports)
}

pub fn ensure_default_marketplace(state: &mut State) -> anyhow::Result<()> {
    if !state
        .marketplaces
        .iter()
        .any(|m| m.source == DEFAULT_MARKETPLACE_SOURCE)
    {
        state.marketplaces.push(MarketRef {
            name: "paternoster-rack".to_string(),
            source: DEFAULT_MARKETPLACE_SOURCE.to_string(),
        });
        save_state(state)?;
    }
    Ok(())
}

pub fn dedupe_markets(markets: &mut Vec<MarketRef>) {
    let mut seen = HashSet::new();
    markets.retain(|m| seen.insert(format!("{}::{}", m.name, m.source)));
}

pub fn discover_across(
    markets: &[MarketRef],
    query: Option<&str>,
    policy: &PolicyFile,
) -> anyhow::Result<Vec<DiscoverItem>> {
    let mut out = Vec::new();
    for m in markets {
        let Ok(loaded) = checked_load_marketplace(&m.source, policy) else {
            continue;
        };
        for p in rack::discover(&loaded, query) {
            out.push(DiscoverItem {
                marketplace: loaded.name.clone(),
                marketplace_source: m.source.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                distribution: p.distribution.clone(),
                license_status: p.license_status.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    Ok(out)
}

pub fn recommend_plugins(items: Vec<DiscoverItem>, context: Option<&str>) -> Vec<Recommendation> {
    let ctx = context.unwrap_or("").to_ascii_lowercase();
    let mut out: Vec<Recommendation> = items
        .into_iter()
        .map(|p| {
            let mut score = 0;
            let mut reason = String::from("baseline relevance");
            if !ctx.is_empty() {
                if p.name.to_ascii_lowercase().contains(&ctx) {
                    score += 50;
                    reason = "name matches context".to_string();
                }
                if p.description.to_ascii_lowercase().contains(&ctx) {
                    score += 30;
                    reason = "description matches context".to_string();
                }
                for perm in &p.permissions {
                    if ctx.contains(&perm.to_ascii_lowercase()) {
                        score += 10;
                    }
                }
            }
            if p.distribution.as_deref() == Some("external-reference-only") {
                score -= 20;
            }
            Recommendation {
                plugin: p.name,
                marketplace: p.marketplace,
                score,
                reason,
                permission_count: p.permissions.len(),
                distribution: p.distribution,
                license_status: p.license_status,
            }
        })
        .collect();
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.plugin.cmp(&b.plugin)));
    out.truncate(10);
    out
}

pub fn show_plugin(
    markets: &[MarketRef],
    name: &str,
    marketplace: Option<&str>,
    policy: &PolicyFile,
) -> anyhow::Result<DiscoverItem> {
    for m in markets {
        let Ok(loaded) = checked_load_marketplace(&m.source, policy) else {
            continue;
        };
        if let Some(filter) = marketplace {
            if loaded.name != filter {
                continue;
            }
        }
        if let Ok(p) = rack::show(&loaded, name) {
            return Ok(DiscoverItem {
                marketplace: loaded.name.clone(),
                marketplace_source: m.source.clone(),
                name: p.name.clone(),
                description: p.description.clone().unwrap_or_default(),
                version: p.version.clone(),
                source: p.source.clone(),
                distribution: p.distribution.clone(),
                license_status: p.license_status.clone(),
                permissions: p.permissions.clone(),
            });
        }
    }
    anyhow::bail!("plugin not found: {}", name)
}

pub fn checked_load_marketplace(
    source: &str,
    policy: &PolicyFile,
) -> anyhow::Result<rack::Marketplace> {
    if policy.general.require_signed_marketplace {
        let ok = verify_marketplace_signature(source)?;
        if !ok {
            anyhow::bail!("marketplace signature verification failed: {}", source);
        }
    }
    rack::load_marketplace(source)
}

pub fn policy_eval_for_plugin(
    policy: &PolicyFile,
    p: &DiscoverItem,
    agent: AdapterTarget,
) -> PolicyEvalReport {
    match enforce_policy_for_plugin(policy, p) {
        Ok(_) => PolicyEvalReport {
            plugin: p.name.clone(),
            agent: format!("{:?}", agent).to_lowercase(),
            allowed: true,
            reason: "allowed".to_string(),
        },
        Err(e) => PolicyEvalReport {
            plugin: p.name.clone(),
            agent: format!("{:?}", agent).to_lowercase(),
            allowed: false,
            reason: e.to_string(),
        },
    }
}

pub fn enforce_policy_for_plugin(policy: &PolicyFile, p: &DiscoverItem) -> anyhow::Result<()> {
    if policy.general.denied_plugins.iter().any(|x| x == &p.name) {
        anyhow::bail!("policy denied plugin: {}", p.name);
    }
    if !policy.general.allowed_sources.is_empty()
        && !policy
            .general
            .allowed_sources
            .iter()
            .any(|s| source_matches_allowed(&p.marketplace_source, s))
    {
        anyhow::bail!("policy denied source: {}", p.marketplace_source);
    }
    if p.permissions
        .iter()
        .any(|perm| policy.general.blocked_permissions.contains(perm))
    {
        anyhow::bail!("policy blocked permission in plugin: {}", p.name);
    }

    if p.distribution.as_deref() == Some("external-reference-only")
        && !policy.general.allow_external_reference_installs
        && !policy
            .general
            .allow_external_reference_plugins
            .iter()
            .any(|x| x == &p.name)
    {
        anyhow::bail!(
            "policy blocked external-reference-only plugin: {} (set allow_external_reference_installs=true or add plugin to allow_external_reference_plugins)",
            p.name
        );
    }

    if policy.general.block_unknown_licenses {
        let cls = classify_plugin_license(p)?;
        if cls == "unknown"
            && !policy
                .general
                .allow_unknown_license_plugins
                .iter()
                .any(|x| x == &p.name)
        {
            anyhow::bail!(
                "policy blocked unknown-license plugin: {} (add to allow_unknown_license_plugins to override)",
                p.name
            );
        }
    }

    Ok(())
}

fn normalize_license_token(raw: &str) -> String {
    raw.trim().to_ascii_uppercase().replace(' ', "-")
}

fn classify_plugin_license(p: &DiscoverItem) -> anyhow::Result<&'static str> {
    let src = rack::resolve_plugin_path(&p.marketplace_source, &p.source)?;
    let mut tokens = Vec::new();

    for name in ["LICENSE", "LICENSE.md", "COPYING"] {
        let lp = src.join(name);
        if lp.exists() {
            let text = std::fs::read_to_string(lp)
                .unwrap_or_default()
                .to_ascii_uppercase();
            tokens.push(text);
        }
    }

    let manifest = src.join(".claude-plugin").join("plugin.json");
    if manifest.exists() {
        let raw = std::fs::read_to_string(manifest).unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(l) = v.get("license").and_then(|x| x.as_str()) {
                tokens.push(normalize_license_token(l));
            }
        }
    }

    let joined = tokens.join("\n");
    if joined.is_empty() {
        return Ok("unknown");
    }

    let permissive_needles = ["MIT", "APACHE", "BSD", "ISC", "UNLICENSE", "CC0"];
    if permissive_needles.iter().any(|n| joined.contains(n)) {
        return Ok("permissive");
    }

    let copyleft_needles = ["GPL", "AGPL", "LGPL", "MPL"];
    if copyleft_needles.iter().any(|n| joined.contains(n)) {
        return Ok("copyleft");
    }

    Ok("unknown")
}

pub fn run_rack_license_audit(marketplace_source: &str) -> String {
    let path = PathBuf::from(marketplace_source);
    if !path.exists() {
        return "not_applicable".to_string();
    }

    let rack_dir = if path.is_dir() {
        path
    } else if let Some(parent) = path.parent() {
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    };

    match rack_license_audit_readonly(&rack_dir) {
        Ok(r) if r.unknown_count == 0 => "ok".to_string(),
        Ok(_) => "failed".to_string(),
        Err(_) => "error".to_string(),
    }
}

pub fn parse_target(target: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = target.split('@').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), Some(parts[1].to_string()))
    } else {
        (target.to_string(), None)
    }
}

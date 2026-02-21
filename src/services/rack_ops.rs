use crate::domain::models::{CheckItem, RackDoctorReport, RackLicenseAuditSummary};
use std::path::PathBuf;

pub fn rack_doctor(rack_dir: &str, sign_key: Option<&str>) -> RackDoctorReport {
    let root = PathBuf::from(rack_dir);
    let checks = vec![
        CheckItem {
            name: "rack_dir_exists".to_string(),
            status: if root.exists() { "ok" } else { "missing" }.to_string(),
        },
        CheckItem {
            name: "marketplace_json".to_string(),
            status: if root.join(".pater/marketplace.json").exists() {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_official_snapshot".to_string(),
            status: if root
                .join("_upstreams/claude-plugins-official/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_claude_code_snapshot".to_string(),
            status: if root
                .join("_upstreams/claude-code/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "upstream_skills_snapshot".to_string(),
            status: if root
                .join("_upstreams/skills/.claude-plugin/marketplace.json")
                .exists()
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
        CheckItem {
            name: "sign_key".to_string(),
            status: match sign_key {
                Some(k) if PathBuf::from(k).exists() => "ok".to_string(),
                Some(_) => "missing".to_string(),
                None => "not_provided".to_string(),
            },
        },
        CheckItem {
            name: "openssl_available".to_string(),
            status: if std::process::Command::new("openssl")
                .arg("version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                "ok"
            } else {
                "missing"
            }
            .to_string(),
        },
    ];

    let overall = if checks
        .iter()
        .all(|c| c.status == "ok" || c.status == "not_provided")
    {
        "ok"
    } else {
        "needs_attention"
    }
    .to_string();

    RackDoctorReport { overall, checks }
}

fn parse_upstream_plugins(path: &PathBuf) -> Vec<serde_json::Value> {
    if !path.exists() {
        return vec![];
    }
    let raw = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    v.get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
}

pub fn rack_sync_upstreams(rack_dir: &str) -> anyhow::Result<usize> {
    let root = PathBuf::from(rack_dir);
    let upstreams = vec![
        root.join("_upstreams/claude-plugins-official/.claude-plugin/marketplace.json"),
        root.join("_upstreams/claude-code/.claude-plugin/marketplace.json"),
        root.join("_upstreams/skills/.claude-plugin/marketplace.json"),
    ];
    let mut seen = std::collections::HashSet::new();
    let mut plugins = Vec::<serde_json::Value>::new();
    for p in upstreams {
        for pl in parse_upstream_plugins(&p) {
            let name = pl
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || seen.contains(&name) {
                continue;
            }
            seen.insert(name);
            plugins.push(pl);
        }
    }
    let out = serde_json::json!({
        "name": "paternoster-rack",
        "owner": {"name": "Paternoster Rack"},
        "metadata": {"sync_priority": ["claude-plugins-official", "claude-code", "skills"]},
        "plugins": plugins,
    });
    let out_path = root.join(".pater/marketplace.json");
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out_path, serde_json::to_string_pretty(&out)?)?;
    Ok(seen.len())
}

fn classify_local_plugin_license(plugin_path: &std::path::Path) -> &'static str {
    for name in ["LICENSE", "LICENSE.md", "COPYING"] {
        let p = plugin_path.join(name);
        if p.exists() {
            let txt = std::fs::read_to_string(p)
                .unwrap_or_default()
                .to_ascii_uppercase();
            if ["MIT", "APACHE", "BSD", "ISC", "UNLICENSE", "CC0"]
                .iter()
                .any(|n| txt.contains(n))
            {
                return "permissive";
            }
            if ["GPL", "AGPL", "LGPL", "MPL"]
                .iter()
                .any(|n| txt.contains(n))
            {
                return "copyleft";
            }
        }
    }
    "unknown"
}

pub fn rack_license_audit_readonly(
    rack_dir: &std::path::Path,
) -> anyhow::Result<RackLicenseAuditSummary> {
    let mp = rack_dir.join(".pater/marketplace.json");
    let raw = std::fs::read_to_string(&mp)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let plugins = v
        .get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    let mut permissive = 0usize;
    let mut copyleft = 0usize;
    let mut unknown = 0usize;

    for p in plugins {
        let source = p.get("source");
        let mut cls = "unknown";
        if let Some(s) = source.and_then(|x| x.as_str()) {
            if s.starts_with("./") {
                cls = classify_local_plugin_license(&rack_dir.join(s.trim_start_matches("./")));
            }
        }
        match cls {
            "permissive" => permissive += 1,
            "copyleft" => copyleft += 1,
            _ => unknown += 1,
        }
    }

    Ok(RackLicenseAuditSummary {
        permissive,
        copyleft,
        unknown_count: unknown,
        total: permissive + copyleft + unknown,
    })
}

pub fn rack_license_audit(rack_dir: &str) -> anyhow::Result<RackLicenseAuditSummary> {
    let root = PathBuf::from(rack_dir);
    let summary = rack_license_audit_readonly(&root)?;

    let mp = root.join(".pater/marketplace.json");
    let raw = std::fs::read_to_string(&mp)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let plugins = v
        .get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    let mut detailed = Vec::new();
    for p in plugins {
        let name = p.get("name").and_then(|x| x.as_str()).unwrap_or("");
        let source = p.get("source");
        let mut cls = "unknown";
        if let Some(s) = source.and_then(|x| x.as_str()) {
            if s.starts_with("./") {
                cls = classify_local_plugin_license(&root.join(s.trim_start_matches("./")));
            }
        }
        detailed.push(serde_json::json!({"name": name, "classification": if cls=="unknown" {"proprietary/unknown"} else {cls}}));
    }

    let report = serde_json::json!({"plugins": detailed});
    std::fs::write(
        root.join(".pater/license-audit.json"),
        serde_json::to_string_pretty(&report)?,
    )?;

    Ok(summary)
}

pub fn rack_mark_unknown_external(rack_dir: &str) -> anyhow::Result<usize> {
    let root = PathBuf::from(rack_dir);
    let mp_path = root.join(".pater/marketplace.json");
    let audit_path = root.join(".pater/license-audit.json");
    let mut m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mp_path)?)?;
    let a: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&audit_path)?)?;
    let mut unknown = std::collections::HashSet::new();
    if let Some(arr) = a.get("plugins").and_then(|x| x.as_array()) {
        for p in arr {
            if p.get("classification").and_then(|x| x.as_str()) == Some("proprietary/unknown") {
                if let Some(name) = p.get("name").and_then(|x| x.as_str()) {
                    unknown.insert(name.to_string());
                }
            }
        }
    }

    let mut marked = 0usize;
    if let Some(arr) = m.get_mut("plugins").and_then(|x| x.as_array_mut()) {
        for p in arr {
            if let Some(name) = p.get("name").and_then(|x| x.as_str()) {
                if unknown.contains(name) {
                    p["distribution"] =
                        serde_json::Value::String("external-reference-only".to_string());
                    p["license_status"] = serde_json::Value::String("unknown".to_string());
                    marked += 1;
                }
            }
        }
    }

    std::fs::write(mp_path, serde_json::to_string_pretty(&m)?)?;
    Ok(marked)
}

pub fn rack_sign_marketplace(rack_dir: &str, sign_key: &str) -> anyhow::Result<()> {
    let root = PathBuf::from(rack_dir);
    let status = std::process::Command::new("openssl")
        .args([
            "pkeyutl",
            "-sign",
            "-inkey",
            sign_key,
            "-rawin",
            "-in",
            root.join(".pater/marketplace.json")
                .to_string_lossy()
                .as_ref(),
            "-out",
            root.join(".pater/marketplace.sig.bin")
                .to_string_lossy()
                .as_ref(),
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("openssl signing failed")
    }
    let bytes = std::fs::read(root.join(".pater/marketplace.sig.bin"))?;
    std::fs::write(
        root.join(".pater/marketplace.sig"),
        format!("{}\n", hex::encode(bytes)),
    )?;
    let _ = std::fs::remove_file(root.join(".pater/marketplace.sig.bin"));
    Ok(())
}

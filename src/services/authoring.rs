use std::path::PathBuf;

fn rack_marketplace_path(rack_dir: &str) -> PathBuf {
    PathBuf::from(rack_dir).join(".pater/marketplace.json")
}

fn load_marketplace_value(rack_dir: &str) -> anyhow::Result<serde_json::Value> {
    let p = rack_marketplace_path(rack_dir);
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}

fn save_marketplace_value(rack_dir: &str, v: &serde_json::Value) -> anyhow::Result<()> {
    let p = rack_marketplace_path(rack_dir);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(v)?)?;
    Ok(())
}

fn get_plugin_mut<'a>(
    m: &'a mut serde_json::Value,
    name: &str,
) -> anyhow::Result<&'a mut serde_json::Value> {
    let arr = m
        .get_mut("plugins")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("missing plugins array"))?;
    arr.iter_mut()
        .find(|p| p.get("name").and_then(|x| x.as_str()) == Some(name))
        .ok_or_else(|| anyhow::anyhow!("plugin not found: {}", name))
}

fn ensure_array_field<'a>(
    obj: &'a mut serde_json::Value,
    field: &str,
) -> anyhow::Result<&'a mut Vec<serde_json::Value>> {
    if obj.get(field).is_none() {
        obj[field] = serde_json::Value::Array(vec![]);
    }
    obj.get_mut(field)
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("invalid array field {}", field))
}

pub fn plugin_create(rack_dir: &str, name: &str, description: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    if m.get("plugins").is_none() {
        m["plugins"] = serde_json::Value::Array(vec![]);
    }
    let arr = m.get_mut("plugins").and_then(|x| x.as_array_mut()).unwrap();
    if arr
        .iter()
        .any(|p| p.get("name").and_then(|x| x.as_str()) == Some(name))
    {
        anyhow::bail!("plugin exists: {}", name);
    }
    arr.push(serde_json::json!({"name": name, "source": format!("./plugins/{}", name), "description": description, "version": "0.1.0", "skills": [], "hooks": [], "subagents": []}));
    save_marketplace_value(rack_dir, &m)?;

    let base = PathBuf::from(rack_dir).join("plugins").join(name);
    std::fs::create_dir_all(base.join(".claude-plugin"))?;
    std::fs::create_dir_all(base.join("skills"))?;
    std::fs::write(
        base.join(".claude-plugin/plugin.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({"name": name, "description": description, "version": "0.1.0"}),
        )?,
    )?;
    Ok(())
}

pub fn plugin_update(
    rack_dir: &str,
    name: &str,
    description: Option<String>,
    version: Option<String>,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, name)?;
    if let Some(d) = description {
        p["description"] = serde_json::Value::String(d);
    }
    if let Some(v) = version {
        p["version"] = serde_json::Value::String(v.clone());
        let manifest = PathBuf::from(rack_dir)
            .join("plugins")
            .join(name)
            .join(".claude-plugin/plugin.json");
        if manifest.exists() {
            let mut mv: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
            mv["version"] = serde_json::Value::String(v);
            std::fs::write(manifest, serde_json::to_string_pretty(&mv)?)?;
        }
    }
    save_marketplace_value(rack_dir, &m)
}

pub fn plugin_remove(rack_dir: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let arr = m
        .get_mut("plugins")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("missing plugins array"))?;
    arr.retain(|p| p.get("name").and_then(|x| x.as_str()) != Some(name));
    save_marketplace_value(rack_dir, &m)?;
    let dir = PathBuf::from(rack_dir).join("plugins").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn skill_create(
    rack_dir: &str,
    plugin: &str,
    name: &str,
    description: &str,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let skills = ensure_array_field(p, "skills")?;
    if !skills.iter().any(|s| s.as_str() == Some(name)) {
        skills.push(serde_json::Value::String(name.to_string()));
    }
    save_marketplace_value(rack_dir, &m)?;
    let d = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join("skills")
        .join(name);
    std::fs::create_dir_all(&d)?;
    std::fs::write(
        d.join("SKILL.md"),
        format!("---\ndescription: {}\n---\n\n# {}\n", description, name),
    )?;
    Ok(())
}

pub fn skill_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let skills = ensure_array_field(p, "skills")?;
    skills.retain(|s| s.as_str() != Some(name));
    save_marketplace_value(rack_dir, &m)?;
    let d = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join("skills")
        .join(name);
    if d.exists() {
        std::fs::remove_dir_all(d)?;
    }
    Ok(())
}

pub fn subagent_create(
    rack_dir: &str,
    plugin: &str,
    name: &str,
    purpose: &str,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "subagents")?;
    arr.retain(|x| x.get("name").and_then(|v| v.as_str()) != Some(name));
    arr.push(serde_json::json!({"name":name,"purpose":purpose}));
    save_marketplace_value(rack_dir, &m)
}

pub fn subagent_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "subagents")?;
    arr.retain(|x| x.get("name").and_then(|v| v.as_str()) != Some(name));
    save_marketplace_value(rack_dir, &m)
}

pub fn hook_create(
    rack_dir: &str,
    plugin: &str,
    agent: &str,
    event: &str,
    run: &str,
) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "hooks")?;
    arr.push(serde_json::json!({"agent":agent,"event":event,"run":run}));
    save_marketplace_value(rack_dir, &m)
}

pub fn hook_remove(rack_dir: &str, plugin: &str, agent: &str, event: &str) -> anyhow::Result<()> {
    let mut m = load_marketplace_value(rack_dir)?;
    let p = get_plugin_mut(&mut m, plugin)?;
    let arr = ensure_array_field(p, "hooks")?;
    arr.retain(|x| {
        !(x.get("agent").and_then(|v| v.as_str()) == Some(agent)
            && x.get("event").and_then(|v| v.as_str()) == Some(event))
    });
    save_marketplace_value(rack_dir, &m)
}

pub fn mcp_create(rack_dir: &str, plugin: &str, name: &str, command: &str) -> anyhow::Result<()> {
    let manifest = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join(".claude-plugin/plugin.json");
    if !manifest.exists() {
        anyhow::bail!("plugin manifest not found for {}", plugin);
    }
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    if v.get("mcps").is_none() {
        v["mcps"] = serde_json::json!([]);
    }
    let arr = v
        .get_mut("mcps")
        .and_then(|x| x.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("invalid mcps"))?;
    arr.retain(|x| x.get("name").and_then(|s| s.as_str()) != Some(name));
    arr.push(serde_json::json!({"name":name,"command":command}));
    std::fs::write(manifest, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

pub fn mcp_remove(rack_dir: &str, plugin: &str, name: &str) -> anyhow::Result<()> {
    let manifest = PathBuf::from(rack_dir)
        .join("plugins")
        .join(plugin)
        .join(".claude-plugin/plugin.json");
    if !manifest.exists() {
        return Ok(());
    }
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    if let Some(arr) = v.get_mut("mcps").and_then(|x| x.as_array_mut()) {
        arr.retain(|x| x.get("name").and_then(|s| s.as_str()) != Some(name));
    }
    std::fs::write(manifest, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

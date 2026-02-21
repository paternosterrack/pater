use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Index {
    pub version: String,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Skill {
    pub id: String,
    pub version: String,
    pub summary: String,
    pub agents: Vec<String>,
    pub hooks: Vec<Hook>,
    pub subagents: Vec<Subagent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hook {
    pub agent: String,
    pub event: String,
    pub run: String,
    #[serde(skip)]
    pub skill_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subagent {
    pub name: String,
    pub purpose: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RackError {
    #[error("skill not found: {0}")]
    SkillNotFound(String),
    #[error("duplicate skill id: {0}")]
    DuplicateSkill(String),
}

pub fn load_index(path: &str) -> anyhow::Result<Index> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn search<'a>(idx: &'a Index, query: &str) -> Vec<&'a Skill> {
    let q = query.to_ascii_lowercase();
    idx.skills
        .iter()
        .filter(|s| s.id.to_ascii_lowercase().contains(&q) || s.summary.to_ascii_lowercase().contains(&q))
        .collect()
}

pub fn show<'a>(idx: &'a Index, id: &str) -> anyhow::Result<&'a Skill> {
    idx.skills
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| RackError::SkillNotFound(id.to_string()).into())
}

pub fn list_hooks(idx: &Index, agent: Option<&str>) -> Vec<Hook> {
    let mut out = Vec::new();
    for skill in &idx.skills {
        for h in &skill.hooks {
            if agent.map(|a| a == h.agent).unwrap_or(true) {
                let mut h2 = h.clone();
                h2.skill_id = skill.id.clone();
                out.push(h2);
            }
        }
    }
    out
}

pub fn validate(idx: &Index) -> anyhow::Result<()> {
    let mut seen = std::collections::HashSet::new();
    for s in &idx.skills {
        if !seen.insert(&s.id) {
            return Err(RackError::DuplicateSkill(s.id.clone()).into());
        }
    }
    Ok(())
}

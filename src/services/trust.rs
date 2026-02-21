use crate::rack;
use std::path::PathBuf;

fn trusted_pubkeys_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".config/pater/trust/pubkeys.txt"))
}

pub fn list_pubkeys() -> anyhow::Result<Vec<String>> {
    let path = trusted_pubkeys_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect())
}

pub fn trust_init(official_pubkey_hex: &str) -> anyhow::Result<()> {
    let path = trusted_pubkeys_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    if !existing.lines().any(|l| l.trim() == official_pubkey_hex) {
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(official_pubkey_hex);
        existing.push('\n');
        std::fs::write(path, existing)?;
    }
    Ok(())
}

fn load_trusted_pubkeys() -> anyhow::Result<Vec<ed25519_dalek::VerifyingKey>> {
    let path = trusted_pubkeys_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let bytes = hex::decode(l)?;
        if bytes.len() != 32 {
            continue;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        if let Ok(k) = ed25519_dalek::VerifyingKey::from_bytes(&arr) {
            out.push(k);
        }
    }
    Ok(out)
}

pub fn verify_marketplace_signature(source: &str) -> anyhow::Result<bool> {
    let raw = rack::load_marketplace_raw(source)?;
    let sigs_raw = rack::load_marketplace_signature(source)?;
    let keys = load_trusted_pubkeys()?;
    if keys.is_empty() {
        return Ok(false);
    }

    let mut signatures = Vec::new();
    for line in sigs_raw.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let sig_bytes = match hex::decode(s) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sig_arr: [u8; 64] = match sig_bytes.as_slice().try_into() {
            Ok(a) => a,
            Err(_) => continue,
        };
        signatures.push(ed25519_dalek::Signature::from_bytes(&sig_arr));
    }

    for sig in signatures {
        for k in &keys {
            if k.verify_strict(raw.as_bytes(), &sig).is_ok() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

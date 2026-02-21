use std::path::PathBuf;

pub fn canonical_market_source_id(raw: &str) -> String {
    let s = raw.trim();

    if s.split('/').count() == 2 && !s.contains("://") && !s.starts_with('.') {
        return format!("github:{}", s.to_ascii_lowercase());
    }

    if let Some(rest) = s.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0];
            let repo = parts[1].trim_end_matches(".git");
            if !owner.is_empty() && !repo.is_empty() {
                return format!(
                    "github:{}/{}",
                    owner.to_ascii_lowercase(),
                    repo.to_ascii_lowercase()
                );
            }
        }
    }

    if let Some(rest) = s.strip_prefix("https://raw.githubusercontent.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0];
            let repo = parts[1];
            if !owner.is_empty() && !repo.is_empty() {
                return format!(
                    "github:{}/{}",
                    owner.to_ascii_lowercase(),
                    repo.to_ascii_lowercase()
                );
            }
        }
    }

    let p = PathBuf::from(s);
    if p.exists() {
        if let Ok(c) = p.canonicalize() {
            return format!("path:{}", c.to_string_lossy());
        }
    }

    s.trim_end_matches('/').to_ascii_lowercase()
}

pub fn source_matches_allowed(source: &str, allowed: &str) -> bool {
    canonical_market_source_id(source) == canonical_market_source_id(allowed)
}

#[cfg(test)]
mod tests {
    use super::{canonical_market_source_id, source_matches_allowed};

    #[test]
    fn source_matching_normalizes_github_forms() {
        assert!(source_matches_allowed(
            "paternosterrack/rack",
            "https://github.com/paternosterrack/rack.git"
        ));
        assert!(source_matches_allowed(
            "paternosterrack/rack",
            "https://raw.githubusercontent.com/paternosterrack/rack/main/.pater/marketplace.json"
        ));
    }

    #[test]
    fn source_matching_rejects_prefix_tricks() {
        assert!(!source_matches_allowed(
            "https://github.com/paternosterrack/rack-evil",
            "https://github.com/paternosterrack/rack"
        ));
    }

    #[test]
    fn canonical_id_is_stable_for_github_shorthand() {
        assert_eq!(
            canonical_market_source_id("PaternosterRack/Rack"),
            "github:paternosterrack/rack"
        );
    }
}

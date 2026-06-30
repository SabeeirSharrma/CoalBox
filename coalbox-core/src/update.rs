use serde::Deserialize;

use crate::error::CoalboxError;

const GITHUB_API: &str = "https://api.github.com/repos/SabeeirSharrma/CoalBox/releases/latest";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
}

#[derive(Debug, Clone)]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release: Option<ReleaseInfo>,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn check_for_update() -> Result<UpdateCheck, CoalboxError> {
    let current = current_version();

    let response_text = ureq::get(GITHUB_API)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", format!("coalbox/{}", current))
        .call()
        .map_err(|e| CoalboxError::Import(format!("Failed to check for updates: {}", e)))?
        .body_mut()
        .read_to_string()
        .map_err(|e| CoalboxError::Import(format!("Failed to read response: {}", e)))?;

    let response: ReleaseInfo = serde_json::from_str(&response_text)
        .map_err(|e| CoalboxError::Import(format!("Failed to parse response: {}", e)))?;

    let latest = response.tag_name.trim_start_matches('v').to_string();

    let update_available = compare_versions(&current, &latest);

    Ok(UpdateCheck {
        current_version: current,
        latest_version: latest,
        update_available,
        release: Some(response),
    })
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let cur = parse(current);
    let lat = parse(latest);

    for (c, l) in cur.iter().zip(lat.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    lat.len() > cur.len()
}

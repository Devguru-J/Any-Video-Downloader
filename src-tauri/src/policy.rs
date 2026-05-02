use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const POLICY_URL: &str =
    "https://github.com/Devguru-J/Any-Video-Downloader/releases/latest/download/policy.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub min_required_version: String,
    pub message: String,
}

impl Default for UpdatePolicy {
    fn default() -> Self {
        Self {
            min_required_version: "0.0.0".into(),
            message: "Please update to continue.".into(),
        }
    }
}

/// Fetch policy from CDN; on failure fall back to local cache; on no cache,
/// return a permissive default so we never lock out users due to a network
/// blip.
pub async fn fetch_policy(app: &AppHandle) -> Result<UpdatePolicy> {
    let cache = cache_path(app)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(concat!("AnyVideoDownloader/", env!("CARGO_PKG_VERSION")))
        .build()?;

    match client.get(POLICY_URL).send().await {
        Ok(r) if r.status().is_success() => {
            let policy: UpdatePolicy = r.json().await?;
            // Persist for offline fallback.
            if let Some(parent) = cache.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&cache, serde_json::to_vec(&policy)?);
            Ok(policy)
        }
        _ => {
            if let Ok(bytes) = std::fs::read(&cache) {
                if let Ok(p) = serde_json::from_slice::<UpdatePolicy>(&bytes) {
                    return Ok(p);
                }
            }
            Ok(UpdatePolicy::default())
        }
    }
}

fn cache_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app.path().app_local_data_dir()?.join("policy.json"))
}

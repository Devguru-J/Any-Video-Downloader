use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Resolve a sidecar binary path inside the app's resource dir.
/// Tauri places `externalBin` entries next to the main executable, with a
/// platform/triple suffix in dev (e.g. `yt-dlp-x86_64-apple-darwin`).
/// In a packaged app the suffix is stripped.
pub fn sidecar_path(app: &AppHandle, name: &str) -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("no parent for current exe"))?
        .to_path_buf();

    let candidates = [
        exe_dir.join(name),
        exe_dir.join(format!("{name}{}", if cfg!(windows) { ".exe" } else { "" })),
        exe_dir.join(format!("{name}-{}", target_triple())),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    // Fall back to resource_dir for packaged macOS bundles.
    if let Ok(res_dir) = app.path().resource_dir() {
        let r = res_dir.join(name);
        if r.exists() {
            return Ok(r);
        }
    }

    // Last resort: PATH lookup. Useful in dev before binaries are vendored.
    if let Ok(p) = which::which(name) {
        return Ok(p);
    }

    Err(anyhow!(
        "sidecar binary `{name}` not found near {}",
        exe_dir.display()
    ))
}

fn target_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "aarch64-unknown-linux-gnu"
    }
}

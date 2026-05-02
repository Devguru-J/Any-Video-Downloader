use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tauri::AppHandle;
use tokio::process::Command;

use crate::sidecar::sidecar_path;

/// Modern Chrome UA — many sites refuse the default yt-dlp UA outright.
const BROWSER_UA: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(Debug, Serialize, Clone)]
pub struct VideoFormat {
    pub id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub tbr: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VideoMeta {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub formats: Vec<VideoFormat>,
    pub is_playlist: bool,
    pub playlist_count: Option<u32>,
}

/// Run `yt-dlp -J` against a URL and parse the JSON metadata.
pub async fn probe(app: &AppHandle, url: &str) -> Result<VideoMeta> {
    let bin = sidecar_path(app, "yt-dlp")?;

    let output = Command::new(&bin)
        .arg("-J")
        .arg("--no-playlist")
        .arg("--no-warnings")
        .arg("--ignore-config")
        // Impersonate a real browser; yt-dlp picks the best available
        // impersonation target. Critical for Cloudflare-fronted sites.
        .arg("--extractor-args")
        .arg("generic:impersonate")
        .arg("--user-agent")
        .arg(BROWSER_UA)
        .arg("--add-header")
        .arg("Accept-Language: en-US,en;q=0.9,ko;q=0.8")
        .arg("--")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("spawning yt-dlp at {}", bin.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("yt-dlp failed: {}", stderr.trim()));
    }

    let v: Value = serde_json::from_slice(&output.stdout)
        .context("parsing yt-dlp JSON")?;

    parse_meta(url, &v)
}

fn parse_meta(url: &str, v: &Value) -> Result<VideoMeta> {
    let is_playlist = v.get("_type").and_then(Value::as_str) == Some("playlist");

    let title = v
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled")
        .to_string();
    let thumbnail = v.get("thumbnail").and_then(Value::as_str).map(String::from);
    let duration = v.get("duration").and_then(Value::as_f64);
    let uploader = v
        .get("uploader")
        .or_else(|| v.get("channel"))
        .and_then(Value::as_str)
        .map(String::from);

    let formats = v
        .get("formats")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(parse_format).collect())
        .unwrap_or_default();

    let playlist_count = v.get("playlist_count").and_then(Value::as_u64).map(|n| n as u32);

    Ok(VideoMeta {
        url: url.to_string(),
        title,
        thumbnail,
        duration,
        uploader,
        formats,
        is_playlist,
        playlist_count,
    })
}

fn parse_format(v: &Value) -> Option<VideoFormat> {
    let id = v.get("format_id").and_then(Value::as_str)?.to_string();
    let ext = v
        .get("ext")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let resolution = v
        .get("resolution")
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| {
            let w = v.get("width").and_then(Value::as_u64)?;
            let h = v.get("height").and_then(Value::as_u64)?;
            Some(format!("{w}x{h}"))
        });

    Some(VideoFormat {
        id,
        ext,
        resolution,
        fps: v.get("fps").and_then(Value::as_f64),
        vcodec: v.get("vcodec").and_then(Value::as_str).map(String::from),
        acodec: v.get("acodec").and_then(Value::as_str).map(String::from),
        filesize: v
            .get("filesize")
            .or_else(|| v.get("filesize_approx"))
            .and_then(Value::as_u64),
        tbr: v.get("tbr").and_then(Value::as_f64),
        note: v.get("format_note").and_then(Value::as_str).map(String::from),
    })
}

#[derive(Debug, Deserialize)]
pub struct ProgressLine {
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    pub speed: Option<f64>,
    pub eta: Option<f64>,
    pub filename: Option<String>,
}

/// Build the yt-dlp argv for a download. Keeps the `--ffmpeg-location` near
/// the bundled binary so muxing works without ffmpeg on PATH.
pub fn download_argv(
    yt_dlp: &Path,
    ffmpeg: &Path,
    url: &str,
    format_id: &str,
    output_dir: &Path,
) -> Vec<String> {
    let template = output_dir.join("%(title).100B [%(id)s].%(ext)s");
    let progress_template = r#"download:{"id":"%(info.id)s","downloaded":%(progress.downloaded_bytes)s,"total":%(progress.total_bytes,progress.total_bytes_estimate)s,"speed":%(progress.speed)s,"eta":%(progress.eta)s,"status":"%(progress.status)s","filename":"%(progress.filename)s"}"#;

    vec![
        yt_dlp.display().to_string(),
        "--no-playlist".into(),
        "--no-warnings".into(),
        "--ignore-config".into(),
        "--newline".into(),
        "--progress-template".into(),
        progress_template.into(),
        "--ffmpeg-location".into(),
        ffmpeg.display().to_string(),
        // Browser impersonation — needed for Cloudflare-protected sites.
        "--extractor-args".into(),
        "generic:impersonate".into(),
        "--user-agent".into(),
        BROWSER_UA.into(),
        "--add-header".into(),
        "Accept-Language: en-US,en;q=0.9,ko;q=0.8".into(),
        // Retry transient failures aggressively.
        "--retries".into(),
        "10".into(),
        "--fragment-retries".into(),
        "10".into(),
        "-f".into(),
        format_id.to_string(),
        "-o".into(),
        template.display().to_string(),
        "--".into(),
        url.to_string(),
    ]
}

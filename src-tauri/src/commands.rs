use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

use crate::jobs::{now_secs, DownloadJob, JobRegistry, JobState, JobStatus};
use crate::policy::{fetch_policy, UpdatePolicy};
use crate::scraper;
use crate::sidecar::sidecar_path;
use crate::ytdlp::{self, ProgressLine, VideoMeta};

#[tauri::command]
pub async fn probe_url(
    app: AppHandle,
    url: String,
    cookies_from_browser: String,
) -> Result<VideoMeta, String> {
    auto_probe(&app, &url, &cookies_from_browser)
        .await
        .map_err(|e| e.to_string())
}

/// Try to extract a video from `url` automatically. Strategy:
///   1. yt-dlp directly (fast path; covers ~most public sites).
///   2. If that fails, open a hidden Tauri webview with the page so the real
///      browser engine handles any Cloudflare JS challenge naturally and JS
///      can resolve dynamic players. Capture iframe + media URLs.
///   3. Probe each captured candidate with yt-dlp; the first one that
///      yields formats wins.
///   4. As a last resort, synthesise a minimal VideoMeta from a direct
///      m3u8/mp4 URL so the user can still download it.
async fn auto_probe(
    app: &AppHandle,
    url: &str,
    cookies_from_browser: &str,
) -> anyhow::Result<VideoMeta> {
    let direct = ytdlp::probe(app, url, cookies_from_browser).await;
    if let Ok(meta) = &direct {
        if !meta.formats.is_empty() {
            return direct;
        }
    }
    let direct_err = direct.err();

    tracing::info!("yt-dlp direct probe failed; spinning up scraper webview");

    let mut candidates = scraper::scrape(app, url, Duration::from_secs(20)).await?;
    scraper::rank(&mut candidates);
    tracing::info!(count = candidates.len(), "scraper finished");

    if candidates.is_empty() {
        return Err(direct_err.unwrap_or_else(|| {
            anyhow::anyhow!("이 페이지에서 영상 후보를 찾지 못했습니다.")
        }));
    }

    // Try yt-dlp on each candidate — give the page URL as referer so private
    // CDNs that require it accept the request.
    let mut last_err: Option<anyhow::Error> = None;
    for c in &candidates {
        match ytdlp::probe_with_referer(app, &c.url, cookies_from_browser, Some(url)).await {
            Ok(meta) if !meta.formats.is_empty() => {
                tracing::info!(picked = %c.url, "scraper candidate succeeded");
                return Ok(meta);
            }
            Ok(_) => {}
            Err(e) => last_err = Some(e),
        }
    }

    // Synthesise a VideoMeta from the best direct media URL we saw, so the
    // user can still kick off a download. yt-dlp can usually fetch a raw
    // m3u8/mp4 URL even if it can't classify the surrounding page.
    if let Some(c) = candidates
        .iter()
        .find(|c| c.kind == "media")
        .or_else(|| candidates.first())
    {
        return Ok(VideoMeta {
            url: c.url.clone(),
            title: extract_title_from_url(url),
            thumbnail: None,
            duration: None,
            uploader: None,
            formats: vec![ytdlp::VideoFormat {
                id: "best".into(),
                ext: ext_from_url(&c.url).unwrap_or_else(|| "mp4".into()),
                resolution: None,
                fps: None,
                vcodec: None,
                acodec: None,
                filesize: None,
                tbr: None,
                note: Some("스캐너가 찾은 직접 링크".into()),
            }],
            is_playlist: false,
            playlist_count: None,
        });
    }

    Err(last_err.unwrap_or_else(|| {
        anyhow::anyhow!("이 페이지에서 다운로드 가능한 영상을 찾지 못했습니다.")
    }))
}

fn extract_title_from_url(u: &str) -> String {
    url::Url::parse(u)
        .ok()
        .and_then(|p| p.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "video".into())
}

fn ext_from_url(u: &str) -> Option<String> {
    let lower = u.to_lowercase();
    for cand in ["m3u8", "mpd", "mp4", "webm", "ts"] {
        if lower.contains(&format!(".{cand}")) {
            return Some(if cand == "m3u8" { "mp4".into() } else { cand.into() });
        }
    }
    None
}

#[tauri::command]
pub async fn list_jobs(reg: State<'_, JobRegistry>) -> Result<Vec<JobState>, String> {
    Ok(reg.list())
}

#[tauri::command]
pub async fn clear_history(reg: State<'_, JobRegistry>) -> Result<(), String> {
    reg.clear_finished();
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    reg: State<'_, JobRegistry>,
    job_id: String,
) -> Result<(), String> {
    reg.cancel(&job_id);
    Ok(())
}

#[tauri::command]
pub async fn open_in_finder(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_policy(app: AppHandle) -> Result<UpdatePolicy, String> {
    fetch_policy(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    reg: State<'_, JobRegistry>,
    job: DownloadJob,
    cookies_from_browser: String,
) -> Result<String, String> {
    let yt = sidecar_path(&app, "yt-dlp").map_err(|e| e.to_string())?;
    let ff = sidecar_path(&app, "ffmpeg").map_err(|e| e.to_string())?;
    let out = PathBuf::from(&job.output_dir);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let argv = ytdlp::download_argv(
        &yt,
        &ff,
        &job.url,
        &job.format_id,
        &out,
        &cookies_from_browser,
        job.referer.as_deref(),
    );
    tracing::debug!(?argv, "spawning yt-dlp");

    let initial = JobState {
        job: job.clone(),
        status: JobStatus::Queued,
        started_at: now_secs(),
        ended_at: None,
    };
    reg.upsert(initial.clone());
    let _ = app.emit("download://progress", &initial);

    let (cancel_tx, cancel_rx) = oneshot::channel();
    reg.register_cancel(&job.id, cancel_tx);

    let app_for_task = app.clone();
    let reg_for_task: JobRegistry = (*reg).clone();
    let job_id_return = job.id.clone();
    let job_id_for_task = job.id.clone();

    tauri::async_runtime::spawn(async move {
        let started = now_secs();
        let result = run_job(
            app_for_task.clone(),
            reg_for_task.clone(),
            job.clone(),
            argv,
            cancel_rx,
            started,
        )
        .await;

        reg_for_task.drop_cancel(&job_id_for_task);

        if let Err(e) = result {
            let state = JobState {
                job,
                status: JobStatus::Error {
                    message: e.to_string(),
                },
                started_at: started,
                ended_at: Some(now_secs()),
            };
            reg_for_task.upsert(state.clone());
            let _ = app_for_task.emit("download://progress", &state);
        }
    });

    Ok(job_id_return)
}

async fn run_job(
    app: AppHandle,
    reg: JobRegistry,
    job: DownloadJob,
    argv: Vec<String>,
    mut cancel_rx: oneshot::Receiver<()>,
    started: u64,
) -> anyhow::Result<()> {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(unix)]
    {
        // Put yt-dlp in its own session so child.kill() takes the whole tree.
        // tokio::process::Command exposes pre_exec as an inherent method on Unix.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        // Hide the console window for the spawned process. CREATE_NO_WINDOW.
        cmd.creation_flags(0x0800_0000);
    }

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("piped");
    let stderr = child.stderr.take().expect("piped");

    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let mut last_filename: Option<String> = None;
    let mut last_stderr_line: Option<String> = None;

    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                let _ = child.kill().await;
                let state = JobState {
                    job: job.clone(),
                    status: JobStatus::Cancelled,
                    started_at: started,
                    ended_at: Some(now_secs()),
                };
                reg.upsert(state.clone());
                let _ = app.emit("download://progress", &state);
                return Ok(());
            }
            line = out_reader.next_line() => {
                match line? {
                    Some(l) => handle_stdout(&app, &reg, &job, &l, started, &mut last_filename),
                    None => break,
                }
            }
            line = err_reader.next_line() => {
                if let Ok(Some(l)) = line {
                    if !l.trim().is_empty() {
                        last_stderr_line = Some(l);
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        let msg = last_stderr_line
            .unwrap_or_else(|| format!("yt-dlp exited with status {status}"));
        let state = JobState {
            job: job.clone(),
            status: JobStatus::Error { message: msg },
            started_at: started,
            ended_at: Some(now_secs()),
        };
        reg.upsert(state.clone());
        let _ = app.emit("download://progress", &state);
        return Ok(());
    }

    let path = last_filename.unwrap_or_else(|| job.output_dir.clone());
    let state = JobState {
        job: job.clone(),
        status: JobStatus::Done { path },
        started_at: started,
        ended_at: Some(now_secs()),
    };
    reg.upsert(state.clone());
    let _ = app.emit("download://progress", &state);
    Ok(())
}

fn handle_stdout(
    app: &AppHandle,
    reg: &JobRegistry,
    job: &DownloadJob,
    line: &str,
    started: u64,
    last_filename: &mut Option<String>,
) {
    if let Some(rest) = line.strip_prefix("download:") {
        if let Ok(p) = serde_json::from_str::<ProgressLine>(rest) {
            if let Some(name) = &p.filename {
                if !name.is_empty() && name != "NA" {
                    *last_filename = Some(name.clone());
                }
            }
            let state = JobState {
                job: job.clone(),
                status: JobStatus::Running {
                    downloaded: p.downloaded.unwrap_or(0),
                    total: p.total.unwrap_or(0),
                    speed: p.speed.unwrap_or(0.0),
                    eta: p.eta.unwrap_or(0.0),
                },
                started_at: started,
                ended_at: None,
            };
            reg.upsert(state.clone());
            let _ = app.emit("download://progress", &state);
            return;
        }
    }

    if let Some(rest) = line.strip_prefix("[Merger] Merging formats into \"") {
        if let Some(end) = rest.rfind('"') {
            *last_filename = Some(rest[..end].to_string());
        }
    } else if let Some(rest) = line.strip_prefix("[download] Destination: ") {
        *last_filename = Some(rest.to_string());
    }
}

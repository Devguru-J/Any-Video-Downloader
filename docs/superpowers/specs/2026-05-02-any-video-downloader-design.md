# Any Video Downloader — Design Spec

**Date:** 2026-05-02
**Repo:** https://github.com/Devguru-J/Any-Video-Downloader
**Targets:** macOS (Intel + Apple Silicon), Windows (x64)

## 1. Goals

A small desktop app that, given any web page URL containing a video, lets the
user pick a destination folder and download the video — including pages that
embed JW Player, HLS (m3u8), DASH (mpd), or generic HTML5 `<video>` tags.

Non-goals: browser extension, mobile, streaming playback, transcoding presets,
torrent.

## 2. User flow

```
Paste URL  →  Detect (probe page)  →  Pick formats/quality  →  Pick folder
            ↘                       ↗
              (auto-detect on paste)
                                                                    ↓
                                                          Download starts; progress
                                                          shown in queue. Items move
                                                          to History when done.
```

## 3. Architecture

```
┌──────────────────── Tauri 2.0 app ────────────────────┐
│                                                       │
│  React + TS + Tailwind   ←── IPC ──→   Rust backend   │
│  (sidebar + queue UI)                  (commands)     │
│                                              │        │
│                                              ↓        │
│                                  Sidecar processes:   │
│                                  • yt-dlp (extract)   │
│                                  • ffmpeg (mux/HLS)   │
│                                                       │
│  Tauri updater plugin  ←──→  GitHub Releases          │
│  Force-update check    ←──→  version.json (CDN)       │
└───────────────────────────────────────────────────────┘
```

**Why Tauri over Electron:** ~10 MB installer vs ~150 MB; native webview;
first-class signed auto-updater; Rust process orchestration suits sidecars.

**Why yt-dlp:** de-facto standard, handles JW Player, generic HLS/DASH,
embedded `<video>`, 1800+ site-specific extractors. Bundled as a sidecar so
users never see a CLI.

## 4. Detection & download engine

The Rust backend exposes Tauri commands; React calls them via `invoke()`.

| Command | Purpose |
|---|---|
| `probe_url(url)` | Run `yt-dlp -J --no-playlist --no-warnings <url>` → return parsed metadata: title, thumbnail, list of formats `(id, ext, resolution, vcodec, acodec, filesize, tbr)` |
| `list_playlist(url)` | If a playlist, return entries (title, url) so the user can pick subset |
| `start_download(job)` | Spawn yt-dlp with chosen format selector (`-f`), output template, progress JSON line mode (`--progress-template '{"id":"%(info.id)s","downloaded":%(progress.downloaded_bytes)s,"total":%(progress.total_bytes)s,"speed":%(progress.speed)s,"eta":%(progress.eta)s}'`). Stream stdout, parse, emit `download://progress` events to React. |
| `cancel_download(jobId)` | Kill child process tree |
| `open_in_finder(path)` | Reveal completed file |

Format selection default: `bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best`.
ffmpeg sidecar handles muxing.

**JW Player & generic pages:** yt-dlp's "generic" extractor scans the HTML
for JW Player setup blocks, m3u8/mpd manifests, and `<video src>` tags. For
sites where JS is required, we fall back to a `--cookies-from-browser` hint
(user can opt in) and, in v1.1, an embedded headless webview that captures
the resolved manifest URL.

**Concurrency:** queue with N=3 simultaneous downloads (configurable). Each
job is one yt-dlp process so cancel/pause maps cleanly.

## 5. GUI (direction B — sidebar + queue)

Layout:

```
┌──────────────┬────────────────────────────────────────┐
│ AnyVideo     │  [ 🔗 Paste URL                  Add ] │
│              │                                        │
│ ⬇ Active   3 │  ┌─ Active queue ─────────────────┐    │
│ 📁 History   │  │ thumb │ title.mp4   ▆▆▆▆▆░ 62% │    │
│ ⚙ Settings   │  │ thumb │ clip_b.mp4  ▆▆▆▆▆▆ 99% │    │
│              │  └────────────────────────────────┘    │
│ ─── ad slot ─│                                        │
└──────────────┴────────────────────────────────────────┘
```

**Sidebar sections:** Active (count badge), History, Settings. A reserved
ad slot at the bottom-left (hidden until ads ship).

**Active view:** queue list with thumbnail, title, format badge, progress bar,
speed/ETA, cancel button. Per-item expand reveals format picker + destination.

**Detection drawer:** when URL is pasted/added, slide-in panel shows detected
formats with a default pre-selected. User confirms or tweaks, then hits
"Download". Folder defaults to global Settings path; per-item override via
folder icon.

**History view:** completed/failed items with re-download, "open in Finder",
remove. Persisted in SQLite (`tauri-plugin-sql`).

**Settings:**
- Default download folder
- Concurrent downloads (1–8)
- Theme (system / light / dark)
- Updates: auto-check toggle, channel (stable), "Check now" button, current version
- About: license, yt-dlp version, ffmpeg version

**Visual language:** light, neutral grays, single accent (black on white /
white on black depending on theme), rounded-md, 1px borders. Inter font.
No gradients or shadows beyond `shadow-sm`.

## 6. Auto-update

`@tauri-apps/plugin-updater` checks an endpoint on launch (and on demand from
Settings → "Check now").

- **Update endpoint:** `https://anyvideo.dev/updates/{target}/{current_version}`
  served from a static CDN (Cloudflare Pages). Returns `latest.json` with
  signature, version, notes, platform-specific URL. Hosted in repo until we
  buy a domain — fall back to `https://github.com/Devguru-J/Any-Video-Downloader/releases/latest/download/latest.json`.
- **Signing:** Tauri minisign keypair. Public key embedded in `tauri.conf.json`,
  private key in GitHub Actions secrets.
- **UX:** non-blocking banner on app launch when update available
  ("Update v1.2.3 ready — Restart"). One click = download + apply + relaunch.
  Settings page shows status: "Up to date" / "Update available" / "Downloading".

## 7. Force-update gate

Separate from the updater: a small policy check that can lock out old clients.

- On launch (and every 6 h while running), GET
  `https://anyvideo.dev/policy.json`:
  ```json
  { "min_required_version": "2.0.0", "message": "Please update to continue." }
  ```
- If `current_version < min_required_version`, replace the entire app surface
  with a blocking screen: title, message, "Update now" button (triggers
  updater), "Quit" button. No downloads can start.
- Cache the last-seen policy locally so a network outage doesn't lock out
  users who were already on a compliant version.
- When ads ship in v2.0, set `min_required_version = "2.0.0"`. Pre-2.0 clients
  are forced to update before they can use the app again.

This is intentionally a soft DRM: a determined user can patch the binary, but
99 %+ of normal users will simply update.

## 8. Distribution

GitHub Releases via `tauri-action`:

- **macOS:** signed `.dmg` (universal: `aarch64-apple-darwin` + `x86_64-apple-darwin`),
  notarized via `xcrun notarytool`. Apple Developer ID required.
- **Windows:** signed `.msi` + `.exe` (NSIS). Authenticode cert required.
- Each release publishes `latest.json` with signed update bundles.

Initial v0.1 ships unsigned for testing — user sees Gatekeeper / SmartScreen
warning. Code-signing certs added before v1.0.

## 9. Repo layout

```
any_video/
├── src/                       React + TS UI
│   ├── components/
│   ├── lib/                   IPC wrappers, types
│   ├── views/                 active.tsx, history.tsx, settings.tsx
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/          probe.rs, download.rs, etc.
│   │   ├── policy.rs          force-update gate
│   │   └── jobs.rs            queue, progress events
│   ├── binaries/              yt-dlp, ffmpeg per-target (gitignored, fetched in CI)
│   ├── tauri.conf.json
│   └── Cargo.toml
├── scripts/
│   └── fetch-binaries.sh      pull latest yt-dlp/ffmpeg per target
├── .github/workflows/
│   └── release.yml            tauri-action build + sign + publish
└── docs/superpowers/specs/
```

## 10. Risks & mitigations

| Risk | Mitigation |
|---|---|
| yt-dlp can't extract a given site | Show clear "Couldn't detect a video on this page" + log raw stderr in a "Details" expandable. v1.1: headless-webview fallback. |
| Site requires login/cookies | Settings → optional "Use cookies from browser X" (yt-dlp flag). |
| ffmpeg sidecar size (~30 MB) | Accept it. Pre-built static, per-arch. Both binaries gitignored, fetched at build time. |
| Apple notarization friction | Ship unsigned for v0.1 / pre-release; require signing only for stable channel. |
| Force-update lockout if our policy.json goes down | Local cache + grace window (already on compliant version → never locked). |
| Legal exposure (downloading copyrighted content) | App is general-purpose tool (yt-dlp is widely distributed). Ship a clear ToS / first-run notice that user is responsible for what they download. |

## 11. Milestones

1. **M1 — Skeleton (this PR):** Tauri scaffold, sidecar binaries fetched, `probe_url` working end-to-end on a sample URL, B-style empty UI.
2. **M2 — Download path:** queue, progress, history, folder picker, cancel.
3. **M3 — Polish:** settings, theme, error states, keyboard shortcuts (Cmd/Ctrl+V auto-detect).
4. **M4 — Release pipeline:** GH Actions, signing, updater endpoint, force-update gate.
5. **M5 — v1.0 launch:** signed builds for both platforms, public.
6. **M6 — Ads + v2.0 floor:** ad slot wired, `min_required_version` bump.

## 12. Open questions

- Domain for update / policy endpoints? (Falling back to GitHub raw for now.)
- Apple Developer ID and Windows EV cert — when to acquire?
- Ad provider for v2 (AdSense vs Carbon vs sponsor slot)?

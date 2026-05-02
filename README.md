# Any Video Downloader

A small cross-platform desktop app (macOS + Windows) that detects video on any
web page and downloads it to a folder you pick. Built on Tauri 2 + React +
Rust, powered by bundled `yt-dlp` and `ffmpeg`. Supports JW Player, HLS, DASH,
and generic HTML5 video.

## Status

`v0.1.0` — scaffold complete. Backend commands and UI wired; binaries vendored
at build time. Pre-release.

## Develop

Prereqs: Node 20+, Rust 1.77+, platform Tauri deps
(<https://v2.tauri.app/start/prerequisites/>).

```bash
npm install
bash scripts/fetch-binaries.sh        # macOS / Linux
# or: pwsh scripts/fetch-binaries.ps1  # Windows
npm run tauri dev
```

## Architecture

See `docs/superpowers/specs/2026-05-02-any-video-downloader-design.md`.

```
React UI (sidebar + queue)
   ↓ invoke
Rust commands ──► spawn yt-dlp/ffmpeg sidecars, parse progress, emit events
   ↓
Tauri updater  ──► GitHub Releases
Force-update gate ──► policy.json on CDN
```

## Release

Tag a version (`v0.2.0`) and push. GitHub Actions builds signed bundles for
macOS (arm64 + x64) and Windows (x64), publishing to Releases as a draft. Add
release notes and publish.

Required GitHub secrets:

- `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — minisign
  for updater bundles. Generate with `npm run tauri signer generate`.
- macOS code-signing/notarization: `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
  `APPLE_PASSWORD`, `APPLE_TEAM_ID`. (Optional for pre-1.0.)

## Force-update

The app polls `policy.json` from the latest GitHub release. To force users
onto a new floor (e.g., when ads ship in v2.0):

```json
{
  "min_required_version": "2.0.0",
  "message": "Please update to continue using Any Video Downloader."
}
```

Upload that file as a release asset on the **latest** tag. Older clients see a
blocking screen with an "Update now" button next time they launch.

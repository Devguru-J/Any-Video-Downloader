#!/usr/bin/env bash
# Fetch yt-dlp + ffmpeg per-target into src-tauri/binaries/.
# Tauri's externalBin expects files named `<base>-<target-triple>` in dev,
# and packaged builds rename them at bundle time.
set -euo pipefail

cd "$(dirname "$0")/.."
DEST="src-tauri/binaries"
mkdir -p "$DEST"

YTDLP_VERSION="${YTDLP_VERSION:-2026.03.17}"
YTDLP_BASE="https://github.com/yt-dlp/yt-dlp/releases/download/${YTDLP_VERSION}"

fetch() {
  local url="$1" out="$2"
  echo "  ↓ $url"
  curl -fL "$url" -o "$out"
  chmod +x "$out" 2>/dev/null || true
}

case "$(uname -s)/$(uname -m)" in
  Darwin/arm64)
    TRIPLE="aarch64-apple-darwin"
    fetch "$YTDLP_BASE/yt-dlp_macos" "$DEST/yt-dlp-$TRIPLE"
    # ffmpeg static (universal) from evermeet
    fetch "https://evermeet.cx/ffmpeg/getrelease/zip" "$DEST/ffmpeg.zip"
    unzip -o "$DEST/ffmpeg.zip" -d "$DEST" >/dev/null
    mv "$DEST/ffmpeg" "$DEST/ffmpeg-$TRIPLE"
    rm -f "$DEST/ffmpeg.zip"
    ;;
  Darwin/x86_64)
    TRIPLE="x86_64-apple-darwin"
    fetch "$YTDLP_BASE/yt-dlp_macos" "$DEST/yt-dlp-$TRIPLE"
    fetch "https://evermeet.cx/ffmpeg/getrelease/zip" "$DEST/ffmpeg.zip"
    unzip -o "$DEST/ffmpeg.zip" -d "$DEST" >/dev/null
    mv "$DEST/ffmpeg" "$DEST/ffmpeg-$TRIPLE"
    rm -f "$DEST/ffmpeg.zip"
    ;;
  Linux/x86_64)
    TRIPLE="x86_64-unknown-linux-gnu"
    fetch "$YTDLP_BASE/yt-dlp_linux" "$DEST/yt-dlp-$TRIPLE"
    # Linux ffmpeg static (johnvansickle build)
    fetch "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz" "$DEST/ffmpeg.tar.xz"
    tar -xf "$DEST/ffmpeg.tar.xz" -C "$DEST"
    mv "$DEST"/ffmpeg-*-amd64-static/ffmpeg "$DEST/ffmpeg-$TRIPLE"
    rm -rf "$DEST"/ffmpeg-*-amd64-static "$DEST/ffmpeg.tar.xz"
    ;;
  *)
    echo "Unsupported host: $(uname -s)/$(uname -m). Use the Windows .ps1 helper or run on the target host." >&2
    exit 1
    ;;
esac

echo "Done. Binaries:"
ls -lh "$DEST"

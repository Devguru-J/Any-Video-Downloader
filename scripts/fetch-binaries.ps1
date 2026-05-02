#!/usr/bin/env pwsh
# Fetch yt-dlp + ffmpeg for Windows x64 into src-tauri/binaries/.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$Dest = "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

$Triple = "x86_64-pc-windows-msvc"
$YtdlpVersion = if ($env:YTDLP_VERSION) { $env:YTDLP_VERSION } else { "2026.03.17" }
$YtdlpUrl = "https://github.com/yt-dlp/yt-dlp/releases/download/$YtdlpVersion/yt-dlp.exe"

Write-Host "  Downloading yt-dlp..."
Invoke-WebRequest -Uri $YtdlpUrl -OutFile "$Dest/yt-dlp-$Triple.exe"

Write-Host "  Downloading ffmpeg (gyan.dev essentials build)..."
$FfUrl = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
$FfZip = "$Dest/ffmpeg.zip"
Invoke-WebRequest -Uri $FfUrl -OutFile $FfZip
Expand-Archive -Path $FfZip -DestinationPath "$Dest/ff-extract" -Force
$FfExe = Get-ChildItem -Path "$Dest/ff-extract" -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
Move-Item -Force $FfExe.FullName "$Dest/ffmpeg-$Triple.exe"
Remove-Item -Recurse -Force "$Dest/ff-extract", $FfZip

Write-Host "Done. Binaries in $Dest:"
Get-ChildItem $Dest

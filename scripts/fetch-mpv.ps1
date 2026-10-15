<#
.SYNOPSIS
  Stages the mpv runtime binary for RexPaper.

.DESCRIPTION
  Live wallpapers are rendered by spawning mpv.exe, which is ~114 MB and is
  therefore NOT committed to git. This script downloads a pinned (latest)
  x86_64 mpv build and extracts mpv.exe into src-tauri/mpv/, which is both the
  dev staging location and the Tauri bundle resource path.

  Run this once before `npm run tauri build` on a fresh clone. CI runs it too.

.PARAMETER Url
  Override the download URL. Defaults to the latest shinchiro x86_64 build.

.PARAMETER Force
  Re-download even if src-tauri/mpv/mpv.exe already exists.

.NOTES
  Requires 7-Zip (7z.exe) to extract the .7z archive.
#>
[CmdletBinding()]
param(
  [string]$Url,
  [switch]$Force
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$destDir  = Join-Path $repoRoot 'src-tauri\mpv'
$destExe  = Join-Path $destDir  'mpv.exe'

if ((Test-Path $destExe) -and -not $Force) {
  Write-Host "mpv.exe already staged at $destExe (pass -Force to refresh)."
  exit 0
}

if (-not $Url) {
  Write-Host 'Resolving latest shinchiro mpv x86_64 build...'
  $release = Invoke-RestMethod `
    -Uri 'https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest' `
    -Headers @{ 'User-Agent' = 'rexpaper-fetch-mpv' }
  $asset = $release.assets |
    Where-Object { $_.name -match '^mpv-x86_64-\d{8}.*\.7z$' } |
    Select-Object -First 1
  if (-not $asset) { throw "No x86_64 mpv asset found in release $($release.tag_name)." }
  $Url = $asset.browser_download_url
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("rexpaper-mpv-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $tmp | Out-Null

try {
  $archive = Join-Path $tmp 'mpv.7z'
  Write-Host "Downloading $Url"
  Invoke-WebRequest -Uri $Url -OutFile $archive -UseBasicParsing

  $sevenZip = @(
    "$env:ProgramFiles\7-Zip\7z.exe",
    "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
  ) | Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $sevenZip) { $sevenZip = (Get-Command 7z -ErrorAction SilentlyContinue).Source }
  if (-not $sevenZip) { throw '7-Zip (7z.exe) not found. Install 7-Zip or pass a .zip URL with -Url.' }

  & $sevenZip x $archive "-o$tmp" -y | Out-Null

  $found = Get-ChildItem -Path $tmp -Recurse -Filter 'mpv.exe' | Select-Object -First 1
  if (-not $found) { throw 'mpv.exe not found inside the downloaded archive.' }

  New-Item -ItemType Directory -Force $destDir | Out-Null
  Copy-Item -Force $found.FullName $destExe
  $mb = [math]::Round((Get-Item $destExe).Length / 1MB, 1)
  Write-Host "Staged $destExe ($mb MB)"
}
finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

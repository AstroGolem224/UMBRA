# UMBRA Portable Packager
# Creates a portable folder with the exe + marker file so settings persist next to the binary.

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path "$projectRoot\src-tauri")) {
    $projectRoot = Split-Path -Parent $PSScriptRoot
}

$exePath = "$projectRoot\src-tauri\target\release\UMBRA.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "ERROR: Release binary not found at $exePath" -ForegroundColor Red
    Write-Host "Run 'npx tauri build' first." -ForegroundColor Yellow
    exit 1
}

$version = (Get-Content "$projectRoot\package.json" | ConvertFrom-Json).version
$outDir = "$projectRoot\dist\UMBRA-v${version}-portable"

if (Test-Path $outDir) { Remove-Item -Recurse -Force $outDir }
New-Item -ItemType Directory -Path $outDir | Out-Null

# Copy executable
Copy-Item $exePath "$outDir\UMBRA.exe"

# Create portable marker — triggers config storage next to exe
New-Item -ItemType File -Path "$outDir\portable" | Out-Null

# Copy WebView2 loader if present
$wv2 = "$projectRoot\src-tauri\target\release\WebView2Loader.dll"
if (Test-Path $wv2) { Copy-Item $wv2 $outDir }

Write-Host ""
Write-Host "Portable build ready:" -ForegroundColor Green
Write-Host "  $outDir" -ForegroundColor Cyan
Write-Host ""
Write-Host "Contents:" -ForegroundColor Gray
Get-ChildItem $outDir | ForEach-Object { Write-Host "  $($_.Name)" -ForegroundColor Gray }
Write-Host ""
Write-Host "First launch will create config.json next to UMBRA.exe." -ForegroundColor Yellow
Write-Host "All settings, tokens and database stay in this folder." -ForegroundColor Yellow

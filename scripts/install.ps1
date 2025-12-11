# Slides Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/OWNER/REPO/master/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

# CONFIGURE THIS: Set your GitHub repo
$Owner = "MichaelBrauner"
$Repo = "slides-rs"

$BinaryName = "slides.exe"
$InstallDir = "$env:LOCALAPPDATA\slides"
$GithubApi = "https://api.github.com/repos/$Owner/$Repo/releases/latest"

Write-Host "Installing slides..." -ForegroundColor Cyan

# Create install directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-Host "  Created directory: $InstallDir" -ForegroundColor Gray
}

# Get latest release info
Write-Host "  Fetching latest release..." -ForegroundColor Gray
try {
    $Release = Invoke-RestMethod -Uri $GithubApi -Headers @{ "User-Agent" = "slides-installer" }
    $LatestTag = $Release.tag_name
    Write-Host "  Latest version: $LatestTag" -ForegroundColor Gray
} catch {
    Write-Host "Error: Could not fetch latest release." -ForegroundColor Red
    Write-Host "Please check your internet connection or download manually from:" -ForegroundColor Yellow
    Write-Host "https://github.com/$Owner/$Repo/releases" -ForegroundColor Yellow
    exit 1
}

# Find Windows binary in assets
$Asset = $Release.assets | Where-Object { $_.name -like "*windows*" -and $_.name -like "*.exe" }
if (!$Asset) {
    Write-Host "Error: Windows binary not found in release." -ForegroundColor Red
    exit 1
}

$DownloadUrl = $Asset.browser_download_url
$DestPath = Join-Path $InstallDir $BinaryName

# Download binary
Write-Host "  Downloading slides..." -ForegroundColor Gray
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $DestPath -UseBasicParsing
} catch {
    Write-Host "Error: Download failed." -ForegroundColor Red
    exit 1
}

Write-Host "  Downloaded to: $DestPath" -ForegroundColor Gray

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "  Adding to PATH..." -ForegroundColor Gray
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "  Added $InstallDir to user PATH" -ForegroundColor Gray
}

# Verify installation
Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "IMPORTANT: Please restart your terminal (or open a new one)" -ForegroundColor Yellow
Write-Host "Then you can use: slides build" -ForegroundColor Cyan
Write-Host ""

# Try to run version check
try {
    $Version = & $DestPath --version 2>&1
    Write-Host "Installed: $Version" -ForegroundColor Gray
} catch {
    Write-Host "Binary installed at: $DestPath" -ForegroundColor Gray
}

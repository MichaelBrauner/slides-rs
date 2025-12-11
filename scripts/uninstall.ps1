# Slides Uninstall Script for Windows
# Usage: irm https://raw.githubusercontent.com/MichaelBrauner/slides-rs/main/scripts/uninstall.ps1 | iex

$ErrorActionPreference = "Stop"

$InstallDir = "$env:LOCALAPPDATA\slides"

Write-Host "Uninstalling slides..." -ForegroundColor Cyan

# Remove binary
if (Test-Path $InstallDir) {
    Remove-Item $InstallDir -Recurse -Force
    Write-Host "  Removed $InstallDir" -ForegroundColor Gray
} else {
    Write-Host "  Directory not found: $InstallDir" -ForegroundColor Yellow
}

# Remove from PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -like "*$InstallDir*") {
    $NewPath = ($UserPath -split ';' | Where-Object { $_ -ne $InstallDir }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "  Removed from PATH" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Uninstall complete!" -ForegroundColor Green

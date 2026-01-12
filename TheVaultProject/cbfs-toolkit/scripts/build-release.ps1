# ============================================================================
# CBFS Toolkit - Release Build & Packaging Script
# ============================================================================
# This script builds the release version, creates a distribution package,
# a desktop shortcut, and a zip file for deployment.
# ============================================================================

param(
    [switch]$SkipBuild,
    [string]$OutputDir = "$PSScriptRoot\..\dist"
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$AppName = "CBFSToolkit"
$ExeName = "cbfs-toolkit.exe"
$Version = "24.0.0"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  CBFS Toolkit Release Builder v$Version" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Release (if not skipped)
if (-not $SkipBuild) {
    Write-Host "[1/5] Building release..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    try {
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed with exit code $LASTEXITCODE"
        }
        Write-Host "      Build completed successfully!" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
} else {
    Write-Host "[1/5] Skipping build (using existing release)" -ForegroundColor Gray
}

# Step 2: Create distribution folder structure
Write-Host "[2/5] Creating distribution folder structure..." -ForegroundColor Yellow

$DistDir = $OutputDir
$AppDir = Join-Path $DistDir $AppName

# Clean and create directories
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}

New-Item -ItemType Directory -Force -Path $AppDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $AppDir "data") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $AppDir "config") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $AppDir "logs") | Out-Null

Write-Host "      Created folder structure:" -ForegroundColor Green
Write-Host "        $AppDir\" -ForegroundColor Gray
Write-Host "        $AppDir\data\" -ForegroundColor Gray
Write-Host "        $AppDir\config\" -ForegroundColor Gray
Write-Host "        $AppDir\logs\" -ForegroundColor Gray

# Step 3: Copy required files
Write-Host "[3/5] Copying required files..." -ForegroundColor Yellow

$ReleaseExe = Join-Path $ProjectRoot "target\release\$ExeName"
if (-not (Test-Path $ReleaseExe)) {
    throw "Release executable not found at: $ReleaseExe. Please run 'cargo build --release' first."
}

# Copy main executable
Copy-Item $ReleaseExe -Destination $AppDir
Write-Host "      Copied: $ExeName" -ForegroundColor Green

# Copy users.db if exists (optional - will be created on first run)
$UsersDb = Join-Path $ProjectRoot "users.db"
if (Test-Path $UsersDb) {
    Copy-Item $UsersDb -Destination (Join-Path $AppDir "data")
    Write-Host "      Copied: users.db -> data\" -ForegroundColor Green
}

# Copy client_secret.json if exists (for Google Drive integration)
$ClientSecret = Join-Path $ProjectRoot "client_secret.json"
if (Test-Path $ClientSecret) {
    Copy-Item $ClientSecret -Destination (Join-Path $AppDir "config")
    Write-Host "      Copied: client_secret.json -> config\" -ForegroundColor Green
}

# Create README for distribution
$ReadmeContent = @"
# Vault Console v$Version

## Quick Start
1. Run 'vault-console.exe' or use the desktop shortcut
2. Default admin credentials: admin / admin (change immediately!)

## Folder Structure
- vault-console.exe  : Main application
- data\              : Database files (users.db)
- config\            : Configuration files (client_secret.json for Google Drive)
- logs\              : Application logs

## Command Line Options
  vault-console.exe -i    : Start in interactive mode
  vault-console.exe -h    : Show help

## Google Drive Setup
Place your 'client_secret.json' in the config\ folder to enable Google Drive integration.

## Support
For issues, contact your system administrator.
"@

$ReadmeContent | Out-File -FilePath (Join-Path $AppDir "README.txt") -Encoding UTF8
Write-Host "      Created: README.txt" -ForegroundColor Green

# Step 4: Create desktop shortcut
Write-Host "[4/5] Creating desktop shortcut..." -ForegroundColor Yellow

$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $DesktopPath "Vault Console.lnk"
$TargetPath = Join-Path $AppDir $ExeName

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $TargetPath
$Shortcut.Arguments = "-i"
$Shortcut.WorkingDirectory = $AppDir
$Shortcut.Description = "Vault Console - Secure File Vault Manager"
$Shortcut.Save()

Write-Host "      Created shortcut: $ShortcutPath" -ForegroundColor Green

# Step 5: Create ZIP package
Write-Host "[5/5] Creating ZIP package..." -ForegroundColor Yellow

$ZipPath = Join-Path $DistDir "$AppName-v$Version.zip"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Compress-Archive -Path $AppDir -DestinationPath $ZipPath -CompressionLevel Optimal
$ZipSize = (Get-Item $ZipPath).Length / 1MB

Write-Host "      Created: $ZipPath" -ForegroundColor Green
Write-Host "      Size: $([math]::Round($ZipSize, 2)) MB" -ForegroundColor Gray

# Summary
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Distribution folder: $AppDir" -ForegroundColor White
Write-Host "ZIP package:         $ZipPath" -ForegroundColor White
Write-Host "Desktop shortcut:    $ShortcutPath" -ForegroundColor White
Write-Host ""
Write-Host "To deploy: Copy the ZIP or the $AppName folder to the target machine." -ForegroundColor Yellow

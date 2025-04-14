#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ============================================================
# Constants
# ============================================================
Set-Variable -Name APP_NAME                 -Value 'actiona-run'                                                         -Option Constant
Set-Variable -Name GITHUB_REPO              -Value 'Jmgr/actiona4'                                                       -Option Constant
Set-Variable -Name GITHUB_RELEASES_BASE_URL -Value "https://github.com/$GITHUB_REPO/releases/latest/download"            -Option Constant
Set-Variable -Name SUPPORTED_ARCH           -Value 'AMD64'                                                               -Option Constant  # AMD64 = x86_64 on Windows
Set-Variable -Name TARGET_TRIPLE            -Value 'x86_64-pc-windows-msvc'                                              -Option Constant
Set-Variable -Name ARCHIVE_FILENAME         -Value "$TARGET_TRIPLE.zip"                                                  -Option Constant
Set-Variable -Name DOWNLOAD_URL             -Value "$GITHUB_RELEASES_BASE_URL/$ARCHIVE_FILENAME"                         -Option Constant
Set-Variable -Name DEFAULT_INSTALL_DIR      -Value (Join-Path $env:LOCALAPPDATA 'actiona-run')                           -Option Constant

# ============================================================
# Output helpers
# ============================================================
function Write-Info([string]$Message) { Write-Host $Message -ForegroundColor Green }
# Exit-WithError is used instead of Write-Error to avoid shadowing the built-in cmdlet
function Exit-WithError([string]$Message) { Write-Host "Error: $Message" -ForegroundColor Red; exit 1 }

# ============================================================
# Platform checks
# RuntimeInformation works on both Windows PowerShell 5.1 and PowerShell 7+
# ============================================================
$IsOSWin = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)

if (-not $IsOSWin) {
    $CurrentOS = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
    Exit-WithError "Unsupported operating system: $CurrentOS. Only Windows is supported."
}

# PROCESSOR_ARCHITECTURE is AMD64 for x86_64 on Windows
$CurrentArch = $env:PROCESSOR_ARCHITECTURE
if ($CurrentArch -ne $SUPPORTED_ARCH) {
    Exit-WithError "Unsupported architecture: $CurrentArch. Only $SUPPORTED_ARCH (x86_64) is currently supported."
}

# ============================================================
# Determine install directory
# Set ACTIONA_INSTALL_DIR in the environment to override the default.
# ============================================================
$InstallDir = if ($env:ACTIONA_INSTALL_DIR) { $env:ACTIONA_INSTALL_DIR } else { $DEFAULT_INSTALL_DIR }
Write-Info "Installing $APP_NAME to: $InstallDir"

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

# ============================================================
# Download archive to a temporary directory
# The temp dir is cleaned up in the finally block (success or failure).
# ============================================================
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    $TempArchive = Join-Path $TempDir $ARCHIVE_FILENAME

    Write-Info "Downloading $ARCHIVE_FILENAME ..."
    # Suppress the default progress bar — it dramatically slows Invoke-WebRequest
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $TempArchive
    $ProgressPreference = 'Continue'

    # ============================================================
    # Extract the archive into the install directory
    # ============================================================
    Write-Info "Extracting to $InstallDir ..."
    Expand-Archive -Path $TempArchive -DestinationPath $InstallDir -Force

    # ============================================================
    # Run the first-time setup wizard (configures update checks, telemetry, etc.)
    # ============================================================
    Write-Info "Running initial setup ..."
    & (Join-Path $InstallDir "$APP_NAME.exe") setup

    Write-Info "Installation complete."
} finally {
    # Clean up temp directory regardless of success or failure
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}

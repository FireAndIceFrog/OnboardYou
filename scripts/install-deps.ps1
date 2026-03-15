<#
.SYNOPSIS
Installs system dependencies needed to build and run OnboardYou.

DESCRIPTION
- On Windows (native PowerShell), this script uses Winget to install the required tooling.
- On WSL, it delegates to the existing Bash script at `./scripts/install-deps.sh`.

USAGE (Windows):
  Open PowerShell as Administrator and run:
    ./scripts/install-deps.ps1

USAGE (WSL):
  In WSL, run:
    sudo ./scripts/install-deps.sh
#>

[CmdletBinding()]
param(
    [switch]$Force
)

function Write-Info($msg) {
    Write-Host "[INFO] $msg" -ForegroundColor Cyan
}

function Write-Warn($msg) {
    Write-Host "[WARN] $msg" -ForegroundColor Yellow
}

function Write-ErrorAndExit($msg) {
    Write-Host "[ERROR] $msg" -ForegroundColor Red
    exit 1
}

# Detect WSL (if running under WSL, defer to the Bash installer)
if ($env:WSL_DISTRO_NAME) {
    Write-Info "Detected WSL distro: $($env:WSL_DISTRO_NAME). Delegating to bash installer."
    bash -lc "cd '$(pwd)' && sudo ./scripts/install-deps.sh"
    exit $LASTEXITCODE
}

# Ensure we are running as Administrator
if (-not ([bool](New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator))) {
    Write-Warn "This script should be run as Administrator to install system packages."
    Write-Warn "Re-run in an elevated PowerShell session."
    exit 1
}

# Ensure winget is available
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-ErrorAndExit "winget is not available. Please install App Installer / winget: https://learn.microsoft.com/windows/package-manager/winget/"
}

function Install-WingetPackage {
    param(
        [string]$Id,
        [string]$Name
    )

    $installed = winget list --id $Id 2>$null | Select-String -Pattern $Id
    if ($installed) {
        Write-Info "$Name already installed."
        return
    }

    Write-Info "Installing $Name..."
    winget install --id $Id --accept-package-agreements --accept-source-agreements --silent
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "Failed to install $Name (winget exit code $LASTEXITCODE). You may need to install it manually."
    }
}

$packages = @(
    @{Id = 'Git.Git'; Name = 'Git'},
    @{Id = 'OpenJS.NodeJS.LTS'; Name = 'Node.js LTS'},
    @{Id = 'Python.Python.3'; Name = 'Python 3'},
    @{Id = 'RustLang.Rust'; Name = 'Rust'},
    @{Id = 'PostgreSQL.PostgreSQL'; Name = 'PostgreSQL (including psql)'},
    @{Id = 'Amazon.AWSCLI'; Name = 'AWS CLI'},
    @{Id = 'GnuWin32.Make'; Name = 'make'}
)

foreach ($pkg in $packages) {
    Install-WingetPackage -Id $pkg.Id -Name $pkg.Name
}

Write-Info "✅ Dependency installation complete."
Write-Host "Next steps:`n  1) cd $(Get-Location)`n  2) make setup`n  3) make deploy (or other Makefile targets as needed)"

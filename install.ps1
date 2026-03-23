# install.ps1 — Install aip (AI Providers) from Gitea Releases
# Usage:
#   irm https://gitea.lan.wiqun.com/Albert/ai-providers/raw/branch/main/install.ps1 | iex
#   $env:VERSION="1.1.0"; irm ... | iex
#   $env:UNINSTALL="1"; irm ... | iex
#   .\install.ps1 -Version 1.1.0
#   .\install.ps1 -Uninstall

param(
    [string]$Version = $env:VERSION,
    [string]$InstallDir = $env:INSTALL_DIR,
    [switch]$Uninstall = ($env:UNINSTALL -eq "1")
)

$ErrorActionPreference = "Stop"

$GiteaBase = "https://gitea.lan.wiqun.com"
$Repo = "Albert/ai-providers"
$Target = "x86_64-pc-windows-gnu"
$DefaultInstallDir = Join-Path $env:LOCALAPPDATA "aip"

function Say($msg) {
    Write-Host "aip: $msg"
}

function Err($msg) {
    Write-Host "aip: ERROR: $msg" -ForegroundColor Red
    exit 1
}

function Get-LatestVersion {
    # Try /releases/latest
    try {
        $response = Invoke-RestMethod -Uri "$GiteaBase/api/v1/repos/$Repo/releases/latest" -UseBasicParsing
        $tag = $response.tag_name
        if ($tag) {
            return $tag -replace '^v', ''
        }
    } catch {}

    # Fallback: list releases
    try {
        $response = Invoke-RestMethod -Uri "$GiteaBase/api/v1/repos/$Repo/releases?limit=1" -UseBasicParsing
        if ($response.Count -gt 0) {
            $tag = $response[0].tag_name
            if ($tag) {
                return $tag -replace '^v', ''
            }
        }
    } catch {}

    Err "could not determine latest version from Gitea API"
}

function Configure-Path($dir) {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $paths = $userPath -split ';' | Where-Object { $_ -ne '' }

    if ($paths -contains $dir) {
        Say "PATH already contains $dir"
        return
    }

    $newPath = ($paths + $dir) -join ';'
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

    # Also update current session
    if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $dir })) {
        $env:PATH = "$dir;$env:PATH"
    }

    Say "added $dir to user PATH"
    Say "new terminals will have aip available automatically"
}

function Remove-PathEntry($dir) {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $paths = $userPath -split ';' | Where-Object { $_ -ne '' -and $_ -ne $dir }
    $newPath = $paths -join ';'
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

    # Also update current session
    $sessionPaths = $env:PATH -split ';' | Where-Object { $_ -ne $dir }
    $env:PATH = $sessionPaths -join ';'

    Say "removed $dir from user PATH"
}

function Do-Install {
    if (-not $InstallDir) {
        $InstallDir = $DefaultInstallDir
    }

    if (-not $Version) {
        $Version = Get-LatestVersion
    }

    Say "installing aip v$Version ($Target)"

    $url = "$GiteaBase/$Repo/releases/download/v$Version/aip-v$Version-$Target.exe"
    $binPath = Join-Path $InstallDir "aip.exe"

    Say "downloading from $url"

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Download
    try {
        Invoke-WebRequest -Uri $url -OutFile $binPath -UseBasicParsing
    } catch {
        Err "download failed — check that version v$Version exists"
    }

    # Verify
    try {
        $versionOutput = & $binPath --version 2>&1
        Say "installed $versionOutput to $binPath"
    } catch {
        Say "installed to $binPath (could not verify version)"
    }

    Configure-Path $InstallDir
}

function Do-Uninstall {
    if (-not $InstallDir) {
        $InstallDir = $DefaultInstallDir
    }

    $binPath = Join-Path $InstallDir "aip.exe"

    if (Test-Path $binPath) {
        Remove-Item $binPath -Force
        Say "removed $binPath"
    } else {
        Say "$binPath not found, nothing to remove"
    }

    # Clean up empty directory
    if ((Test-Path $InstallDir) -and (Get-ChildItem $InstallDir | Measure-Object).Count -eq 0) {
        Remove-Item $InstallDir -Force
        Say "removed empty directory $InstallDir"
    }

    Remove-PathEntry $InstallDir
    Say "uninstall complete"
}

# Main
if ($Uninstall) {
    Do-Uninstall
} else {
    Do-Install
}

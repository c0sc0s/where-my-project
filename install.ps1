param(
    [string]$Repo,
    [string]$Version = "latest",
    [string]$InstallDir = "$HOME\bin",
    [switch]$Force
)

function Get-NormalizedVersion {
    param([string]$Value)

    if (-not $Value) {
        return $null
    }

    return $Value.Trim().TrimStart("v", "V")
}

function Get-InstalledProjVersion {
    param([string]$BinaryPath)

    if (-not (Test-Path $BinaryPath)) {
        return $null
    }

    try {
        $versionOutput = & $BinaryPath --version 2>$null
        if (-not $versionOutput) {
            return $null
        }

        $firstLine = ($versionOutput | Select-Object -First 1).ToString().Trim()
        if ($firstLine -match "([0-9]+\.[0-9]+\.[0-9]+)") {
            return $Matches[1]
        }
    } catch {
        return $null
    }

    return $null
}

function Install-Proj {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Repo,

        [string]$Version = "latest",

        [string]$InstallDir = "$HOME\bin",

        [switch]$Force
    )

    Set-StrictMode -Version Latest
    $ErrorActionPreference = "Stop"

    if (-not (Get-Command Invoke-RestMethod -ErrorAction SilentlyContinue)) {
        throw "Invoke-RestMethod is required."
    }

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $releaseApi = if ($Version -eq "latest") {
        "https://api.github.com/repos/$Repo/releases/latest"
    } else {
        "https://api.github.com/repos/$Repo/releases/tags/$Version"
    }

    Write-Host "Fetching release metadata from $releaseApi" -ForegroundColor Cyan
    $release = Invoke-RestMethod -Uri $releaseApi -Headers @{ "User-Agent" = "proj-installer" }
    $releaseVersion = Get-NormalizedVersion $release.tag_name

    $asset = $release.assets | Where-Object { $_.name -eq "proj-windows-x86_64.zip" } | Select-Object -First 1
    if (-not $asset) {
        throw "Release asset 'proj-windows-x86_64.zip' not found."
    }

    $destination = Join-Path $InstallDir "proj.exe"
    $installedVersion = Get-InstalledProjVersion -BinaryPath $destination
    if (-not $Force -and $installedVersion -and $releaseVersion -and $installedVersion -eq $releaseVersion) {
        Write-Host "proj $installedVersion is already installed." -ForegroundColor Green
        Write-Host "Use -Force to reinstall the same version." -ForegroundColor Yellow
        return
    }

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("proj-install-" + [Guid]::NewGuid().ToString("N"))
    $zipPath = Join-Path $tempDir $asset.name
    $extractDir = Join-Path $tempDir "extract"

    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    New-Item -ItemType Directory -Path $extractDir -Force | Out-Null

    try {
        Write-Host "Downloading $($asset.name)" -ForegroundColor Cyan
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -Headers @{ "User-Agent" = "proj-installer" }

        Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

        $binary = Get-ChildItem -Path $extractDir -Filter "proj.exe" -Recurse | Select-Object -First 1
        if (-not $binary) {
            throw "proj.exe was not found in the downloaded archive."
        }

        Copy-Item -LiteralPath $binary.FullName -Destination $destination -Force

        $profilePath = $PROFILE.CurrentUserCurrentHost
        $profileDir = Split-Path $profilePath -Parent
        if (-not (Test-Path $profileDir)) {
            New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
        }
        if (-not (Test-Path $profilePath)) {
            New-Item -ItemType File -Path $profilePath -Force | Out-Null
        }

        $profileContent = Get-Content $profilePath -Raw
        $marker = "proj init | Out-String | Invoke-Expression"
        if ($profileContent -notmatch [regex]::Escape($marker)) {
            Add-Content -Path $profilePath -Value ""
            Add-Content -Path $profilePath -Value "# proj shell integration"
            Add-Content -Path $profilePath -Value '$binPath = "$HOME\bin"'
            Add-Content -Path $profilePath -Value 'if ($env:PATH -notlike "*$binPath*") { $env:PATH = "$binPath;$env:PATH" }'
            Add-Content -Path $profilePath -Value $marker
        }

        $installedLabel = if ($releaseVersion) { "proj $releaseVersion" } else { "proj" }
        Write-Host "Installed $installedLabel to $destination" -ForegroundColor Green
        Write-Host "Reload your profile with: . `$PROFILE" -ForegroundColor Yellow
    } finally {
        Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($MyInvocation.InvocationName -ne ".") {
    if (-not $Repo) {
        throw "Usage: .\install.ps1 -Repo <owner/repo> [-Version latest|vX.Y.Z] [-InstallDir <path>] [-Force]"
    }

    Install-Proj -Repo $Repo -Version $Version -InstallDir $InstallDir -Force:$Force
}

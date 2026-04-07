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

function Get-ReleaseTag {
    param([string]$Version)

    if (-not $Version -or $Version -eq "latest") {
        return $null
    }

    $trimmedVersion = $Version.Trim()
    if ($trimmedVersion.StartsWith("v", [System.StringComparison]::OrdinalIgnoreCase)) {
        return $trimmedVersion
    }

    return "v$trimmedVersion"
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

function Get-RedirectLocation {
    param([string]$Uri)

    $request = [System.Net.HttpWebRequest]::Create($Uri)
    $request.Method = "HEAD"
    $request.AllowAutoRedirect = $false
    $request.UserAgent = "proj-installer"

    $response = $null

    try {
        $response = $request.GetResponse()
    } catch [System.Net.WebException] {
        if (-not $_.Exception.Response) {
            throw
        }

        $response = $_.Exception.Response
    }

    try {
        $location = $response.Headers["Location"]
        if (-not $location) {
            throw "Could not resolve redirect location for $Uri."
        }

        return $location
    } finally {
        $response.Close()
    }
}

function Get-ReleaseInfo {
    param(
        [string]$Repo,
        [string]$Version
    )

    $assetName = "proj-windows-x86_64.zip"

    if ($Version -eq "latest") {
        $latestReleaseUrl = "https://github.com/$Repo/releases/latest"
        Write-Host "Resolving latest release from $latestReleaseUrl" -ForegroundColor Cyan

        $location = Get-RedirectLocation -Uri $latestReleaseUrl
        $resolvedUri = [System.Uri]::new([System.Uri]$latestReleaseUrl, $location)

        if ($resolvedUri.AbsolutePath -notmatch "/releases/tag/(?<tag>[^/]+)$") {
            throw "Could not determine the latest release tag from $latestReleaseUrl."
        }

        $releaseTag = $Matches["tag"]

        return [PSCustomObject]@{
            Tag         = $releaseTag
            Version     = Get-NormalizedVersion $releaseTag
            AssetName   = $assetName
            DownloadUrl = "https://github.com/$Repo/releases/latest/download/$assetName"
        }
    }

    $releaseTag = Get-ReleaseTag -Version $Version

    return [PSCustomObject]@{
        Tag         = $releaseTag
        Version     = Get-NormalizedVersion $releaseTag
        AssetName   = $assetName
        DownloadUrl = "https://github.com/$Repo/releases/download/$releaseTag/$assetName"
    }
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

    $release = Get-ReleaseInfo -Repo $Repo -Version $Version
    $releaseVersion = $release.Version

    $destination = Join-Path $InstallDir "proj.exe"
    $installedVersion = Get-InstalledProjVersion -BinaryPath $destination
    if (-not $Force -and $installedVersion -and $releaseVersion -and $installedVersion -eq $releaseVersion) {
        Write-Host "proj $installedVersion is already installed." -ForegroundColor Green
        Write-Host "Use -Force to reinstall the same version." -ForegroundColor Yellow
        return
    }

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("proj-install-" + [Guid]::NewGuid().ToString("N"))
    $zipPath = Join-Path $tempDir $release.AssetName
    $extractDir = Join-Path $tempDir "extract"

    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    New-Item -ItemType Directory -Path $extractDir -Force | Out-Null

    try {
        Write-Host "Downloading $($release.AssetName)" -ForegroundColor Cyan
        Invoke-WebRequest -Uri $release.DownloadUrl -OutFile $zipPath -Headers @{ "User-Agent" = "proj-installer" }

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

        $profileContent = [string](Get-Content $profilePath -Raw)
        $marker = "proj init | Out-String | Invoke-Expression"
        $activeMarkerPattern = "(?m)^[ \t]*(?!#)" + [regex]::Escape($marker) + "[ \t]*$"
        if ($profileContent -notmatch $activeMarkerPattern) {
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

function Uninstall-Proj {
    [CmdletBinding()]
    param(
        [string]$InstallDir = "$HOME\bin"
    )

    $ErrorActionPreference = "Stop"

    # Remove binary
    $binary = Join-Path $InstallDir "proj.exe"
    if (Test-Path $binary) {
        Remove-Item -LiteralPath $binary -Force
        Write-Host "Removed $binary" -ForegroundColor Green
    } else {
        Write-Host "proj.exe not found at $binary" -ForegroundColor Yellow
    }

    # Remove data file
    $dataFile = "$HOME\.proj.json"
    if (Test-Path $dataFile) {
        Remove-Item -LiteralPath $dataFile -Force
        Write-Host "Removed $dataFile" -ForegroundColor Green
    }

    # Remove profile integration block
    $profilePath = $PROFILE.CurrentUserCurrentHost
    if (Test-Path $profilePath) {
        $lines = Get-Content $profilePath
        $startIndex = -1
        $endIndex = -1

        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -match '^\s*#\s*(proj shell integration|>>>\s*proj)') {
                $startIndex = $i
            }
            if ($startIndex -ge 0 -and $lines[$i] -match 'proj init \| Out-String \| Invoke-Expression') {
                $endIndex = $i
                break
            }
        }

        if ($startIndex -ge 0 -and $endIndex -ge $startIndex) {
            # Trim one leading blank line before the block if present
            if ($startIndex -gt 0 -and $lines[$startIndex - 1].Trim() -eq '') {
                $startIndex--
            }
            $kept = @($lines[0..($startIndex - 1)]) + @($lines[($endIndex + 1)..($lines.Count - 1)])
            Set-Content -Path $profilePath -Value $kept
            Write-Host "Removed proj integration from $profilePath" -ForegroundColor Green
        } else {
            Write-Host "No proj integration found in profile" -ForegroundColor Yellow
        }
    }

    Write-Host "Uninstall complete. Reload your profile with: . `$PROFILE" -ForegroundColor Cyan
}

if ($PSCommandPath -and $MyInvocation.InvocationName -ne ".") {
    if (-not $Repo) {
        throw "Usage: .\install.ps1 -Repo <owner/repo> [-Version latest|vX.Y.Z] [-InstallDir <path>] [-Force]"
    }

    Install-Proj -Repo $Repo -Version $Version -InstallDir $InstallDir -Force:$Force
}

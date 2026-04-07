# proj

`proj` is a small Rust CLI/TUI tool for managing multiple local clones of the same Git repository.

It helps you:

- scan directories for projects you care about
- inspect Git status across clones
- jump into a selected clone from PowerShell

## Features

- `proj scan <path>`: scan directories directly and persist discovered projects
- `proj status`: inspect one or all tracked clones
- `proj list`: interactive TUI picker
- `proj cd`: print a resolved path for shell integration
- `proj init`: print PowerShell integration functions

## Install

### One-Line Install Or Upgrade

Users only need one command:

```powershell
irm https://raw.githubusercontent.com/c0sc0s/where-my-project/main/install.ps1 | iex; Install-Proj -Repo "c0sc0s/where-my-project"
```

That command:

- installs `proj` if it is missing
- upgrades to the latest GitHub Release if a newer version exists
- skips download when the installed version already matches the latest release
- ensures PowerShell profile integration is present

Run the same command again later to upgrade.

### Uninstall

```powershell
irm https://raw.githubusercontent.com/c0sc0s/where-my-project/main/install.ps1 | iex; Uninstall-Proj
```

Removes `proj.exe`, `~/.proj.json`, the profile integration block, and the current PowerShell session aliases/functions. If the install directory becomes empty, it is removed too.

### Option 1: GitHub Release

Download the latest release asset, extract `proj.exe`, and put it in `~/bin`.

Then add this to your PowerShell profile:

```powershell
$binPath = "$HOME\bin"
if ($env:PATH -notlike "*$binPath*") {
    $env:PATH = "$binPath;$env:PATH"
}

proj init | Out-String | Invoke-Expression
```

Reload your profile:

```powershell
. $PROFILE
```

### Option 2: Install Script

Once this repository is on GitHub, you can use the included installer:

```powershell
.\install.ps1 -Repo "c0sc0s/where-my-project"
```

Or directly from GitHub:

```powershell
irm https://raw.githubusercontent.com/c0sc0s/where-my-project/main/install.ps1 | iex
Install-Proj -Repo "c0sc0s/where-my-project"
```

## Build From Source

```powershell
cd proj
cargo build --release
```

The binary will be created at:

`proj/target/release/proj.exe`

## Usage

**Quick Start:**

```powershell
# Scan one or more directories directly
proj scan D:\code C:\work

# Or search all local workspace clones by project name
proj scan tiktok_live_studio

# Open TUI picker (no command needed!)
proj

# Or use the shell alias
pl
```

**Jump to projects:**

```powershell
pcd my-repo        # by repository name when unique
pcd 1              # by index
pcd C:\work\my-repo # by full path
```

**Other commands:**

```powershell
proj status        # Show all projects
proj status my-repo
proj cd 1 --raw
```

## Release

This repository includes a GitHub Actions workflow that:

- builds `proj` on Windows
- packages `proj.exe` with `README.md` and `install.ps1`
- creates a GitHub Release when you push a tag like `v0.2.0`

Release flow:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

Version source of truth is `proj/Cargo.toml`.
Before tagging a new release, bump the crate version there first.

Repository:

- SSH: `git@github.com:c0sc0s/where-my-project.git`
- HTTPS: `https://github.com/c0sc0s/where-my-project`

## Repository Layout

- `README.md`
- `install.ps1`
- `proj/Cargo.toml`
- `.github/workflows/release.yml`

# proj

`proj` is a small Rust CLI/TUI tool for managing multiple local clones of the same Git repository.

It helps you:

- watch repository names you care about
- scan directories for matching clones
- assign stable aliases
- inspect Git status across clones
- jump into a selected clone from PowerShell

## Features

- `proj watch`: manage watched repository names
- `proj scan`: scan directories and persist discovered clones
- `proj alias`: assign aliases to discovered projects
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

Removes `proj.exe`, `~/.proj.json`, and the profile integration block.

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
# Scan and auto-generate smart aliases (parent-dir + repo-name)
proj scan --paths D:\code,C:\work --auto-alias

# Open TUI picker (no command needed!)
proj

# Or use the shell alias
pl
```

**Jump to projects:**

```powershell
pcd work-myrepo    # by alias
pcd 1              # by index
```

**Other commands:**

```powershell
proj watch <repo>  # Watch specific repos
proj status        # Show all projects
proj alias 1 name  # Set custom alias
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

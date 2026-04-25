# Installation

This guide covers installing Clodhopper from a downloaded build artifact and preparing provider CLIs.

The preferred user install path is:

1. GitHub Actions or Release builds the binary.
2. You download the package for your platform.
3. You extract it outside the source repo.
4. You run the included installer.
5. The installer copies `harness` into a normal bin directory such as `~/.local/bin`.

This keeps installation separate from the directory where Clodhopper is developed.

## Requirements

Required for downloaded packages:

- A shell for your platform.
- Any provider CLIs you want the harness to use.

Optional provider tools:

- `codex` for Codex.
- `claude` for Claude Code.
- `gh` with Copilot support for GitHub Copilot.

Required only for source development:

- Rust and Cargo.
- Git.

## Download a Build

For day-to-day use, download one of these artifacts from GitHub Actions or GitHub Releases:

- `clodhopper-linux-x86_64.tar.gz`
- `clodhopper-macos-aarch64.tar.gz`
- `clodhopper-windows-x86_64.zip`

GitHub Actions artifacts are produced by the `Build` workflow. Release assets are produced when a `v*` tag is pushed or the release workflow is run manually.

## Install on macOS or Linux

Extract the package:

```bash
tar -xzf clodhopper-macos-aarch64.tar.gz
cd clodhopper-macos-aarch64
```

Run the installer:

```bash
./install.sh
```

Install to a custom directory:

```bash
./install.sh --dir "$HOME/bin"
```

Or:

```bash
INSTALL_DIR="$HOME/bin" ./install.sh
```

The installer:

- expects the binary at `bin/harness` inside the extracted package
- creates the install directory
- copies the binary to `<install-dir>/harness`
- marks it executable
- verifies `harness --help`
- warns when the install directory is not on `PATH`

## Install on Windows

Extract the package:

```powershell
Expand-Archive .\clodhopper-windows-x86_64.zip
cd .\clodhopper-windows-x86_64\clodhopper-windows-x86_64
```

Run the installer:

```powershell
.\install.ps1
```

Install to a custom directory:

```powershell
.\install.ps1 -InstallDir "$HOME\bin"
```

The installer:

- expects the binary at `bin\harness.exe`
- creates the install directory
- copies the binary to `<install-dir>\harness.exe`
- verifies `harness.exe --help`
- warns when the install directory is not on `PATH`

## Verify Installation

After installation:

```bash
harness --help
harness doctor
```

Expected result:

- Installed providers show `available=true`.
- Logged-in providers show `authenticated=true`.
- Providers that pass smoke checks show `ready=true`.

If a provider is present but unauthenticated:

```bash
harness doctor --prompt-login
```

## Initialize a Workspace

From the repo or project you want Clodhopper to manage:

```bash
harness init
```

This creates `.harness/config.toml` and `.harness/state/logs`.

If the config already exists, `harness init` exits without overwriting it. Edit `.harness/config.toml` directly for local changes.

## Build From Source for Development

Use source builds only when developing Clodhopper itself:

```bash
cargo build
cargo run -- doctor
```

The old `cargo install --path .` style is intentionally not the primary install path because it installs directly from the working tree. Use GitHub artifacts or release packages for normal use.

## Troubleshooting Install Issues

- `harness: command not found`: add the install directory, commonly `~/.local/bin`, to `PATH`.
- `expected release binary`: run the installer from the extracted package root.
- `command not found on PATH`: install the missing provider CLI or update the provider `command` in `.harness/config.toml`.
- `authenticated=false`: run `harness doctor --prompt-login`.
- `ready=false` with `authenticated=true`: inspect the provider's `smoke_check` in `harness providers doctor <provider>`.
- `config already exists`: edit `.harness/config.toml`; Clodhopper will not overwrite it automatically.


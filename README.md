# Clodhopper AI Agent Harness

Clodhopper is a Rust-based harness for moving between local AI coding tools: Codex, Claude Code, and GitHub Copilot. It provides a CLI-first workflow plus a local web console for checking provider readiness, switching orchestrators, launching configured agents, and preserving logs in a shared workspace.

The first version is intentionally local and process-backed. Cloud systems such as Azure AI Foundry, Google AI Studio, and hosted agent runners are documented as extension points, not active v1 execution targets.

## What It Does Today

- Detects Codex, Claude Code, and GitHub Copilot CLI tooling.
- Checks whether each provider is installed, authenticated, and ready for basic automation.
- Provides interactive login handoff through `harness doctor --prompt-login`.
- Loads repo-local config from `.harness/config.toml`.
- Runs named profiles through `harness run --profile <name>`.
- Spawns named agents through `harness spawn --agent <name>`.
- Captures process stdout, stderr, exit status, and metadata under `.harness/state/logs`.
- Serves a local Axum web console through `harness web`.

## Prerequisites

- Any provider CLIs you want the harness to use:
  - Codex CLI: `codex`
  - Claude Code CLI: `claude`
  - GitHub CLI with Copilot support: `gh copilot`

Rust and Git are only required when building from source. Downloaded release packages include a prebuilt `harness` binary.

The harness can run without every provider installed. Missing tools are reported by `harness doctor`.

## Install From a GitHub Build

The preferred install path is a downloaded GitHub Actions or GitHub Release artifact. This keeps the installed harness separate from the source tree.

1. Download the package for your platform:
   - `clodhopper-linux-x86_64.tar.gz`
   - `clodhopper-macos-aarch64.tar.gz`
   - `clodhopper-windows-x86_64.zip`
2. Extract the archive outside the repo.
3. Run the installer from the extracted folder.

macOS/Linux:

```bash
tar -xzf clodhopper-macos-aarch64.tar.gz
cd clodhopper-macos-aarch64
./install.sh
```

Windows PowerShell:

```powershell
Expand-Archive .\clodhopper-windows-x86_64.zip
cd .\clodhopper-windows-x86_64\clodhopper-windows-x86_64
.\install.ps1
```

By default, installers copy the binary to `~/.local/bin`. Use a custom directory when needed:

```bash
./install.sh --dir "$HOME/bin"
```

```powershell
.\install.ps1 -InstallDir "$HOME\bin"
```

Confirm the installed binary:

```bash
harness --help
harness doctor
```

## Build From Source for Development

Use source builds when changing the harness itself:

```bash
cargo build
cargo run -- doctor
```

Avoid installing from the working tree for normal use. The GitHub artifact installer is the supported user install path.

## First-Time Setup

Create the repo-local harness config and state directories:

```bash
harness init
```

This creates:

- `.harness/config.toml`
- `.harness/state/`
- `.harness/state/logs/`

The `.harness/` directory is ignored by git so local credentials, logs, and machine-specific config do not get committed.

Check installed tools and authentication:

```bash
harness doctor
```

If a provider is installed but not authenticated, start the relevant interactive login flow:

```bash
harness doctor --prompt-login
```

Provider login commands used by the harness:

- Codex: `codex login`
- Claude Code: `claude auth login`
- Copilot: `gh auth login`

## Quickstart

```bash
harness init
harness doctor
harness providers list
harness run --profile codex-default --dry-run
harness spawn --agent reviewer --dry-run
harness web
```

Open the web console at:

```text
http://127.0.0.1:4317
```

Use `--dry-run` first. It prints the command Clodhopper would execute without starting an interactive AI process.

## Readiness Checks

`harness doctor` reports one line per provider:

```text
codex (codex) command=codex available=true authenticated=true ready=true version=...
claude (claude-code) command=claude available=true authenticated=true ready=true version=...
copilot (copilot) command=gh available=true authenticated=true ready=true version=...
```

The checks are intentionally lightweight and non-interactive:

- Codex: `codex login status`, `codex exec --help`
- Claude Code: `claude auth status`, `claude --help`
- Copilot: `gh auth status`, `gh copilot -- --help`

Detailed JSON is available per provider:

```bash
harness providers doctor codex
harness providers doctor claude
harness providers doctor copilot
```

## Default Config

`harness init` writes a starter `.harness/config.toml` with three providers, two agents, and three run profiles:

```toml
[workspace]
root = "."
state_dir = ".harness/state"
log_dir = ".harness/state/logs"

[[providers]]
name = "codex"
kind = "codex"
command = "codex"
default_args = []

[[providers]]
name = "claude"
kind = "claude-code"
command = "claude"
default_args = []

[[providers]]
name = "copilot"
kind = "copilot"
command = "gh"
default_args = []

[[agents]]
name = "reviewer"
provider = "codex"
role = "Review changes for correctness, tests, and maintainability."
model = "default"
args = []

[[profiles]]
name = "codex-default"
driver = "codex"
model = "default"
args = []
```

Edit the config to add provider-specific arguments, named agents, and profiles for your workflows.

## Common Commands

```bash
harness doctor
harness doctor --prompt-login
harness providers list
harness providers doctor codex
harness run --profile codex-default --dry-run
harness run --profile claude-default --dry-run
harness run --profile copilot-default --dry-run
harness spawn --agent reviewer --dry-run
harness web --host 127.0.0.1 --port 4317
```

## Project Status

- Phase 0 documentation is in place.
- Phase 1 CLI skeleton is implemented.
- Phase 2 local web console is implemented.
- Phase 3 provider adapter and readiness checks are implemented.
- Phase 4 basic profile and agent execution paths are implemented.
- Phase 5 cloud provider support is documented but not implemented.

## Documentation

- [Documentation Index](docs/README.md)
- [Installation](docs/installation.md)
- [GitHub Builds and Releases](docs/release-artifacts.md)
- [Usage Guide](docs/usage.md)
- [Readiness and Login](docs/readiness.md)
- [Configuration Spec](docs/config-spec.md)
- [Operations and Handoff](docs/operations.md)
- [Architecture](docs/architecture.md)
- [Web Console](docs/web-console.md)
- [Implementation Phases](docs/phases.md)
- [Progress Log](docs/progress.md)
- [Cloud Extensions](docs/cloud-extensions.md)
- [Codex Adapter](docs/adapters/codex.md)
- [Claude Code Adapter](docs/adapters/claude-code.md)
- [Copilot Adapter](docs/adapters/copilot.md)

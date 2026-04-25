# Architecture

## Summary

Clodhopper is a Rust CLI and local web console that orchestrates AI development tools through provider adapters. The first supported providers are Codex, Claude Code, and GitHub Copilot. Each provider is modeled as a process-backed adapter with explicit detection, validation, command construction, process execution, and result collection.

The CLI remains the primary interface. The web console is a companion terminal dashboard served by the Rust process through Axum.

## Components

- **CLI core**: parses commands, loads config, resolves providers/agents/profiles, and invokes shared runtime code.
- **Config loader**: reads `.harness/config.toml` and provides typed workspace, provider, agent, and run-profile settings.
- **Provider registry**: exposes built-in adapters for Codex, Claude Code, and Copilot.
- **Process runner**: launches child processes, captures stdout/stderr, records exit status, and writes logs.
- **Web server**: serves a static dashboard and JSON APIs for provider status, agents, profiles, logs, and launch actions.
- **State directory**: stores logs and future run metadata under `.harness/state`.

## Process Lifecycle

1. Load config.
2. Resolve provider from run profile or agent.
3. Run provider `doctor` checks when requested.
4. Check provider availability, authentication, and a lightweight smoke command.
5. Build an explicit command from provider config, profile/agent settings, and user flags.
6. In dry-run mode, print the command without execution.
7. In live mode, spawn the command in the configured workspace.
8. Capture stdout, stderr, and exit code.
9. Write logs under `.harness/state/logs`.
10. Return a structured result to CLI or web caller.

## Adapter Boundary

Adapters are deliberately thin. They own provider-specific detection and command defaults, but the harness core owns config parsing, workspace resolution, logging, and process supervision.

Every adapter must support:

- `detect`
- `doctor`
- `auth_check`
- `smoke_check`
- `login_command`
- `build_command`
- `spawn`
- `collect_result`

The current implementation represents `spawn` and `collect_result` through one async process execution path that returns a `ProcessResult`.

## Web Console Boundary

The web console should call the same runtime functions as the CLI. It must not invent a second orchestration path. This keeps command behavior consistent across terminal and browser usage.

The initial web UI is intentionally dense and terminal-like:

- Provider status
- Configured agents
- Run profiles
- Recent log list
- Run/spawn actions
- Raw command previews

## Cloud Extension Boundary

Remote providers are future work. The design should support them through a provider interface that can eventually package workspace context, authenticate, submit remote runs, stream logs, collect artifacts, and record audit metadata.

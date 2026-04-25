# Usage Guide

Clodhopper is CLI-first. The web console is a companion view over the same provider checks, profiles, agents, and command-building path.

## Command Overview

```bash
harness init
harness doctor
harness providers list
harness providers doctor <provider>
harness run --profile <profile> [--driver <provider>] [--dry-run]
harness spawn --agent <agent> [--dry-run]
harness web [--host <host>] [--port <port>]
```

## Initialize

```bash
harness init
```

Creates `.harness/config.toml` and local state directories.

Run this once per workspace. The config is intentionally repo-local so each project can define its own agents, profiles, provider commands, and logs.

## Check Readiness

```bash
harness doctor
```

Use this before launching a run. It checks:

- provider command exists
- version can be read
- authentication is available
- a lightweight automation smoke command succeeds

Use JSON output for one provider:

```bash
harness providers doctor codex
```

## List Providers

```bash
harness providers list
```

Shows configured provider names, adapter kinds, and command paths.

## Run a Profile

Profiles are configured orchestrator presets. The default config includes:

- `codex-default`
- `claude-default`
- `copilot-default`

Preview a profile:

```bash
harness run --profile codex-default --dry-run
```

Launch a profile:

```bash
harness run --profile codex-default
```

Switch the driver for a profile at runtime:

```bash
harness run --profile codex-default --driver claude --dry-run
```

This keeps the profile selection flow but uses a different configured provider as the active driver.

## Spawn an Agent

Agents are named role/provider definitions. The default config includes:

- `reviewer`
- `implementer`

Preview:

```bash
harness spawn --agent reviewer --dry-run
```

Launch:

```bash
harness spawn --agent reviewer
```

## Use the Web Console

Start:

```bash
harness web
```

Open:

```text
http://127.0.0.1:4317
```

The web console shows:

- provider readiness
- configured agents
- configured run profiles
- recent logs
- dry-run run/spawn actions

The web console defaults action requests to dry-run. That keeps the browser useful for validation without accidentally launching an interactive process.

## Logs

Live process execution writes logs under:

```text
.harness/state/logs
```

Each run writes:

- `*.stdout.log`
- `*.stderr.log`
- `*.meta.json`

Dry-run commands do not write process logs because no child process is launched.

## Switching Tools

The common switch flow is:

1. Run `harness doctor`.
2. Pick a provider with `ready=true`.
3. Run a profile with that provider as the driver.
4. If one provider is unavailable or out of usage, switch to another configured provider.

Examples:

```bash
harness run --profile codex-default --dry-run
harness run --profile claude-default --dry-run
harness run --profile copilot-default --dry-run
```

When switching tools, review provider-specific args in `.harness/config.toml`. Codex, Claude Code, and Copilot do not expose identical model, prompt, permission, or session flags.

## Safe Workflow

Use this loop while editing config or adding agents:

```bash
harness doctor
harness providers list
harness run --profile <profile> --dry-run
harness spawn --agent <agent> --dry-run
```

Only remove `--dry-run` once the generated command is exactly what you expect.


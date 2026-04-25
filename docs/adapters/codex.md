# Codex Adapter

## Purpose

The Codex adapter launches local Codex CLI processes from harness run profiles or named agents.

## Defaults

- Provider kind: `codex`
- Default command: `codex`
- Default args: none

## Responsibilities

- Detect the configured `codex` command on `PATH`.
- Probe version with `codex --version`.
- Check authentication with `codex login status`.
- Smoke-test automation support with `codex exec --help`.
- Prompt login with `codex login` when `harness doctor --prompt-login` is used.
- Build commands from provider defaults plus profile or agent args.
- Launch in the configured workspace.
- Capture stdout, stderr, and exit status.
- Write run logs under `.harness/state/logs`.

## Open Validation Items

- Confirm the preferred non-interactive invocation mode.
- Confirm model-selection flags.
- Confirm safe prompt/instruction passing behavior.
- Add integration tests with a mocked `codex` executable.

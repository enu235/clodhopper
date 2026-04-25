# Claude Code Adapter

## Purpose

The Claude Code adapter launches local Claude Code CLI processes from harness run profiles or named agents.

## Defaults

- Provider kind: `claude-code`
- Default command: `claude`
- Default args: none

## Responsibilities

- Detect the configured `claude` command on `PATH`.
- Probe version with `claude --version`.
- Check authentication with `claude auth status`.
- Smoke-test automation support with `claude --help`.
- Prompt login with `claude auth login` when `harness doctor --prompt-login` is used.
- Build commands from provider defaults plus profile or agent args.
- Launch in the configured workspace.
- Capture stdout, stderr, and exit status.
- Write run logs under `.harness/state/logs`.

## Open Validation Items

- Confirm model-selection flags.
- Confirm non-interactive and resume-session behavior.
- Confirm how subagent-like workflows should be represented through Claude Code.
- Add integration tests with a mocked `claude` executable.

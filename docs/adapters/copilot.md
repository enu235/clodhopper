# GitHub Copilot Adapter

## Purpose

The Copilot adapter launches local GitHub Copilot tooling. The initial adapter assumes GitHub CLI based Copilot commands because Copilot capabilities are commonly exposed through `gh` extensions and GitHub tooling.

## Defaults

- Provider kind: `copilot`
- Default command: `gh`
- Default args: `["copilot"]`

## Responsibilities

- Detect the configured `gh` command on `PATH`.
- Probe version with `gh --version`.
- Check authentication with `gh auth status`.
- Smoke-test Copilot command availability with `gh copilot -- --help`.
- Prompt login with `gh auth login` when `harness doctor --prompt-login` is used.
- Build commands from provider defaults plus profile or agent args.
- Launch in the configured workspace.
- Capture stdout, stderr, and exit status.
- Write run logs under `.harness/state/logs`.

## Open Validation Items

- Confirm the exact local Copilot CLI surface to support first.
- Confirm whether GitHub Copilot coding agent workflows should be local, cloud-backed, or both.
- Add `gh extension list` checks for Copilot-specific availability.
- Add integration tests with a mocked `gh` executable.

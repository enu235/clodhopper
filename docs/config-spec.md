# Config Spec

The harness config lives at `.harness/config.toml` by default. It is repo-local so different projects can define different agents, workspaces, provider commands, and run profiles.

## Top-Level Shape

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

## Workspace

- `root`: working directory for launched processes. Relative paths resolve from the repo root.
- `state_dir`: harness-owned state directory.
- `log_dir`: process log directory.

## Providers

- `name`: unique provider name referenced by agents and profiles.
- `kind`: adapter kind. Initial values are `codex`, `claude-code`, and `copilot`.
- `command`: optional command override.
- `default_args`: provider-configured arguments passed after adapter defaults and before profile or agent args.
- `env`: optional explicit environment variables for that provider.

Provider defaults:

- `codex`: command `codex`
- `claude-code`: command `claude`
- `copilot`: command `gh`; the adapter adds `copilot` as the first argument.

## Readiness Checks

`harness doctor` performs provider-specific readiness checks:

- Codex: `codex login status` and `codex exec --help`.
- Claude Code: `claude auth status` and `claude --help`.
- Copilot: `gh auth status` and `gh copilot -- --help`.

If a provider is installed but not authenticated, `harness doctor --prompt-login` runs the provider's interactive login command:

- Codex: `codex login`
- Claude Code: `claude auth login`
- Copilot: `gh auth login`

## Agents

- `name`: unique agent name.
- `provider`: provider name to use for spawning.
- `role`: human-readable role or instruction summary.
- `model`: optional provider-specific model preference.
- `args`: provider-specific arguments appended after provider defaults.
- `env`: optional explicit environment variables for this agent.
- `workspace`: optional workspace override.

## Profiles

- `name`: unique profile name.
- `driver`: provider name used as the main orchestrator.
- `model`: optional provider-specific model preference.
- `args`: provider-specific arguments appended after provider defaults.
- `workspace`: optional workspace override.

## Environment Policy

The harness inherits the current process environment by default, then applies explicit provider or agent environment overrides. Future hardening should add allowlists or secret references before remote/cloud execution is implemented.

## Compatibility Notes

Provider CLIs do not expose identical model, prompt, or agent semantics. Config fields should be preserved as provider-specific arguments where necessary instead of pretending every tool has the same API.

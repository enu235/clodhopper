# Readiness and Login

Readiness is the harness install and automation gate. A provider is ready only when it is installed, authenticated, and passes a lightweight smoke check.

## Readiness Fields

`harness doctor` reports:

- `available`: the provider command exists on `PATH` or at the configured path.
- `authenticated`: the provider's auth check exits successfully.
- `ready`: both authentication and smoke checks pass.
- `version`: the first line of the provider's version command, when available.

Detailed JSON:

```bash
harness providers doctor codex
```

Example:

```json
{
  "name": "codex",
  "kind": "codex",
  "command": "codex",
  "available": true,
  "version": "codex-cli 0.125.0",
  "authenticated": true,
  "ready": true,
  "auth_check": ["codex", "login", "status"],
  "smoke_check": ["codex", "exec", "--help"],
  "login_command": ["codex", "login"],
  "message": null
}
```

## Provider Checks

Codex:

- Version: `codex --version`
- Auth: `codex login status`
- Smoke: `codex exec --help`
- Login: `codex login`

Claude Code:

- Version: `claude --version`
- Auth: `claude auth status`
- Smoke: `claude --help`
- Login: `claude auth login`

GitHub Copilot:

- Version: `gh --version`
- Auth: `gh auth status`
- Smoke: `gh copilot -- --help`
- Login: `gh auth login`

## Login Flow

Default readiness checks do not start login prompts.

To let Clodhopper hand off to provider login commands:

```bash
harness doctor --prompt-login
```

For a single provider:

```bash
harness providers doctor codex --prompt-login
```

The harness only starts login when:

- the provider command is available
- the provider reports `authenticated=false`
- `--prompt-login` was explicitly passed

This keeps automation safe in CI or unattended shells.

## CI and Automation

For CI, use non-interactive checks only:

```bash
harness doctor
```

Do not use `--prompt-login` in CI. It is intended for local interactive shells.

Treat `ready=false` as a setup failure for workflows that need that provider. If only one provider is needed, inspect that provider specifically:

```bash
harness providers doctor copilot
```

## Common Failures

`available=false`:

- Install the provider CLI.
- Confirm it is on `PATH`.
- Or set `command` in `.harness/config.toml` to an absolute path.

`authenticated=false`:

- Run `harness doctor --prompt-login`.
- Or run the provider login command directly.

`ready=false` while `authenticated=true`:

- Inspect `message` in `harness providers doctor <provider>`.
- Run the listed `smoke_check` directly.
- Confirm the provider CLI version still supports the expected command.

Copilot smoke check fails:

- Run `gh auth status`.
- Run `gh copilot -- --help`.
- Confirm your GitHub account has Copilot access.


# Operations and Handoff

This document is for humans and future agent sessions that need to continue work without losing the current operating model.

## Before Starting Work

Run:

```bash
git status --short
harness doctor
cargo test
```

If `harness` is not installed, use:

```bash
cargo run -- doctor
cargo test
```

## Development Loop

Use this loop for code changes:

```bash
cargo fmt
cargo test
cargo run -- doctor
cargo run -- run --profile codex-default --dry-run
cargo run -- spawn --agent reviewer --dry-run
```

## Package Validation Loop

Use this loop before relying on GitHub release artifacts:

```bash
cargo fmt -- --check
cargo test --locked
cargo build --release --locked
tmp_root="$(mktemp -d)"
package="$tmp_root/clodhopper-local"
install_dir="$tmp_root/install-bin"
mkdir -p "$package/bin"
cp target/release/harness "$package/bin/harness"
cp scripts/install-unix.sh "$package/install.sh"
chmod +x "$package/bin/harness" "$package/install.sh"
(cd "$package" && ./install.sh --dir "$install_dir")
"$install_dir/harness" doctor
```

This proves the installed binary runs from outside the source tree.

Use this loop for docs-only changes:

```bash
git diff -- README.md docs
```

## Web Console Validation

Start:

```bash
cargo run -- web --port 4321
```

Check:

```bash
curl -s http://127.0.0.1:4321/api/health
curl -s http://127.0.0.1:4321/api/providers
curl -s -X POST http://127.0.0.1:4321/api/run \
  -H 'content-type: application/json' \
  -d '{"profile":"codex-default","dry_run":true}'
```

Stop the web process after validation.

## Handoff Checklist

At the end of a session, update `docs/progress.md` with:

- agent/tool used
- branch
- active phase
- files changed
- commands run
- tests run
- decisions made
- blockers
- next commands

## Safety Rules

- Do not commit `.harness/`.
- Do not use the source checkout as the normal install location.
- Prefer downloaded GitHub artifacts or releases for user installation.
- Use `--dry-run` before launching a newly configured profile or agent.
- Keep provider-specific flags in config rather than hardcoding them in core code.
- Do not automate provider login without explicit user action.
- Do not bypass vendor usage limits, authentication, or terms.
- Keep cloud execution disabled until the remote provider design is implemented.

## Current Gaps

- No live log streaming yet.
- No process cancellation yet.
- No durable run metadata index yet.
- No mocked-provider integration tests yet.
- No implemented cloud provider execution yet.

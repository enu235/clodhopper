# Documentation Index

This folder is the durable operating manual for Clodhopper. Use it as the source of truth when switching between authoring agents or handing work to subagents.

## Start Here

- [Installation](installation.md): build, install, initialize, and verify local provider tooling.
- [GitHub Builds and Releases](release-artifacts.md): CI artifacts, release packages, and installer behavior.
- [Usage Guide](usage.md): day-to-day CLI and web console workflows.
- [Readiness and Login](readiness.md): provider checks, login handoff, CI-safe readiness behavior.
- [Configuration Spec](config-spec.md): `.harness/config.toml` shape and semantics.
- [Operations and Handoff](operations.md): validation commands, session handoff, and safety rules.

## Design and Roadmap

- [Architecture](architecture.md): core components, adapter boundary, process lifecycle.
- [Implementation Phases](phases.md): project checklist and remaining work.
- [Progress Log](progress.md): chronological session notes and handoff status.
- [Web Console](web-console.md): local dashboard behavior and API routes.
- [Cloud Extensions](cloud-extensions.md): future Azure AI Foundry, Google AI Studio, and remote provider design.

## Provider Adapters

- [Codex Adapter](adapters/codex.md)
- [Claude Code Adapter](adapters/claude-code.md)
- [GitHub Copilot Adapter](adapters/copilot.md)

## Recommended Agent Handoff Flow

1. Read [Progress Log](progress.md).
2. Run `git status --short`.
3. Run `cargo test`.
4. Run `cargo run -- doctor`.
5. Review [Implementation Phases](phases.md) for the next unchecked item.
6. Update [Progress Log](progress.md) before ending the session.

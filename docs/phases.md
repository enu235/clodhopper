# Implementation Phases

Use this file as the durable project checklist. Update `docs/progress.md` at the end of every meaningful session.

Checklist states:

- `[ ] Not started`
- `[~] In progress`
- `[!] Blocked`
- `[x] Done`

## Phase 0: Documentation and Operating Model

- [x] Create `README.md`.
- [x] Create architecture documentation.
- [x] Create config spec.
- [x] Create phase checklist.
- [x] Create progress and handoff document.
- [x] Create adapter docs for Codex, Claude Code, and Copilot.
- [x] Create web console doc.
- [x] Create cloud extension doc.

## Phase 1: Rust Workspace and CLI Skeleton

- [x] Create Rust package with `harness` binary.
- [x] Add GitHub build automation.
- [x] Add downloadable release packages.
- [x] Add install scripts that copy the binary outside the source tree.
- [x] Add `harness init`.
- [x] Add `harness doctor`.
- [x] Add `harness doctor --prompt-login`.
- [x] Add `harness providers list`.
- [x] Add `harness providers doctor <provider>`.
- [x] Add `harness run --profile <name>`.
- [x] Add `harness spawn --agent <name>`.
- [x] Add config loading.
- [x] Add state and log directory creation.
- [x] Add dry-run support for safe command inspection.

## Phase 2: Local Web Console

- [x] Add `harness web`.
- [x] Add Axum local server.
- [x] Serve static terminal-dashboard UI.
- [x] Add provider status API.
- [x] Add agents API.
- [x] Add profiles API.
- [x] Add logs API.
- [x] Add run/spawn APIs.
- [ ] Add live log streaming.
- [ ] Add richer run history metadata.

## Phase 3: Provider Adapter Layer

- [x] Define process-backed provider interface.
- [x] Implement Codex adapter.
- [x] Implement Claude Code adapter.
- [x] Implement Copilot adapter.
- [x] Implement detection and version probing.
- [x] Implement authentication readiness checks.
- [x] Implement non-AI smoke checks.
- [x] Implement optional interactive login prompts.
- [x] Implement command construction.
- [x] Implement process execution and capture.
- [ ] Add provider-specific model flag helpers after real CLI behavior is validated.
- [ ] Add integration tests against mocked provider commands.

## Phase 4: Orchestrator Switching and Subagents

- [x] Add run profiles that select active driver.
- [x] Add named agents that select provider.
- [x] Support CLI profile and agent execution.
- [x] Expose profile and agent execution through web API.
- [ ] Add concurrent child process registry.
- [ ] Add cancellation.
- [ ] Add status refresh for running processes.
- [ ] Add durable run metadata index.

## Phase 5: Cloud Extension Design

- [x] Document remote provider extension points.
- [x] Document Azure AI Foundry future path.
- [x] Document Google AI Studio future path.
- [ ] Define remote provider trait once local provider semantics stabilize.
- [ ] Add workspace packaging design.
- [ ] Add artifact retrieval design.
- [ ] Add cloud auth and audit model.

# Cloud Extensions

## Scope

Cloud execution is future work. V1 only implements local CLI-backed providers for Codex, Claude Code, and GitHub Copilot. This document records the extension shape so local provider decisions do not block Azure AI Foundry, Google AI Studio, or other hosted runners later.

## Future Remote Provider Responsibilities

A remote provider must eventually support:

- Authentication and credential validation.
- Workspace packaging.
- Remote run submission.
- Log streaming or polling.
- Artifact retrieval.
- Status and cancellation.
- Audit metadata.
- Cost/usage reporting when available.

## Azure AI Foundry Direction

Future Azure AI Foundry support should model Foundry projects as remote execution targets. The harness should keep local config separate from cloud credentials and should record project, deployment, agent, region, and run identifiers in durable metadata.

## Google AI Studio Direction

Future Google AI Studio support should begin as a model/provider integration, then evolve toward remote agent execution if the platform exposes a suitable workflow. Workspace packaging and artifact return should use the same remote provider concepts as Azure.

## Guardrails

- Do not bypass provider limits, authentication, or terms.
- Do not upload workspace contents without explicit configuration.
- Do not treat local env vars as cloud-safe secrets.
- Record all remote execution identifiers in run metadata.


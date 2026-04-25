# Web Console

## Purpose

The web console is a local companion dashboard for the CLI. It should feel close to modern terminal AI tools: dense, log-forward, command-oriented, and fast to scan.

The web console is not the primary orchestrator in v1. It exposes common actions and status through the same Rust core used by the CLI.

## Launch

```bash
harness web --host 127.0.0.1 --port 4317
```

## Initial Views

- Provider status
- Agents
- Run profiles
- Logs
- Run/spawn controls

## Initial API Routes

- `GET /api/health`
- `GET /api/providers`
- `GET /api/agents`
- `GET /api/profiles`
- `GET /api/logs`
- `POST /api/run`
- `POST /api/spawn`

The `POST` routes accept `dry_run` so command construction can be validated from the UI without launching an interactive tool.

## UX Rules

- Prefer terminal-like density over a marketing page.
- Keep logs and commands visible.
- Avoid hiding provider failures behind generic UI states.
- Show the active config path and workspace path.
- Keep destructive controls out of v1.

## Deferred Work

- Live log streaming through server-sent events or WebSockets.
- Process cancellation.
- Run history search.
- Remote/cloud provider views.
- Optional authentication if binding beyond localhost is supported.


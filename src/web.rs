use crate::config::HarnessConfig;
use crate::providers::ProviderRegistry;
use anyhow::Result;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    config_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
    config_path: String,
}

#[derive(Debug, Serialize)]
struct LogEntry {
    name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct RunRequest {
    profile: String,
    dry_run: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SpawnRequest {
    agent: String,
    dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CommandResponse {
    dry_run: bool,
    command: String,
    status_code: Option<i32>,
    success: Option<bool>,
    stdout: Option<String>,
    stderr: Option<String>,
}

pub async fn serve(config_path: PathBuf, host: String, port: u16) -> Result<()> {
    let state = Arc::new(AppState { config_path });
    let app = Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .route("/api/providers", get(providers))
        .route("/api/agents", get(agents))
        .route("/api/profiles", get(profiles))
        .route("/api/logs", get(logs))
        .route("/api/run", post(run))
        .route("/api/spawn", post(spawn))
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    println!("web console listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        config_path: state.config_path.display().to_string(),
    })
}

async fn providers(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => config,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let registry = ProviderRegistry::new();
    let mut reports = Vec::new();
    for provider in &config.providers {
        let adapter = match registry.adapter_for(&provider.kind) {
            Ok(adapter) => adapter,
            Err(error) => {
                return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
            }
        };
        reports.push(adapter.doctor(provider));
    }
    Json(reports).into_response()
}

async fn agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => Json(config.agents).into_response(),
        Err(error) => (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

async fn profiles(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => Json(config.profiles).into_response(),
        Err(error) => (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

async fn logs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => config,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(&config.workspace.log_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                entries.push(LogEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: path.display().to_string(),
                });
            }
        }
    }
    entries.sort_by(|a, b| b.name.cmp(&a.name));
    Json(entries).into_response()
}

async fn run(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RunRequest>,
) -> impl IntoResponse {
    let config = match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => config,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let profile = match config.profile(&request.profile) {
        Ok(profile) => profile,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let provider = match config.provider(&profile.driver) {
        Ok(provider) => provider,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let registry = ProviderRegistry::new();
    let adapter = match registry.adapter_for(&provider.kind) {
        Ok(adapter) => adapter,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let workspace = profile
        .workspace
        .clone()
        .unwrap_or_else(|| config.workspace.root.clone());
    let plan = match adapter.build_command(provider, &profile.args, workspace, &profile.env) {
        Ok(plan) => plan,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    execute_web_command(plan, request.dry_run.unwrap_or(true)).await
}

async fn spawn(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpawnRequest>,
) -> impl IntoResponse {
    let config = match HarnessConfig::load_or_default(&state.config_path) {
        Ok(config) => config,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let agent = match config.agent(&request.agent) {
        Ok(agent) => agent,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let provider = match config.provider(&agent.provider) {
        Ok(provider) => provider,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let registry = ProviderRegistry::new();
    let adapter = match registry.adapter_for(&provider.kind) {
        Ok(adapter) => adapter,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let workspace = agent
        .workspace
        .clone()
        .unwrap_or_else(|| config.workspace.root.clone());
    let plan = match adapter.build_command(provider, &agent.args, workspace, &agent.env) {
        Ok(plan) => plan,
        Err(error) => {
            return (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    execute_web_command(plan, request.dry_run.unwrap_or(true)).await
}

async fn execute_web_command(
    plan: crate::providers::CommandPlan,
    dry_run: bool,
) -> axum::response::Response {
    if dry_run {
        return Json(CommandResponse {
            dry_run,
            command: plan.shell_preview(),
            status_code: None,
            success: None,
            stdout: None,
            stderr: None,
        })
        .into_response();
    }

    match crate::providers::run_process(plan).await {
        Ok(result) => Json(CommandResponse {
            dry_run,
            command: result.command,
            status_code: result.status_code,
            success: Some(result.success),
            stdout: Some(result.stdout),
            stderr: Some(result.stderr),
        })
        .into_response(),
        Err(error) => (axum::http::StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Clodhopper Harness</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #101114;
      --panel: #181a20;
      --panel-2: #20232b;
      --text: #e8e9ed;
      --muted: #a3a7b3;
      --line: #343844;
      --ok: #63c174;
      --bad: #ee6d6d;
      --accent: #72a7ff;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      background: var(--bg);
      color: var(--text);
      font: 14px/1.45 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }
    header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 14px 18px;
      border-bottom: 1px solid var(--line);
      background: #14161b;
    }
    h1 { margin: 0; font-size: 16px; font-weight: 700; }
    main {
      display: grid;
      grid-template-columns: 1.1fr .9fr;
      gap: 12px;
      padding: 12px;
    }
    section {
      border: 1px solid var(--line);
      background: var(--panel);
      border-radius: 6px;
      min-height: 150px;
      overflow: hidden;
    }
    h2 {
      margin: 0;
      padding: 10px 12px;
      font-size: 13px;
      border-bottom: 1px solid var(--line);
      background: var(--panel-2);
    }
    pre, .list { margin: 0; padding: 12px; overflow: auto; }
    .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
    .row { display: flex; justify-content: space-between; gap: 12px; padding: 6px 0; border-bottom: 1px solid #282b34; }
    .row:last-child { border-bottom: 0; }
    .muted { color: var(--muted); }
    .ok { color: var(--ok); }
    .bad { color: var(--bad); }
    button {
      border: 1px solid var(--line);
      background: #242936;
      color: var(--text);
      border-radius: 5px;
      padding: 7px 10px;
      font: inherit;
      cursor: pointer;
    }
    button:hover { border-color: var(--accent); }
    select {
      width: 100%;
      margin-bottom: 8px;
      border: 1px solid var(--line);
      background: #0f1117;
      color: var(--text);
      border-radius: 5px;
      padding: 8px;
      font: inherit;
    }
    @media (max-width: 900px) {
      main, .grid { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <header>
    <h1>Clodhopper Harness</h1>
    <span id="health" class="muted">loading</span>
  </header>
  <main>
    <section>
      <h2>Providers</h2>
      <div id="providers" class="list"></div>
    </section>
    <section>
      <h2>Actions</h2>
      <div class="list">
        <label class="muted" for="profile">profile</label>
        <select id="profile"></select>
        <button onclick="runProfile()">dry-run profile</button>
        <br><br>
        <label class="muted" for="agent">agent</label>
        <select id="agent"></select>
        <button onclick="spawnAgent()">dry-run agent</button>
      </div>
    </section>
    <section>
      <h2>Agents</h2>
      <pre id="agents"></pre>
    </section>
    <section>
      <h2>Run Profiles</h2>
      <pre id="profiles"></pre>
    </section>
    <section>
      <h2>Logs</h2>
      <pre id="logs"></pre>
    </section>
    <section>
      <h2>Command Output</h2>
      <pre id="output"></pre>
    </section>
  </main>
  <script>
    async function getJson(path) {
      const res = await fetch(path);
      if (!res.ok) throw new Error(await res.text());
      return res.json();
    }
    async function postJson(path, body) {
      const res = await fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
      if (!res.ok) throw new Error(await res.text());
      return res.json();
    }
    function renderJson(id, value) {
      document.getElementById(id).textContent = JSON.stringify(value, null, 2);
    }
    async function refresh() {
      const health = await getJson('/api/health');
      document.getElementById('health').textContent = health.config_path;
      const providers = await getJson('/api/providers');
      document.getElementById('providers').innerHTML = providers.map(p => `<div class="row"><span>${p.name} <span class="muted">${p.kind}</span></span><span class="${p.available ? 'ok' : 'bad'}">${p.available ? 'available' : 'missing'}</span></div>`).join('');
      const agents = await getJson('/api/agents');
      const profiles = await getJson('/api/profiles');
      renderJson('agents', agents);
      renderJson('profiles', profiles);
      renderJson('logs', await getJson('/api/logs'));
      document.getElementById('agent').innerHTML = agents.map(a => `<option value="${a.name}">${a.name}</option>`).join('');
      document.getElementById('profile').innerHTML = profiles.map(p => `<option value="${p.name}">${p.name}</option>`).join('');
    }
    async function runProfile() {
      const profile = document.getElementById('profile').value;
      renderJson('output', await postJson('/api/run', { profile, dry_run: true }));
    }
    async function spawnAgent() {
      const agent = document.getElementById('agent').value;
      renderJson('output', await postJson('/api/spawn', { agent, dry_run: true }));
    }
    refresh().catch(err => document.getElementById('output').textContent = err.stack || String(err));
  </script>
</body>
</html>
"#;

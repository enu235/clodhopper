use crate::config::ProviderConfig;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderDoctorReport {
    pub name: String,
    pub kind: String,
    pub command: String,
    pub available: bool,
    pub version: Option<String>,
    pub authenticated: bool,
    pub ready: bool,
    pub auth_check: Vec<String>,
    pub smoke_check: Vec<String>,
    pub login_command: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommandPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub workspace: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub command: String,
    pub status_code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait ProviderAdapter: Send + Sync {
    fn default_command(&self) -> &'static str;
    fn default_args(&self) -> Vec<String> {
        Vec::new()
    }

    fn command_for(&self, provider: &ProviderConfig) -> String {
        provider
            .command
            .clone()
            .unwrap_or_else(|| self.default_command().to_string())
    }

    fn build_command(
        &self,
        provider: &ProviderConfig,
        extra_args: &[String],
        workspace: PathBuf,
        extra_env: &BTreeMap<String, String>,
    ) -> Result<CommandPlan> {
        let mut args = self.default_args();
        args.extend(provider.default_args.clone());
        args.extend(extra_args.to_vec());

        let mut env = provider.env.clone();
        env.extend(extra_env.clone());

        Ok(CommandPlan {
            program: self.command_for(provider),
            args,
            env,
            workspace,
        })
    }

    fn doctor(&self, provider: &ProviderConfig) -> ProviderDoctorReport {
        let command = self.command_for(provider);
        let available = command_exists(&command);
        if !available {
            let login_command = self.login_command(provider);
            return ProviderDoctorReport {
                name: provider.name.clone(),
                kind: provider.kind.clone(),
                command,
                available: false,
                version: None,
                authenticated: false,
                ready: false,
                auth_check: self.auth_check_command(provider),
                smoke_check: self.smoke_check_command(provider),
                login_command,
                message: Some("command not found on PATH".to_string()),
            };
        }

        let version = probe_version(&command);
        let auth_check = self.auth_check_command(provider);
        let smoke_check = self.smoke_check_command(provider);
        let auth_result = run_check(&auth_check);
        let smoke_result = run_check(&smoke_check);
        let authenticated = auth_result.success;
        let ready = authenticated && smoke_result.success;
        let message = match (auth_result.success, smoke_result.success) {
            (true, true) => None,
            (false, _) => Some(auth_result.message),
            (true, false) => Some(smoke_result.message),
        };

        ProviderDoctorReport {
            name: provider.name.clone(),
            kind: provider.kind.clone(),
            command,
            available: true,
            version,
            authenticated,
            ready,
            auth_check,
            smoke_check,
            login_command: self.login_command(provider),
            message,
        }
    }

    fn auth_check_command(&self, provider: &ProviderConfig) -> Vec<String>;

    fn smoke_check_command(&self, provider: &ProviderConfig) -> Vec<String>;

    fn login_command(&self, provider: &ProviderConfig) -> Vec<String>;
}

pub struct CodexAdapter;
pub struct ClaudeCodeAdapter;
pub struct CopilotAdapter;

impl ProviderAdapter for CodexAdapter {
    fn default_command(&self) -> &'static str {
        "codex"
    }

    fn auth_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "login".into(), "status".into()]
    }

    fn smoke_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "exec".into(), "--help".into()]
    }

    fn login_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "login".into()]
    }
}

impl ProviderAdapter for ClaudeCodeAdapter {
    fn default_command(&self) -> &'static str {
        "claude"
    }

    fn auth_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "auth".into(), "status".into()]
    }

    fn smoke_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "--help".into()]
    }

    fn login_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "auth".into(), "login".into()]
    }
}

impl ProviderAdapter for CopilotAdapter {
    fn default_command(&self) -> &'static str {
        "gh"
    }

    fn default_args(&self) -> Vec<String> {
        vec!["copilot".to_string()]
    }

    fn auth_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "auth".into(), "status".into()]
    }

    fn smoke_check_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![
            self.command_for(provider),
            "copilot".into(),
            "--".into(),
            "--help".into(),
        ]
    }

    fn login_command(&self, provider: &ProviderConfig) -> Vec<String> {
        vec![self.command_for(provider), "auth".into(), "login".into()]
    }
}

pub struct ProviderRegistry {
    codex: CodexAdapter,
    claude: ClaudeCodeAdapter,
    copilot: CopilotAdapter,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            codex: CodexAdapter,
            claude: ClaudeCodeAdapter,
            copilot: CopilotAdapter,
        }
    }

    pub fn adapter_for(&self, kind: &str) -> Result<&dyn ProviderAdapter> {
        match kind {
            "codex" => Ok(&self.codex),
            "claude-code" => Ok(&self.claude),
            "copilot" => Ok(&self.copilot),
            _ => bail!("unknown provider kind '{kind}'"),
        }
    }
}

impl CommandPlan {
    pub fn shell_preview(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(shell_escape(&self.program));
        parts.extend(self.args.iter().map(|arg| shell_escape(arg)));
        format!(
            "cd {} && {}",
            shell_escape_path(&self.workspace),
            parts.join(" ")
        )
    }
}

pub async fn run_process(plan: CommandPlan) -> Result<ProcessResult> {
    let mut command = Command::new(&plan.program);
    command.args(&plan.args);
    command.current_dir(&plan.workspace);
    command.envs(&plan.env);

    let output = command
        .output()
        .await
        .with_context(|| format!("failed to execute {}", plan.shell_preview()))?;

    Ok(ProcessResult {
        command: plan.shell_preview(),
        status_code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

pub fn run_interactive(command: &[String]) -> Result<bool> {
    let Some((program, args)) = command.split_first() else {
        bail!("cannot run an empty command");
    };
    let status = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {}", shell_join(command)))?;
    Ok(status.success())
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') {
        return std::path::Path::new(command).is_file();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

fn probe_version(command: &str) -> Option<String> {
    let output = std::process::Command::new(command)
        .arg("--version")
        .output()
        .ok()?;
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let trimmed = text.lines().next()?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

struct CheckResult {
    success: bool,
    message: String,
}

fn run_check(command: &[String]) -> CheckResult {
    let Some((program, args)) = command.split_first() else {
        return CheckResult {
            success: false,
            message: "empty readiness check".to_string(),
        };
    };
    let output = std::process::Command::new(program).args(args).output();
    match output {
        Ok(output) if output.status.success() => CheckResult {
            success: true,
            message: "ok".to_string(),
        },
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let details = first_non_empty_line(&stderr)
                .or_else(|| first_non_empty_line(&stdout))
                .unwrap_or("check failed");
            CheckResult {
                success: false,
                message: format!("{} failed: {}", shell_join(command), details),
            }
        }
        Err(error) => CheckResult {
            success: false,
            message: format!("{} failed: {error}", shell_join(command)),
        },
    }
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn shell_escape_path(path: &std::path::Path) -> String {
    shell_escape(&path.display().to_string())
}

fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=@".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub fn shell_join(parts: &[String]) -> String {
    parts
        .iter()
        .map(|part| shell_escape(part))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copilot_defaults_to_gh_copilot() {
        let adapter = CopilotAdapter;
        let provider = ProviderConfig {
            name: "copilot".to_string(),
            kind: "copilot".to_string(),
            command: None,
            default_args: Vec::new(),
            env: BTreeMap::new(),
        };
        let plan = adapter
            .build_command(&provider, &[], PathBuf::from("."), &BTreeMap::new())
            .unwrap();
        assert_eq!(plan.program, "gh");
        assert_eq!(plan.args, vec!["copilot"]);
    }

    #[test]
    fn readiness_commands_are_provider_specific() {
        let codex = CodexAdapter;
        let claude = ClaudeCodeAdapter;
        let copilot = CopilotAdapter;
        let codex_provider = provider("codex", "codex", "codex");
        let claude_provider = provider("claude", "claude-code", "claude");
        let copilot_provider = provider("copilot", "copilot", "gh");

        assert_eq!(
            codex.auth_check_command(&codex_provider),
            vec!["codex", "login", "status"]
        );
        assert_eq!(
            claude.auth_check_command(&claude_provider),
            vec!["claude", "auth", "status"]
        );
        assert_eq!(
            copilot.auth_check_command(&copilot_provider),
            vec!["gh", "auth", "status"]
        );
        assert_eq!(
            copilot.smoke_check_command(&copilot_provider),
            vec!["gh", "copilot", "--", "--help"]
        );
    }

    fn provider(name: &str, kind: &str, command: &str) -> ProviderConfig {
        ProviderConfig {
            name: name.to_string(),
            kind: kind.to_string(),
            command: Some(command.to_string()),
            default_args: Vec::new(),
            env: BTreeMap::new(),
        }
    }
}

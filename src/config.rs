use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG: &str = r#"[workspace]
root = "."
state_dir = ".harness/state"
log_dir = ".harness/state/logs"

[[providers]]
name = "codex"
kind = "codex"
command = "codex"
default_args = []

[[providers]]
name = "claude"
kind = "claude-code"
command = "claude"
default_args = []

[[providers]]
name = "copilot"
kind = "copilot"
command = "gh"
default_args = []

[[agents]]
name = "reviewer"
provider = "codex"
role = "Review changes for correctness, tests, and maintainability."
model = "default"
args = []

[[agents]]
name = "implementer"
provider = "claude"
role = "Implement scoped tasks using the current workspace."
model = "default"
args = []

[[profiles]]
name = "codex-default"
driver = "codex"
model = "default"
args = []

[[profiles]]
name = "claude-default"
driver = "claude"
model = "default"
args = []

[[profiles]]
name = "copilot-default"
driver = "copilot"
model = "default"
args = []
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessConfig {
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub profiles: Vec<ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_root")]
    pub root: PathBuf,
    #[serde(default = "default_state_dir")]
    pub state_dir: PathBuf,
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub kind: String,
    pub command: Option<String>,
    #[serde(default)]
    pub default_args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub role: String,
    pub model: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub driver: String,
    pub model: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub workspace: Option<PathBuf>,
}

impl HarnessConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let mut config: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        config.normalize_paths(path);
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let mut config: Self = toml::from_str(DEFAULT_CONFIG)?;
            config.normalize_paths(path);
            config.validate()?;
            Ok(config)
        }
    }

    pub fn provider(&self, name: &str) -> Result<&ProviderConfig> {
        self.providers
            .iter()
            .find(|provider| provider.name == name)
            .with_context(|| format!("unknown provider '{name}'"))
    }

    pub fn agent(&self, name: &str) -> Result<&AgentConfig> {
        self.agents
            .iter()
            .find(|agent| agent.name == name)
            .with_context(|| format!("unknown agent '{name}'"))
    }

    pub fn profile(&self, name: &str) -> Result<&ProfileConfig> {
        self.profiles
            .iter()
            .find(|profile| profile.name == name)
            .with_context(|| format!("unknown profile '{name}'"))
    }

    pub fn ensure_state_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.workspace.state_dir)
            .with_context(|| format!("failed to create {}", self.workspace.state_dir.display()))?;
        std::fs::create_dir_all(&self.workspace.log_dir)
            .with_context(|| format!("failed to create {}", self.workspace.log_dir.display()))?;
        Ok(())
    }

    fn normalize_paths(&mut self, config_path: &Path) {
        let base = config_path
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));
        self.workspace.root = absolutize(base, &self.workspace.root);
        self.workspace.state_dir = absolutize(base, &self.workspace.state_dir);
        self.workspace.log_dir = absolutize(base, &self.workspace.log_dir);
        for agent in &mut self.agents {
            if let Some(workspace) = &agent.workspace {
                agent.workspace = Some(absolutize(base, workspace));
            }
        }
        for profile in &mut self.profiles {
            if let Some(workspace) = &profile.workspace {
                profile.workspace = Some(absolutize(base, workspace));
            }
        }
    }

    fn validate(&self) -> Result<()> {
        if self.providers.is_empty() {
            bail!("config must define at least one provider");
        }
        for agent in &self.agents {
            self.provider(&agent.provider)
                .with_context(|| format!("agent '{}' references a missing provider", agent.name))?;
        }
        for profile in &self.profiles {
            self.provider(&profile.driver).with_context(|| {
                format!("profile '{}' references a missing driver", profile.name)
            })?;
        }
        Ok(())
    }
}

fn default_root() -> PathBuf {
    PathBuf::from(".")
}

fn default_state_dir() -> PathBuf {
    PathBuf::from(".harness/state")
}

fn default_log_dir() -> PathBuf {
    PathBuf::from(".harness/state/logs")
}

fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config: HarnessConfig = toml::from_str(DEFAULT_CONFIG).unwrap();
        config.validate().unwrap();
        assert_eq!(config.providers.len(), 3);
    }
}

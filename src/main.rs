mod config;
mod providers;
mod web;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use config::{AgentConfig, HarnessConfig, ProfileConfig};
use providers::{CommandPlan, ProcessResult, ProviderRegistry};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "harness")]
#[command(about = "Switchable AI agent harness for Codex, Claude Code, and Copilot.")]
struct Cli {
    #[arg(long, global = true, default_value = ".harness/config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    Doctor(DoctorArgs),
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
    Run(RunArgs),
    Spawn(SpawnArgs),
    Web(WebArgs),
}

#[derive(Debug, Subcommand)]
enum ProviderCommands {
    List,
    Doctor(ProviderDoctorArgs),
}

#[derive(Debug, Args)]
struct DoctorArgs {
    #[arg(long)]
    prompt_login: bool,
}

#[derive(Debug, Args)]
struct ProviderDoctorArgs {
    provider: String,
    #[arg(long)]
    prompt_login: bool,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long)]
    profile: String,
    #[arg(long)]
    driver: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct SpawnArgs {
    #[arg(long)]
    agent: String,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args, Clone)]
struct WebArgs {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 4317)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_config(&cli.config),
        Commands::Doctor(args) => doctor(&cli.config, args).await,
        Commands::Providers { command } => provider_command(&cli.config, command).await,
        Commands::Run(args) => run_profile(&cli.config, args).await,
        Commands::Spawn(args) => spawn_agent(&cli.config, args).await,
        Commands::Web(args) => web::serve(cli.config, args.host, args.port).await,
    }
}

fn init_config(config_path: &Path) -> Result<()> {
    if config_path.exists() {
        bail!("config already exists at {}", config_path.display());
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    std::fs::write(config_path, config::DEFAULT_CONFIG)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    let config = HarnessConfig::load(config_path)?;
    config.ensure_state_dirs()?;

    println!("created {}", config_path.display());
    println!("created {}", config.workspace.log_dir.display());
    Ok(())
}

async fn doctor(config_path: &Path, args: DoctorArgs) -> Result<()> {
    let config = HarnessConfig::load_or_default(config_path)?;
    config.ensure_state_dirs()?;
    println!("config: {}", config_path.display());
    println!("workspace: {}", config.workspace.root.display());
    println!("state: {}", config.workspace.state_dir.display());
    println!("logs: {}", config.workspace.log_dir.display());

    let registry = ProviderRegistry::new();
    for provider in &config.providers {
        let adapter = registry.adapter_for(&provider.kind)?;
        let report = adapter.doctor(provider);
        println!(
            "{} ({}) command={} available={} authenticated={} ready={} version={}",
            provider.name,
            provider.kind,
            adapter.command_for(provider),
            report.available,
            report.authenticated,
            report.ready,
            report.version.unwrap_or_else(|| "unknown".to_string())
        );
        if let Some(message) = report.message {
            println!("  {}", message);
        }
        if !report.ready && !report.login_command.is_empty() {
            println!("  login: {}", providers::shell_join(&report.login_command));
            if args.prompt_login && report.available && !report.authenticated {
                println!("  starting interactive login for {}", provider.name);
                let login_ok = providers::run_interactive(&report.login_command)?;
                println!("  login completed: {login_ok}");
            }
        }
    }
    Ok(())
}

async fn provider_command(config_path: &Path, command: ProviderCommands) -> Result<()> {
    let config = HarnessConfig::load_or_default(config_path)?;
    let registry = ProviderRegistry::new();

    match command {
        ProviderCommands::List => {
            for provider in &config.providers {
                let adapter = registry.adapter_for(&provider.kind)?;
                println!(
                    "{} kind={} command={}",
                    provider.name,
                    provider.kind,
                    adapter.command_for(provider)
                );
            }
        }
        ProviderCommands::Doctor(args) => {
            let provider_config = config.provider(&args.provider)?;
            let adapter = registry.adapter_for(&provider_config.kind)?;
            let report = adapter.doctor(provider_config);
            println!("{}", serde_json::to_string_pretty(&report)?);
            if args.prompt_login && report.available && !report.authenticated {
                providers::run_interactive(&report.login_command)?;
            }
        }
    }
    Ok(())
}

async fn run_profile(config_path: &Path, args: RunArgs) -> Result<()> {
    let config = HarnessConfig::load_or_default(config_path)?;
    config.ensure_state_dirs()?;
    let profile = config.profile(&args.profile)?;
    let driver = args.driver.as_deref().unwrap_or(&profile.driver);
    let provider = config.provider(driver)?;
    let plan = build_profile_plan(&config, provider, profile)?;
    execute_or_print(&config, "run", &profile.name, plan, args.dry_run).await
}

async fn spawn_agent(config_path: &Path, args: SpawnArgs) -> Result<()> {
    let config = HarnessConfig::load_or_default(config_path)?;
    config.ensure_state_dirs()?;
    let agent = config.agent(&args.agent)?;
    let provider = config.provider(&agent.provider)?;
    let plan = build_agent_plan(&config, provider, agent)?;
    execute_or_print(&config, "agent", &agent.name, plan, args.dry_run).await
}

fn build_profile_plan(
    config: &HarnessConfig,
    provider: &config::ProviderConfig,
    profile: &ProfileConfig,
) -> Result<CommandPlan> {
    let registry = ProviderRegistry::new();
    let adapter = registry.adapter_for(&provider.kind)?;
    let workspace = profile
        .workspace
        .clone()
        .unwrap_or_else(|| config.workspace.root.clone());
    adapter.build_command(provider, &profile.args, workspace, &profile.env)
}

fn build_agent_plan(
    config: &HarnessConfig,
    provider: &config::ProviderConfig,
    agent: &AgentConfig,
) -> Result<CommandPlan> {
    let registry = ProviderRegistry::new();
    let adapter = registry.adapter_for(&provider.kind)?;
    let workspace = agent
        .workspace
        .clone()
        .unwrap_or_else(|| config.workspace.root.clone());
    adapter.build_command(provider, &agent.args, workspace, &agent.env)
}

async fn execute_or_print(
    config: &HarnessConfig,
    category: &str,
    name: &str,
    plan: CommandPlan,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("{}", plan.shell_preview());
        return Ok(());
    }

    let result = providers::run_process(plan).await?;
    write_process_logs(config, category, name, &result)?;
    print_process_result(&result);
    if !result.success {
        bail!("process exited with status {:?}", result.status_code);
    }
    Ok(())
}

fn write_process_logs(
    config: &HarnessConfig,
    category: &str,
    name: &str,
    result: &ProcessResult,
) -> Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let safe_name = name.replace(['/', '\\', ' '], "-");
    let base = config
        .workspace
        .log_dir
        .join(format!("{stamp}-{category}-{safe_name}"));
    std::fs::write(base.with_extension("stdout.log"), &result.stdout)?;
    std::fs::write(base.with_extension("stderr.log"), &result.stderr)?;
    std::fs::write(
        base.with_extension("meta.json"),
        serde_json::to_vec_pretty(result)?,
    )?;
    Ok(())
}

fn print_process_result(result: &ProcessResult) {
    println!("command: {}", result.command);
    println!("status: {:?}", result.status_code);
    if !result.stdout.trim().is_empty() {
        println!("stdout:\n{}", result.stdout);
    }
    if !result.stderr.trim().is_empty() {
        eprintln!("stderr:\n{}", result.stderr);
    }
}

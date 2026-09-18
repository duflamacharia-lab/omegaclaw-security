use clap::{Parser, Subcommand};
use omegaclaw::agent::{AgentConfig, OmegaClawAgent};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "omegaclaw-agent",
    version,
    about = "Bounded defensive Web3 security agent"
)]
struct Cli {
    #[arg(long, env = "OMEGACLAW_WORKSPACE", default_value = ".")]
    workspace: PathBuf,
    #[arg(long, env="OMEGACLAW_PROVIDER", default_value="offline", value_parser=["offline","gemini","dify","huggingface"])]
    provider: String,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug)]
enum Command {
    Inspect,
    Analyze {
        path: String,
        #[arg(
            short,
            long,
            default_value = "Review this file defensively and identify evidence-backed next steps"
        )]
        question: String,
    },
    Check {
        name: String,
    },
    Audit,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = AgentConfig::from_env(cli.workspace.canonicalize()?);
    config.provider = cli.provider;
    let agent = OmegaClawAgent::new(config);
    match cli.command {
        Command::Inspect => println!(
            "{}",
            serde_json::to_string_pretty(&agent.inspect_repo().await?)?
        ),
        Command::Analyze { path, question } => println!(
            "{}",
            serde_json::to_string_pretty(&agent.analyze_path(&path, &question).await?)?
        ),
        Command::Check { name } => println!(
            "{}",
            serde_json::to_string_pretty(&agent.run_safe_check(&name).await?)?
        ),
        Command::Audit => {
            let events = agent.audit_events().await;
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
    }
    Ok(())
}

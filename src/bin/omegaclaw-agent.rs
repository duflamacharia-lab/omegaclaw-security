use clap::{Parser, Subcommand};
use omegaclaw::{
    agent::{AgentConfig, OmegaClawAgent},
    manifest::ArtifactManifest,
};
use serde::Deserialize;
use std::fs;
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
    ValidateKnowledge,
    ValidateManifests {
        #[arg(long, default_value = "benchmarks/manifests.json")]
        path: PathBuf,
    },
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
        Command::ValidateKnowledge => println!(
            "{}",
            serde_json::to_string_pretty(&agent.validate_knowledge_prerequisites()?)?
        ),
        Command::ValidateManifests { path } => {
            #[derive(Deserialize)]
            struct Catalog {
                artifacts: Vec<ArtifactManifest>,
            }
            let catalog: Catalog = serde_json::from_str(&fs::read_to_string(path)?)?;
            for artifact in &catalog.artifacts {
                artifact.validate().map_err(anyhow::Error::msg)?;
            }
            println!("validated {} artifact manifests", catalog.artifacts.len());
        }
    }
    Ok(())
}

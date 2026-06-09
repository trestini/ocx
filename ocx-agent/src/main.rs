use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: AgentCommands,
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Add an agent to the current project
    Add {
        /// Agent name
        name: String,
    },
    /// List available agents
    List {
        /// List system-wide installed agents
        #[arg(long = "system")]
        system: bool,
    },
    /// Create a new agent specific for this project
    New {
        /// Agent name
        name: String,
        /// Model that this agent must use
        #[arg(short = 'm', long = "model", required = true)]
        model: String,
        /// Model type (primary or subagent)
        #[arg(short = 't', long = "type", default_value = "primary")]
        model_type: String,
        /// System instruction for the agent
        #[arg(short = 'p', long = "prompt")]
        prompt: Option<String>,
        /// Allowed permissions
        #[arg(long = "allowed", default_value = "read,grep", value_delimiter = ',')]
        allowed: Vec<String>,
        /// Denied permissions
        #[arg(long = "denied", default_value = "*", value_delimiter = ',')]
        denied: Vec<String>,
    },
    /// Remove an agent from the current project
    Rm {
        /// Agent name
        name: String,
    },
}

fn require_local_project() {
    if !Path::new(".opencode/opencode.json").is_file() {
        println!("Local project was not created");
        // TODO: re-evaluate exit codes
        std::process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        AgentCommands::Add { name } => {
            require_local_project();

            let mut config = ocx_common::read_opencode_config(true).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            ocx_common::add_agent_to_config(&mut config, name).unwrap_or_else(|e| {
                println!("{e}");
                // TODO: re-evaluate exit codes
                std::process::exit(1);
            });

            ocx_common::write_local_config(&config).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            println!("Agent {name} added to the project");
        }
        AgentCommands::List { system } => {
            require_local_project();

            if !system {
                println!("no agents configured");
                return;
            }
            let entries = ocx_common::list_system_agents();
            let mut counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for (name, _) in &entries {
                *counts.entry(name).or_insert(0) += 1;
            }

            let mut seqs: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();

            for (name, path) in &entries {
                if counts[name.as_str()] == 1 {
                    println!("{name}");
                } else {
                    let seq = seqs.entry(name).or_insert(0);
                    *seq += 1;
                    println!("{name}.{seq} ({})", path.display());
                }
            }
        }
        AgentCommands::New {
            name,
            model,
            model_type,
            prompt,
            allowed,
            denied,
        } => {
            require_local_project();
            println!("new agent:");
            println!("  name: {name}");
            println!("  model: {model}");
            println!("  type: {model_type}");
            if let Some(p) = prompt {
                println!("  prompt: {p}");
            }
            println!("  allowed: {}", allowed.join(", "));
            println!("  denied: {}", denied.join(", "));
        }
        AgentCommands::Rm { name } => {
            require_local_project();
            println!("remove agent: {name}");
        }
    }
}

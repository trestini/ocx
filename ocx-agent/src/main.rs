use clap::{Parser, Subcommand};

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
    List,
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        AgentCommands::Add { name } => {
            println!("add agent: {name}");
        }
        AgentCommands::List => {
            println!("list agents");
        }
        AgentCommands::New {
            name,
            model,
            model_type,
            prompt,
            allowed,
            denied,
        } => {
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
            println!("remove agent: {name}");
        }
    }
}

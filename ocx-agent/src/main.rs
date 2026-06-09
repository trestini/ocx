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
    #[command(alias = "ls")]
    List {
        /// List system-wide installed agents
        #[arg(long = "system")]
        system: bool,
    },
    /// Create a new agent specific for this project
    Create {
        /// Agent name
        name: String,
        /// Model that this agent must use
        #[arg(short = 'm', long = "model", required = true)]
        model: String,
        /// Model type (primary or subagent)
        #[arg(short = 't', long = "type")]
        model_type: Option<String>,
        /// System instruction for the agent
        #[arg(short = 'p', long = "prompt")]
        prompt: Option<String>,
        /// Allowed permissions
        #[arg(long = "allow", value_delimiter = ',')]
        allowed: Option<Vec<String>>,
        /// Denied permissions
        #[arg(long = "denied", value_delimiter = ',')]
        denied: Option<Vec<String>>,
    },
    /// Export an agent to markdown
    Export {
        /// Agent name
        name: String,
    },
    /// Remove an agent from the current project
    #[command(alias = "rm")]
    Remove {
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
                let config = ocx_common::read_opencode_config(true).unwrap_or_else(|e| {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                });
                let names: Vec<&str> = config
                    .as_object()
                    .and_then(|obj| obj.get("agent"))
                    .and_then(|v| v.as_object())
                    .map(|agents| agents.keys().map(|k| k.as_str()).collect())
                    .unwrap_or_default();
                for name in names {
                    println!("{name}");
                }
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
        AgentCommands::Create {
            name,
            model,
            model_type,
            prompt,
            allowed,
            denied,
        } => {
            require_local_project();

            let mut agent = serde_json::Map::new();
            agent.insert("model".to_string(), serde_json::json!(model));

            if let Some(t) = model_type {
                agent.insert("type".to_string(), serde_json::json!(t));
            }
            if let Some(p) = prompt {
                agent.insert("prompt".to_string(), serde_json::json!(p));
            }

            let mut permission = serde_json::Map::new();
            let args: Vec<String> = std::env::args().collect();
            let deny_first = args.iter().position(|a| a == "--allow" || a == "--denied")
                .is_some_and(|i| args[i] == "--denied");

            if !deny_first {
                if let Some(list) = allowed {
                    for p in list {
                        permission.insert(p.to_string(), serde_json::json!("allow"));
                    }
                }
                if let Some(list) = denied {
                    for p in list {
                        permission.insert(p.to_string(), serde_json::json!("deny"));
                    }
                }
            } else {
                if let Some(list) = denied {
                    for p in list {
                        permission.insert(p.to_string(), serde_json::json!("deny"));
                    }
                }
                if let Some(list) = allowed {
                    for p in list {
                        permission.insert(p.to_string(), serde_json::json!("allow"));
                    }
                }
            }
            if !permission.is_empty() {
                agent.insert("permission".to_string(), serde_json::Value::Object(permission));
            }

            let agent = serde_json::Value::Object(agent);

            let mut config = ocx_common::read_opencode_config(true).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            if let Some(obj) = config.as_object_mut() {
                if !obj.contains_key("agent") {
                    obj.insert("agent".to_string(), serde_json::json!({}));
                }
                if let Some(agent_obj) = obj.get_mut("agent").and_then(|v| v.as_object_mut()) {
                    agent_obj.insert(name.clone(), agent);
                }
            }

            ocx_common::write_local_config(&config).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            println!("Agent {name} created");
        }
        AgentCommands::Export { name } => {
            require_local_project();

            let config = ocx_common::read_opencode_config(true).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            let agent = config
                .as_object()
                .and_then(|obj| obj.get("agent"))
                .and_then(|v| v.as_object())
                .and_then(|agents| agents.get(name.as_str()))
                .ok_or_else(|| format!("Agent {name} doesn't exists"))
                .unwrap_or_else(|e| {
                    println!("{e}");
                    std::process::exit(1);
                });

            let markdown = ocx_common::agent_to_markdown(agent).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            println!("{markdown}");
        }
        AgentCommands::Remove { name } => {
            require_local_project();

            let mut config = ocx_common::read_opencode_config(true).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            let removed = config
                .as_object_mut()
                .and_then(|obj| obj.get_mut("agent"))
                .and_then(|v| v.as_object_mut())
                .and_then(|agents| agents.remove(name.as_str()))
                .is_some();

            if !removed {
                println!("Agent {name} doesn't exists");
                // TODO: re-evaluate exit codes
                std::process::exit(1);
            }

            ocx_common::write_local_config(&config).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });

            println!("Agent {name} removed from the project");
        }
    }
}

use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(about = "Setup current directory as a new OpenCode project")]
struct Cli {
    #[arg(short = 'a', long = "agents", value_delimiter = ',')]
    agents: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    let opencode_dir = Path::new(".opencode");

    if opencode_dir.is_dir() {
        println!("Project already created");
        return;
    }

    fs::create_dir(opencode_dir).expect("failed to create .opencode directory");

    let mut config = serde_json::json!({
        "$schema": "https://opencode.ai/config.json"
    });

    if let Some(agents) = &cli.agents {
        for name in agents {
            ocx_common::add_agent_to_config(&mut config, name).unwrap_or_else(|e| {
                println!("{e}");
                std::process::exit(1);
            });
        }
    }

    ocx_common::write_local_config(&config).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
}

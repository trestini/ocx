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

    if cli.agents.is_some() {
        println!("evaluating --agents");
    }

    let config = serde_json::json!({
        "$schema": "https://opencode.ai/config.json"
    });

    let config_str = serde_json::to_string_pretty(&config).expect("failed to serialize config");
    fs::write(opencode_dir.join("opencode.json"), config_str)
        .expect("failed to write opencode.json");
}

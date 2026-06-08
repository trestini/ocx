use clap::Parser;

#[derive(Parser)]
#[command(about = "Setup current directory as a new OpenCode project")]
struct Cli {
    #[arg(short = 'a', long = "agents", value_delimiter = ',')]
    agents: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    let config = ocx_common::read_opencode_config(false);
    println!("config: {config}");

    match cli.agents {
        Some(agents) => println!("agents: {}", agents.join(", ")),
        None => println!("Project created without agents"),
    }
}

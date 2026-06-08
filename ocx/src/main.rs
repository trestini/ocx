use clap::{CommandFactory, Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new opencode project in the current directory
    New,
    /// Guide the user on setting up agents for opencode
    Agent,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New) => run_binary("ocx-new"),
        Some(Commands::Agent) => run_binary("ocx-agent"),
        None => {
            let mut cmd = Cli::command();
            cmd.print_help().unwrap();
            println!();
        }
    }
}

fn run_binary(name: &str) {
    let status = match Command::new(name).status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to run `{name}`: {e}");
            std::process::exit(1);
        }
    };

    let code = status.code().unwrap_or(1);
    std::process::exit(code);
}

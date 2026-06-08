use std::process::Command;

const HELP: &str = "\
Usage: ocx [COMMAND]

Commands:
  new    Create a new opencode project in the current directory
  agent  Guide the user on setting up agents for opencode
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str());

    match subcommand {
        Some(cmd @ ("new" | "agent")) => {
            let binary = format!("ocx-{cmd}");
            let status = match Command::new(&binary).args(&args[2..]).status() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: failed to run `{binary}`: {e}");
                    std::process::exit(1);
                }
            };
            std::process::exit(status.code().unwrap_or(1));
        }
        Some("help" | "-h" | "--help") => {
            println!("{HELP}");
        }
        Some("-V" | "--version") => {
            println!("ocx 0.1.0");
        }
        Some(cmd) => {
            eprintln!("error: unknown subcommand '{cmd}'");
            println!();
            println!("{HELP}");
            std::process::exit(2);
        }
        None => {
            println!("{HELP}");
        }
    }
}

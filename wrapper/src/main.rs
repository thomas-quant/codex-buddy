use std::io::Read;

use buddy_wrapper::{app, codex::relay::relay_hook_payload};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    HookRelay {
        #[arg(long)]
        socket: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::HookRelay { socket }) => {
            let mut stdin = Vec::new();
            std::io::stdin().read_to_end(&mut stdin)?;
            relay_hook_payload(socket, stdin)?;
            Ok(())
        }
        None => app::run_default(),
    }
}

use clap::{Parser, SubCommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "spoofer")]
#[command(about = "Userland Root spoofer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(SubCommand)]
enum Commands {
    Apply {
        #[arg(required = true)]
        command: Vec<String>,

        #[arg(short, long, default_value = "/sdcard/RootLand/spoof.state")]
        state: String,

        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Apply { command, state, verbose } => {
            setup_logging(verbose);
            run_with_ptrace(command, &state)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}


use clap::{Parser, Subcommand};
use minsuki::{Config, PtraceInterceptor, StateManager};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "minsuki")]
#[command(about = "MinSuki - Minimal SuperUser Emulation Layer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command with fake root privileges (ptrace mode)
    Run {
        /// The command to execute
        #[arg(required = true)]
        command: Vec<String>,
        
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
        
        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Run a command with LD_PRELOAD interception
    Preload {
        /// The command to execute
        #[arg(required = true)]
        command: Vec<String>,
        
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
        
        /// Path to libminsuki.so
        #[arg(short, long)]
        lib: Option<String>,
        
        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show the current fake state
    Status {
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
    },
    
    /// Clear the state file
    Clear {
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
    },
    
    /// Manually modify fake file ownership
    Chown {
        /// File path
        path: String,
        
        /// User ID
        uid: u32,
        
        /// Group ID
        gid: u32,
        
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
    },
    
    /// Manually modify fake file permissions
    Chmod {
        /// File path
        path: String,
        
        /// Mode (octal, e.g., 755)
        mode: String,
        
        /// State file path
        #[arg(short, long, default_value = "/tmp/minsuki.state")]
        state: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Run { command, state, verbose } => {
            setup_logging(verbose);
            run_with_ptrace(command, &state)
        }
        Commands::Preload { command, state, lib, verbose } => {
            setup_logging(verbose);
            run_with_preload(command, &state, lib)
        }
        Commands::Status { state } => {
            show_status(&state)
        }
        Commands::Clear { state } => {
            clear_state(&state)
        }
        Commands::Chown { path, uid, gid, state } => {
            manual_chown(&path, uid, gid, &state)
        }
        Commands::Chmod { path, mode, state } => {
            manual_chmod(&path, &mode, &state)
        }
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn setup_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
}

fn run_with_ptrace(command: Vec<String>, state_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 MinSuki: Running with ptrace interception");
    println!("📦 Command: {}", command.join(" "));
    println!("💾 State file: {}", state_file);
    println!();
    
    let interceptor = PtraceInterceptor::new(state_file)?;
    interceptor.run(&command)?;
    
    Ok(())
}

fn run_with_preload(command: Vec<String>, state_file: &str, lib_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 MinSuki: Running with LD_PRELOAD interception");
    println!("📦 Command: {}", command.join(" "));
    println!("💾 State file: {}", state_file);
    
    // Find libminsuki.so
    let lib = if let Some(path) = lib_path {
        path
    } else {
        // Try to find it in common locations
        let candidates = vec![
            "./target/debug/libminsuki.so",
            "./target/release/libminsuki.so",
            "/usr/local/lib/libminsuki.so",
            "/usr/lib/libminsuki.so",
        ];
        
        candidates.into_iter()
            .find(|p| PathBuf::from(p).exists())
            .ok_or("Could not find libminsuki.so. Please specify with --lib")?
            .to_string()
    };
    
    println!("📚 Library: {}", lib);
    println!();
    
    // Set environment variables
    std::env::set_var("LD_PRELOAD", &lib);
    std::env::set_var("MINSUKI_STATE", state_file);
    
    // Execute the command
    let status = process::Command::new(&command[0])
        .args(&command[1..])
        .status()?;
    
    if !status.success() {
        eprintln!("Command failed with exit code: {:?}", status.code());
    }
    
    Ok(())
}

fn show_status(state_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manager = StateManager::new(state_file)?;
    let state = manager.get_state();
    let state = state.lock().unwrap();
    
    println!("📊 MinSuki State Report");
    println!("========================");
    println!();
    println!("Current UID: {}", state.current_uid);
    println!("Current GID: {}", state.current_gid);
    println!("Effective UID: {} ({})", state.effective_uid, if state.effective_uid == 0 { "root" } else { "user" });
    println!("Effective GID: {} ({})", state.effective_gid, if state.effective_gid == 0 { "root" } else { "user" });
    println!();
    println!("Capabilities: {}", state.capabilities.join(", "));
    println!();
    println!("Fake File Metadata ({} entries):", state.files.len());
    println!("----------------------------------");
    
    for (path, meta) in &state.files {
        println!("  {:?}", path.display());
        println!("    UID: {}, GID: {}, Mode: {:o}", meta.uid, meta.gid, meta.mode);
    }
    
    Ok(())
}

fn clear_state(state_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if PathBuf::from(state_file).exists() {
        std::fs::remove_file(state_file)?;
        println!("✅ Cleared state file: {}", state_file);
    } else {
        println!("ℹ️  State file does not exist: {}", state_file);
    }
    Ok(())
}

fn manual_chown(path: &str, uid: u32, gid: u32, state_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manager = StateManager::new(state_file)?;
    manager.chown(PathBuf::from(path), uid, gid)?;
    println!("✅ Set ownership of {} to {}:{}", path, uid, gid);
    Ok(())
}

fn manual_chmod(path: &str, mode_str: &str, state_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mode = u32::from_str_radix(mode_str, 8)?;
    let manager = StateManager::new(state_file)?;
    manager.chmod(PathBuf::from(path), mode)?;
    println!("✅ Set permissions of {} to {:o}", path, mode);
    Ok(())
}
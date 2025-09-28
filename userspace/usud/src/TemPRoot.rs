use std::process::Command;
use std::ffi::CStr;
use std::mem;
use clap::Parser;
use clap::Subcommand;

let CheckPRoot = Command::new("which proot")
    .output()
    .expect("Failed to execute command");

let PRoot = Command::new("proot")
    .arg("-r $USERSUROOT/temproot/default")
    .output()
    .expect("Failed to execute proot!")

fn rootCheck() {
    if CheckPRoot.status.success() {
        println!("PRoot is installed.");
        PRoot;
    } else {
        eprintln!("PRoot is not installed. Please install PRoot to use this feature.");
    }
}

command! {
    /// UserSU Daemon - Manage root access and system information
    #[command(author, version, about, long_about = None)]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        /// Check and manage root access
        RootCheck,
        /// Display system information
        SysInfo,
    }
}
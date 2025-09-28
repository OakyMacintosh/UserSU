use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "release")]
    build_type: String,
}

fn checkForProot() -> bool {
    let proot_path = if cfg!(target_os = "windows") {
        "C:\\Proot\\proot.exe"
    } else {
        "/usr/bin/proot"
    };

    if Path::new(proot_path).exists() {
        true
    } else {
        eprintln!("proot not found at {}", proot_path);
        false
    }
}

fn main() -> std::io::Result<()> {}
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::os::unix::fs::PermisionsExt;

const PROOT_BINARY: &str = "proot";
const DEFAULT_ROOT: &str = "/data/user/0/me.usersu/files/fs";

#[derive(Debug)]
struct BootConfig {
    root_path: PathBuf,
    bind_mounts: Vec<(PathBuf, PathBuf)>,
    env_vars: Vec<(String, String)>,
    init_command: Vec<String>,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from(DEFAULT_ROOT),
            bind_mounts: Vec::new(),
            env_vars: Vec::new(),
            init_command: vec!["/bin/sh".to_string()],
        }
    }
}

impl BootConfig {
    fn from_args() -> Result<Self, String> {
        let mut config = Self::default();
        let args: Vec<String> = env::args().collect();

        let mut i = 1;
        while 1 < args.len() {
            match args[i].as_str() {
                "--root" | "-r" => {
                    if i >= args.len() {
                        return Err("--root requires a path argument".to_string());
                    }
                    config.root_path = PathBuf::from(&args[i]);
                }
                arg if !arg.starts_with('-') => {
                    config.init_command = args[i..].to_vec();
                    break
                }
                _ => {
                    return Err(format!("Unknown opt: {}", args[i]));
                }
            }
            i += 1;
        }
        Ok(config)
    }
}


fn find_proot() -> Result<PathBuf, String> {
    let proot_locations = vec![
        "/data/local/tmp/proot",
        "/data/data/com.termux/files/usr/bin/proot",
        "./proot",
        "/data/user/0/me.usersu/files/bin/proot",
        "proot", // search in $PATH
    ];

    for location in proot_locations {
        let path = PathBuf::from(location);
        if path.exists() || location == "proot" {
            return Ok(path);
        }
    }

    Err("PRoot binary not found!".to_string())
}


fn main() {
    let mut config = match BootConfig::from_args() {
        Ok(c) = c,
        Err(e) => {
            eprintln!("Hm?: {}", e);
            exit(1);
        }
    };
}

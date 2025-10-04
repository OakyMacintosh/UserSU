use clap::Parser;
use nix::unistd::execvpe;
use std::env;
use std::ffi::{CString, OsStr};
use std::path::PathBuf;
use std::process::exit;

/// usersu — userland "superuser" launcher (safe, illusion only)
#[derive(Parser, Debug)]
#[command(author, version, about = "Launch a program under libfakeroot; does NOT grant real root.")]
struct Opt {
    /// Path to libfakeroot.so (if omitted, looks for ./libfakeroot.so)
    #[arg(short, long)]
    lib: Option<PathBuf>,

    /// Fake root directory to use (defaults to the requested path)
    #[arg(short = 'r', long, default_value = "/data/data/me.usersu/files/rootfs")]
    rootfs: PathBuf,

    /// If set, child will see UID 0 via FAKE_ROOT_UID=1 (userland illusion)
    #[arg(short = 'f', long, default_value_t = false)]
    fake_uid: bool,

    /// Command to exec (required)
    #[arg(required = true)]
    cmd: Vec<String>,
}

fn find_lib(default: &PathBuf) -> Option<PathBuf> {
    if let Some(p) = default.canonicalize().ok() {
        if p.exists() { return Some(p); }
    }
    // try relative ./libfakeroot.so
    let rel = PathBuf::from("libfakeroot.so");
    if rel.exists() {
        return rel.canonicalize().ok();
    }
    None
}

fn main() {
    let opts = Opt::parse();

    // Determine lib path
    let lib_path = opts.lib.unwrap_or_else(|| PathBuf::from("libfakeroot.so"));
    let lib_real = match find_lib(&lib_path) {
        Some(p) => p,
        None => {
            eprintln!("libfakeroot not found at '{}' or './libfakeroot.so'.", lib_path.display());
            eprintln!("Place libfakeroot.so next to this binary or pass --lib /path/to/libfakeroot.so");
            exit(2);
        }
    };

    // Ensure rootfs exists (we will not modify system paths)
    if !opts.rootfs.exists() {
        eprintln!("Warning: FAKE_ROOT directory '{}' does not exist. Creating (user-owned) …", opts.rootfs.display());
        if let Err(e) = std::fs::create_dir_all(&opts.rootfs) {
            eprintln!("Failed to create rootfs directory: {}", e);
            exit(3);
        }
    }

    // Prepare command to exec
    let cmd_path = &opts.cmd[0];
    let cmd_args: Vec<CString> = opts.cmd.iter()
        .map(|s| CString::new(s.as_str()).expect("NUL in arg"))
        .collect();

    // Build env: start from current env, but override/add our variables
    let mut new_env: Vec<(CString, CString)> = env::vars()
        .map(|(k,v)| {
            (CString::new(k).unwrap(), CString::new(v).unwrap())
        })
        .collect();

    // Set LD_PRELOAD to our lib
    let lib_val = lib_real.to_string_lossy().into_owned();
    new_env.push((CString::new("LD_PRELOAD").unwrap(), CString::new(lib_val).unwrap()));

    // Set FAKE_ROOT to the requested rootfs
    let root_val = opts.rootfs.to_string_lossy().into_owned();
    new_env.push((CString::new("FAKE_ROOT").unwrap(), CString::new(root_val).unwrap()));

    // Optionally set FAKE_ROOT_UID
    if opts.fake_uid {
        new_env.push((CString::new("FAKE_ROOT_UID").unwrap(), CString::new("1").unwrap()));
    } else {
        // Ensure it's not set in the child unless present in our env
        // (we'll not explicitly remove other env variables here)
    }

    // Convert to arrays for execvpe
    let c_cmd = CString::new(cmd_path.as_str()).expect("NUL in command");
    let c_args: Vec<&CStr> = cmd_args.iter().map(|s| s.as_c_str()).collect();

    // envp: vector of "KEY=VALUE" CStrings
    let envp_cstrings: Vec<CString> = new_env.into_iter()
        .map(|(k,v)| {
            // build "K=V"
            let mut s = k.into_string().unwrap();
            s.push('=');
            s.push_str(&v.into_string().unwrap());
            CString::new(s).unwrap()
        })
        .collect();
    let envp: Vec<&CStr> = envp_cstrings.iter().map(|s| s.as_c_str()).collect();

    // Exec the command (replace current process)
    match execvpe(&c_cmd, &c_args, &envp) {
        Ok(_) => unreachable!("execvpe succeeded unexpectedly (should not return)"),
        Err(err) => {
            eprintln!("failed to exec {}: {}", cmd_path, err);
            exit(4);
        }
    }
}
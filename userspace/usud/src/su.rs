use std::env;
use std::ffi::CString;
use std::process::exit;
use std::os::unix::prelude::CommandExt;
use std::process::Command;
use std::path::Path;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <path-to-libfakeroot.so> <command> [args...]", args[0]);
        exit(2);
    }

    let lib = &args[1];
    if !Path::new(lib).exists() {
        eprintln!("lib not found: {}", lib);
        exit(2);
    }

    let cmd = &args[2];
    let cmd_args = &args[3..];

    // Where your fake root lives
    let fake_root = env::var("FAKE_ROOT_DIR").unwrap_or_else(|_| "./fakeroot".to_string());

    // Build the Command
    let mut command = Command::new(cmd);
    command.args(cmd_args);

    // Inherit current env, but set LD_PRELOAD and FAKE_ROOT
    command.env("LD_PRELOAD", lib);
    command.env("FAKE_ROOT", &fake_root);
    // If you want programs to think they're root, set this:
    // command.env("FAKE_ROOT_UID", "1");

    // Exec the command in-place (replace current process)
    let err = command.exec(); // if exec returns, it's an error
    eprintln!("failed to exec: {}", err);
    exit(1);
}

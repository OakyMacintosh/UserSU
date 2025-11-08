use clap::{Parser, Subcommand};
use colorize::AnsiColor;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;
use tokio::process::Command as AsyncCommand;

#[derive(Parser)]
#[command(name = "envspoof")]
#[command(author = "Milo/OakyMac")]
#[command(version = "1.0.0")]
#[command(about = "Android Environment Spoofer - Userland root environment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Enter spoofed root shell
    Shell,
    /// Execute command with spoofed environment
    Exec {
        /// Command to execute
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Show current environment status
    Status,
    /// Install psu/psudo aliases
    Install,
}

struct EnvSpoofer {
    spoofed_env: HashMap<String, String>,
    spoofed_paths: Vec<PathBuf>,
    root_fs_path: PathBuf,
}

impl EnvSpoofer {
    fn new() -> Self {
        let root_fs_path = PathBuf::from("/sdcard/SpoofySu");
        
        let mut spoofer = EnvSpoofer {
            spoofed_env: HashMap::new(),
            spoofed_paths: Vec::new(),
            root_fs_path: root_fs_path.clone(),
        };
        
        // Initialize mini root filesystem
        if let Err(e) = spoofer.init_root_filesystem() {
            eprintln!("{}", format!("[!] Warning: Failed to initialize root filesystem: {}", e).yellow());
        }
        
        spoofer.init_environment();
        spoofer
    }

    fn init_environment(&mut self) {
        // Spoof USER and HOME to root (using our mini filesystem)
        self.spoofed_env.insert("USER".to_string(), "root".to_string());
        self.spoofed_env.insert("LOGNAME".to_string(), "root".to_string());
        self.spoofed_env.insert("HOME".to_string(), self.root_fs_path.join("root").to_string_lossy().to_string());
        
        // Spoof shell
        self.spoofed_env.insert("SHELL".to_string(), "/bin/bash".to_string());
        
        // Spoof UID/GID
        self.spoofed_env.insert("UID".to_string(), "0".to_string());
        self.spoofed_env.insert("EUID".to_string(), "0".to_string());
        
        // Add our mini root filesystem paths first
        let mini_paths = vec![
            self.root_fs_path.join("bin"),
            self.root_fs_path.join("sbin"),
            self.root_fs_path.join("usr/bin"),
        ];
        
        for path in mini_paths {
            self.spoofed_paths.push(path);
        }
        
        // Add common root paths
        let root_paths = vec![
            "/sbin",
            "/system/sbin",
            "/system/bin",
            "/system/xbin",
            "/vendor/bin",
            "/data/local/bin",
        ];
        
        for path in root_paths {
            self.spoofed_paths.push(PathBuf::from(path));
        }
        
        // Build spoofed PATH
        let mut path_string = String::new();
        for (i, path) in self.spoofed_paths.iter().enumerate() {
            if i > 0 {
                path_string.push(':');
            }
            path_string.push_str(path.to_str().unwrap_or(""));
        }
        
        // Append original PATH
        if let Ok(orig_path) = env::var("PATH") {
            path_string.push(':');
            path_string.push_str(&orig_path);
        }
        
        self.spoofed_env.insert("PATH".to_string(), path_string);
        
        // Android-specific spoofing
        self.spoofed_env.insert("ANDROID_ROOT".to_string(), "/system".to_string());
        self.spoofed_env.insert("ANDROID_DATA".to_string(), "/data".to_string());
    }

    fn init_root_filesystem(&self) -> io::Result<()> {
        println!("{}", "[*] Initializing mini root filesystem...".cyan());
        
        // Create main directory
        fs::create_dir_all(&self.root_fs_path)?;
        println!("{}", format!("  ✓ Created {}", self.root_fs_path.display()).green());
        
        // Create standard Unix filesystem structure
        let dirs = vec![
            "bin",
            "sbin",
            "etc",
            "root",
            "home",
            "tmp",
            "var",
            "usr/bin",
            "usr/sbin",
            "usr/lib",
            "opt",
            "dev",
            "proc",
            "sys",
        ];
        
        for dir in dirs {
            let path = self.root_fs_path.join(dir);
            fs::create_dir_all(&path)?;
            println!("{}", format!("  ✓ Created /{}", dir).green());
        }
        
        // Create some useful files
        self.create_etc_files()?;
        self.create_bin_scripts()?;
        
        println!("{}", "\n[*] Mini root filesystem initialized successfully!".green().bold());
        println!("{}", format!("    Location: {}", self.root_fs_path.display()).white());
        
        Ok(())
    }
    
    fn create_etc_files(&self) -> io::Result<()> {
        let etc_path = self.root_fs_path.join("etc");
        
        // Create /etc/passwd
        let passwd_content = "root:x:0:0:root:/root:/bin/bash\n";
        fs::write(etc_path.join("passwd"), passwd_content)?;
        
        // Create /etc/group
        let group_content = "root:x:0:\n";
        fs::write(etc_path.join("group"), group_content)?;
        
        // Create /etc/hostname
        fs::write(etc_path.join("hostname"), "spoofysu-android\n")?;
        
        // Create /etc/hosts
        let hosts_content = "127.0.0.1\tlocalhost\n127.0.1.1\tspoofysu-android\n";
        fs::write(etc_path.join("hosts"), hosts_content)?;
        
        println!("{}", "  ✓ Created /etc configuration files".green());
        
        Ok(())
    }
    
    fn create_bin_scripts(&self) -> io::Result<()> {
        let bin_path = self.root_fs_path.join("bin");
        
        // Create a welcome script
        let welcome_script = r#"#!/system/bin/sh
echo "Welcome to SpoofySu Mini Root Environment!"
echo "Author: Milo/OakyMac"
echo ""
echo "Available commands:"
echo "  whoami - Show current user"
echo "  pwd    - Print working directory"
echo "  ls     - List files"
echo ""
"#;
        let welcome_path = bin_path.join("welcome");
        fs::write(&welcome_path, welcome_script)?;
        
        // Set executable permissions
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&welcome_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&welcome_path, perms)?;
        }
        
        // Create a whoami script
        let whoami_script = r#"#!/system/bin/sh
echo "root"
"#;
        let whoami_path = bin_path.join("whoami");
        fs::write(&whoami_path, whoami_script)?;
        
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&whoami_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&whoami_path, perms)?;
        }
        
        println!("{}", "  ✓ Created utility scripts in /bin".green());
        
        Ok(())
    }

    fn print_banner(&self) {
        println!("{}", "\n[*] Userland root environment active".green());
        println!("{}", "[!] boot.img not modified.\n".yellow());
    }

    fn print_status(&self) {
        self.print_banner();
        println!("{}", "Environment Variables:".blue().bold());
        
        let important_vars = vec!["USER", "HOME", "SHELL", "UID", "PATH"];
        for var in important_vars {
            if let Some(value) = self.spoofed_env.get(var) {
                println!("  {} = {}", var.green(), value.white());
            }
        }
        
        println!("\n{}", "Spoofed Root Paths:".blue().bold());
        for path in &self.spoofed_paths {
            let exists = path.exists();
            let status = if exists { "✓".green() } else { "✗".red() };
            println!("  {} {}", status, path.display().to_string().white());
        }
    }

    async fn enter_shell(&self) {
        self.print_banner();
        println!("{}", "Starting spoofed root shell...".green());
        println!("{}", "Type 'exit' to return to normal environment\n".yellow());
        
        // Determine shell to use
        let shell = env::var("SHELL").unwrap_or_else(|_| "/system/bin/sh".to_string());
        
        loop {
            print!("{}", "root@android:~# ".green().bold());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            if input == "exit" {
                println!("{}", "[*] Exiting spoofed environment".yellow());
                break;
            }
            
            self.execute_command_sync(input);
        }
    }

    fn execute_command_sync(&self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        let mut command = Command::new(parts[0]);
        
        if parts.len() > 1 {
            command.args(&parts[1..]);
        }
        
        // Apply spoofed environment
        for (key, value) in &self.spoofed_env {
            command.env(key, value);
        }
        
        match command.stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit())
                    .status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("{}", format!("[!] Command failed with status: {}", status).red());
                }
            }
            Err(e) => {
                eprintln!("{}", format!("[!] Failed to execute: {}", e).red());
            }
        }
    }

    async fn execute_command_async(&self, args: Vec<String>) {
        if args.is_empty() {
            eprintln!("{}", "[!] No command specified".red());
            return;
        }
        
        let mut command = AsyncCommand::new(&args[0]);
        
        if args.len() > 1 {
            command.args(&args[1..]);
        }
        
        // Apply spoofed environment
        for (key, value) in &self.spoofed_env {
            command.env(key, value);
        }
        
        match command.stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit())
                    .status()
                    .await {
            Ok(status) => {
                if !status.success() {
                    eprintln!("{}", format!("[!] Command failed with status: {}", status).red());
                }
            }
            Err(e) => {
                eprintln!("{}", format!("[!] Failed to execute: {}", e).red());
            }
        }
    }

    fn install_aliases(&self) -> io::Result<()> {
        println!("{}", "[*] Installing psu/psudo aliases...".cyan());
        
        let home = env::var("HOME").unwrap_or_else(|_| "/data/local/tmp".to_string());
        let rc_files = vec![".bashrc", ".profile", ".zshrc"];
        
        let alias_content = r#"
# Android Environment Spoofer aliases (Milo/OakyMac)
alias psu='envspoof shell'
alias psudo='envspoof exec'
"#;
        
        for rc_file in rc_files {
            let rc_path = PathBuf::from(&home).join(rc_file);
            if rc_path.exists() {
                let mut content = fs::read_to_string(&rc_path).unwrap_or_default();
                if !content.contains("envspoof") {
                    content.push_str(alias_content);
                    fs::write(&rc_path, content)?;
                    println!("{}", format!("  ✓ Added aliases to {}", rc_file).green());
                } else {
                    println!("{}", format!("  • Aliases already exist in {}", rc_file).yellow());
                }
            }
        }
        
        println!("\n{}", "[*] Installation complete!".green());
        println!("{}", "    Run 'source ~/.bashrc' or restart your shell".yellow());
        println!("\n{}", "Usage:".blue().bold());
        println!("  {} - Enter spoofed root shell", "psu".green());
        println!("  {} - Execute command with root environment", "psudo <command>".green());
        
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let spoofer = EnvSpoofer::new();

    match cli.command {
        Some(Commands::Shell) => {
            spoofer.enter_shell().await;
        }
        Some(Commands::Exec { args }) => {
            spoofer.execute_command_async(args).await;
        }
        Some(Commands::Status) => {
            spoofer.print_status();
        }
        Some(Commands::Install) => {
            if let Err(e) = spoofer.install_aliases() {
                eprintln!("{}", format!("[!] Installation failed: {}", e).red());
            }
        }
        None => {
            spoofer.print_status();
            println!("\n{}", "Usage:".blue().bold());
            println!("  envspoof shell          - Enter spoofed root shell");
            println!("  envspoof exec <cmd>     - Execute command");
            println!("  envspoof status         - Show environment status");
            println!("  envspoof install        - Install psu/psudo aliases");
        }
    }
}
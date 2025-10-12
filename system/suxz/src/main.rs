use clap::{Parser, Subcommand};
use std::path::PathBuf;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::fs;
use std::io::{self, Read, Write, BufRead, BufReader};
use xz2::write::XzEncoder;
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a new Ed25519 key pair
    Keygen {
        /// Output directory for the keys
        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,
    },
    /// Creates a .su.xz file and its signature
    Create {
        /// Input file to compress and sign
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,
        /// Output directory for the .su.xz and .su.xz.sig files
        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,
        /// Path to the private key for signing
        #[arg(short, long, value_name = "FILE")]
        private_key: PathBuf,
    },
    /// Verifies the signature of a .su.xz file
    Verify {
        /// Path to the .su.xz file
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,
        /// Path to the signature file (.su.xz.sig)
        #[arg(short, long, value_name = "FILE")]
        signature: PathBuf,
        /// Path to the public key for verification
        #[arg(short, long, value_name = "FILE")]
        public_key: PathBuf,
    },
    /// Starts an interactive session
    Interactive,
}

fn generate_keys(output_dir: Option<PathBuf>) -> io::Result<()> {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = (&signing_key).into();

    let private_key_bytes = signing_key.to_bytes();
    let public_key_bytes = verifying_key.to_bytes();

    let output_path = output_dir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_path)?;

    let private_key_path = output_path.join("private.key");
    let public_key_path = output_path.join("public.key");

    let mut private_key_file = fs::File::create(&private_key_path)?;
    private_key_file.write_all(&private_key_bytes)?;

    let mut public_key_file = fs::File::create(&public_key_path)?;
    public_key_file.write_all(&public_key_bytes)?;

    println!("{}", format!("Private key saved to: {}", private_key_path.display()).green());
    println!("{}", format!("Public key saved to: {}", public_key_path.display()).green());

    Ok(())
}

fn create_su_xz(
    input_path: &PathBuf,
    output_dir: Option<PathBuf>,
    private_key_path: &PathBuf,
) -> io::Result<()> {
    // Read input file
    let mut input_file = fs::File::open(input_path)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;

    // Compress data
    let mut encoder = XzEncoder::new(Vec::new(), 9);
    encoder.write_all(&input_data)?;
    let compressed_data = encoder.finish()?;

    // Determine output paths
    let output_path = output_dir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_path)?;

    let file_name = input_path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid input file name"))?;
    let su_xz_path = output_path.join(format!("{}.su.xz", file_name.to_string_lossy()));
    let signature_path = output_path.join(format!("{}.su.xz.sig", file_name.to_string_lossy()));

    // Write .su.xz file
    let mut su_xz_file = fs::File::create(&su_xz_path)?;
    su_xz_file.write_all(&compressed_data)?;
    println!("{}", format!(".su.xz file created at: {}", su_xz_path.display()).green());

    // Load private key and sign
    let private_key_bytes = fs::read(private_key_path)?;
    let signing_key_bytes: [u8; 32] = private_key_bytes.as_slice().try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Private key must be 32 bytes long"))?;
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);

    let signature = signing_key.sign(&compressed_data);

    // Write signature file
    let mut signature_file = fs::File::create(&signature_path)?;
    signature_file.write_all(signature.to_bytes().as_ref())?;
    println!("{}", format!("Signature file created at: {}", signature_path.display()).green());

    Ok(())
}

fn verify_su_xz(
    input_path: &PathBuf,
    signature_path: &PathBuf,
    public_key_path: &PathBuf,
) -> io::Result<()> {
    // Read .su.xz file
    let compressed_data = fs::read(input_path)?;

    // Read signature file
    let signature_bytes = fs::read(signature_path)?;
    let signature_array: [u8; 64] = signature_bytes.as_slice().try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Signature must be 64 bytes long"))?;
    let signature = Signature::from_bytes(&signature_array);

    // Load public key
    let public_key_bytes = fs::read(public_key_path)?;
    let public_key_array: [u8; 32] = public_key_bytes.as_slice().try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Public key must be 32 bytes long"))?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid public key: {}", e)))?;

    // Verify signature
    match verifying_key.verify(&compressed_data, &signature) {
        Ok(_) => {
            println!("{}", format!("Signature verified successfully for: {}", input_path.display()).green());
            Ok(())
        }
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::InvalidData, format!("Signature verification failed: {}", e)))
        }
    }
}

fn interactive_session() -> io::Result<()> {
    println!("{}", "Starting interactive session. Type 'help' for commands, 'exit' to quit.".blue());
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut line = String::new();

    loop {
        print!("{}", "suxz> ".blue());
        io::stdout().flush()?;
        line.clear();
        reader.read_line(&mut line)?;
        let trimmed_line = line.trim();

        if trimmed_line == "exit" {
            println!("{}", "Exiting interactive session.".blue());
            break;
        } else if trimmed_line == "help" {
            println!("{}", "Available commands:".yellow());
            println!("  keygen [-o <DIR>]");
            println!("  create -i <FILE> -p <PRIVATE_KEY_FILE> [-o <DIR>]");
            println!("  verify -i <FILE> -s <SIGNATURE_FILE> -p <PUBLIC_KEY_FILE>");
            println!("  exit");
            println!("  help");
        } else {
            let args = shlex::split(trimmed_line).unwrap_or_else(|| {
                eprintln!("{}", "Error parsing command.".red());
                vec![]
            });

            if args.is_empty() {
                continue;
            }

            let cli = Cli::try_parse_from(std::iter::once("suxz").chain(args.iter().map(|s| s.as_str())));

            match cli {
                Ok(cli) => {
                    match &cli.command {
                        Commands::Keygen { output } => {
                            println!("{}", "Generating keys...".blue());
                            if let Err(e) = generate_keys(output.clone()) {
                                eprintln!("{}{}", "Error generating keys: ".red(), e.red());
                            }
                        }
                        Commands::Create { input, output, private_key } => {
                            println!("{}", format!("Creating .su.xz for {:?} with private key {:?}...", input, private_key).blue());
                            if let Err(e) = create_su_xz(input, output.clone(), private_key) {
                                eprintln!("{}{}", "Error creating .su.xz file: ".red(), e.red());
                            }
                        }
                        Commands::Verify { input, signature, public_key } => {
                            println!("{}", format!("Verifying {:?} with signature {:?} and public key {:?}...", input, signature, public_key).blue());
                            if let Err(e) = verify_su_xz(input, signature, public_key) {
                                eprintln!("{}{}", "Error verifying .su.xz file: ".red(), e.red());
                            }
                        }
                        Commands::Interactive => {
                            eprintln!("{}", "Already in interactive mode.".yellow());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}{}", "Error parsing command: ".red(), e.to_string().red());
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keygen { output } => {
            println!("{}", "Generating keys...".blue());
            if let Err(e) = generate_keys(output.clone()) {
                eprintln!("{}{}", "Error generating keys: ".red(), e.red());
            }
        }
        Commands::Create { input, output, private_key } => {
            println!("{}", format!("Creating .su.xz for {:?} with private key {:?}...", input, private_key).blue());
            if let Err(e) = create_su_xz(input, output.clone(), private_key) {
                eprintln!("{}{}", "Error creating .su.xz file: ".red(), e.red());
            }
        }
        Commands::Verify { input, signature, public_key } => {
            println!("{}", format!("Verifying {:?} with signature {:?} and public key {:?}...", input, signature, public_key).blue());
            if let Err(e) = verify_su_xz(input, signature, public_key) {
                eprintln!("{}{}", "Error verifying .su.xz file: ".red(), e.red());
            }
        }
        Commands::Interactive => {
            if let Err(e) = interactive_session() {
                eprintln!("{}{}", "Interactive session error: ".red(), e.red());
            }
        }
    }
}
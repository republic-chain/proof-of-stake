use clap::{Arg, Command};
use production_pos::{crypto::KeyPair};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("validator")
        .version("0.1.0")
        .about("Production PoS validator utilities")
        .subcommand(
            Command::new("generate-keys")
                .about("Generate validator keypair")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file for the private key")
                        .default_value("validator_key.json"),
                ),
        )
        .subcommand(
            Command::new("show-address")
                .about("Show validator address from private key")
                .arg(
                    Arg::new("keyfile")
                        .short('k')
                        .long("keyfile")
                        .value_name("FILE")
                        .help("Private key file")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("register")
                .about("Register as a validator")
                .arg(
                    Arg::new("keyfile")
                        .short('k')
                        .long("keyfile")
                        .value_name("FILE")
                        .help("Private key file")
                        .required(true),
                )
                .arg(
                    Arg::new("stake")
                        .short('s')
                        .long("stake")
                        .value_name("AMOUNT")
                        .help("Initial stake amount")
                        .required(true),
                )
                .arg(
                    Arg::new("commission")
                        .short('c')
                        .long("commission")
                        .value_name("RATE")
                        .help("Commission rate in basis points (e.g., 500 = 5%)")
                        .default_value("500"),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("Validator name")
                        .required(true),
                ),
        )
        .get_matches();

    tracing_subscriber::fmt().init();

    match matches.subcommand() {
        Some(("generate-keys", sub_matches)) => {
            let output_file = sub_matches.get_one::<String>("output").unwrap();
            generate_validator_keys(output_file).await?;
        }
        Some(("show-address", sub_matches)) => {
            let keyfile = sub_matches.get_one::<String>("keyfile").unwrap();
            show_validator_address(keyfile).await?;
        }
        Some(("register", sub_matches)) => {
            let keyfile = sub_matches.get_one::<String>("keyfile").unwrap();
            let stake = sub_matches.get_one::<String>("stake").unwrap();
            let commission = sub_matches.get_one::<String>("commission").unwrap();
            let name = sub_matches.get_one::<String>("name").unwrap();

            register_validator(keyfile, stake, commission, name).await?;
        }
        _ => {
            error!("No subcommand provided. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn generate_validator_keys(output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating new validator keypair...");

    let keypair = KeyPair::generate();

    let key_data = serde_json::json!({
        "private_key": hex::encode(keypair.private_key),
        "public_key": hex::encode(keypair.public_key),
        "address": keypair.address.to_string(),
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    std::fs::write(output_file, serde_json::to_string_pretty(&key_data)?)?;

    info!("Validator keypair generated successfully!");
    info!("Private key saved to: {}", output_file);
    info!("Public key: {}", hex::encode(keypair.public_key));
    info!("Address: {}", keypair.address);
    info!("");
    info!("⚠️  IMPORTANT: Keep your private key file secure and make backups!");
    info!("⚠️  Never share your private key with anyone!");

    Ok(())
}

async fn show_validator_address(keyfile: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Reading validator key from: {}", keyfile);

    let key_content = std::fs::read_to_string(keyfile)?;
    let key_data: serde_json::Value = serde_json::from_str(&key_content)?;

    let private_key_hex = key_data["private_key"]
        .as_str()
        .ok_or("Invalid key file format")?;

    let keypair = KeyPair::from_hex(private_key_hex)?;

    info!("Validator Information:");
    info!("Address: {}", keypair.address);
    info!("Public Key: {}", hex::encode(keypair.public_key));

    Ok(())
}

async fn register_validator(
    keyfile: &str,
    stake: &str,
    commission: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Registering validator...");

    let key_content = std::fs::read_to_string(keyfile)?;
    let key_data: serde_json::Value = serde_json::from_str(&key_content)?;

    let private_key_hex = key_data["private_key"]
        .as_str()
        .ok_or("Invalid key file format")?;

    let keypair = KeyPair::from_hex(private_key_hex)?;
    let stake_amount: u64 = stake.parse()?;
    let commission_rate: u16 = commission.parse()?;

    info!("Validator Details:");
    info!("  Address: {}", keypair.address);
    info!("  Name: {}", name);
    info!("  Stake: {} tokens", stake_amount);
    info!("  Commission: {}% ({} basis points)", commission_rate as f64 / 100.0, commission_rate);

    // In a real implementation, this would:
    // 1. Create a validator registration transaction
    // 2. Sign it with the validator's private key
    // 3. Submit it to the network
    // 4. Wait for confirmation

    info!("✅ Validator registration transaction created");
    info!("📤 Submit this transaction to the network to complete registration");

    // For demonstration, we'll just show what the transaction would look like
    let registration_tx = serde_json::json!({
        "type": "validator_registration",
        "validator_key": hex::encode(keypair.public_key),
        "commission_rate": commission_rate,
        "minimum_stake": stake_amount,
        "metadata": {
            "name": name,
            "website": null,
            "description": null,
            "contact": null
        }
    });

    info!("Transaction data:");
    info!("{}", serde_json::to_string_pretty(&registration_tx)?);

    Ok(())
}
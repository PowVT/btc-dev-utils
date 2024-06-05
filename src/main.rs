use std::process::Command;
use std::time::Duration;
use std::path::PathBuf;
use std::{thread, fs};

use serde_json::Value;
use serde_json::json;
use clap::Parser;
use log::{error, info};

use crate::settings::Settings;

mod settings;

const BTC_REGTEST: &str = "~/bitcoin/src/bitcoin-cli -regtest -rpcuser=user -rpcpassword=password ";
const FEE_RATE: i32 = 15;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "settings.toml")]
    settings_file: PathBuf,

    #[command(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    SignTx,
    InscribeOrd
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Cli::parse();

    let settings = match Settings::from_toml_file(&args.settings_file) {
        Ok(settings) => settings,
        Err(e) => {
            error!("Error reading settings file: {}", e);
            info!(
                "Creating a new settings file at {}",
                args.settings_file.display()
            );
            let settings = Settings::default();
            settings.to_toml_file(&args.settings_file)?;
            settings
        }
    };

    match args.action {
        Action::SignTx => regtest_sign_tx()?,
        Action::InscribeOrd => regtest_inscribe_ord()?,
    }

    Ok(())
}

fn run_command(command: &str, include_btc_core_regtest: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut full_command = String::from(command);
    if include_btc_core_regtest {
        full_command = format!("{}{}", BTC_REGTEST, command);
    }
    let output = Command::new("sh")
                         .arg("-c")
                         .arg(&full_command)
                         .output()?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}\nError: {}", full_command, error_message).into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout)
}

fn regtest_sign_tx() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wallet...");
    run_command("-named createwallet wallet_name=\"regtest_desc_wallet\" descriptors=true", true)?;

    println!("Generating mining address...");
    let mining_address = run_command("getnewaddress", true)?;

    println!("Mining blocks...");
    run_command(&("generatetoaddress 101 ".to_string() + &mining_address), true)?;
    std::thread::sleep(Duration::from_secs(2));

    let balance_str: String = run_command("getbalance", true)?;
    let balance: f64 = balance_str.parse()?;
    println!("Wallet balance: {} BTC", balance);

    if balance < 50.0 {
        return Err("Failed to mine sufficient balance".into());
    }

    println!("Generating recipient address...");
    let recipient_address = run_command("getnewaddress", true)?;

    println!("Creating raw transaction...");
    let unspent_str = run_command("listunspent 1 9999999", true)?;
    let unspent: Value = serde_json::from_str(&unspent_str)?;
    let unspent_txid = &unspent[0]["txid"];
    let unspent_vout = &unspent[0]["vout"];
    let inputs = json!([{"txid": unspent_txid, "vout": unspent_vout}]).to_string();
    let outputs = format!(r#"{{"{}": 49.9999}}"#, recipient_address);
    let raw_tx = run_command(&format!("createrawtransaction '{}' '{}'", inputs, outputs), true)?;
    println!("{}", raw_tx);

    println!("Signing raw transaction...");
    let signed_tx_str = run_command(&format!("signrawtransactionwithwallet {}", raw_tx), true)?;
    let signed_tx: Value = serde_json::from_str(&signed_tx_str)?;
    let signed_raw_tx = &signed_tx["hex"];
    if !signed_tx["complete"].as_bool().unwrap_or(false) {
        return Err("Failed to sign the transaction".into());
    }
    println!("Signed raw transaction: {}", signed_raw_tx);

    let fee = balance - 49.9999;
    let raw_tx_size = (signed_raw_tx.as_str().unwrap().len() / 2) as f64;
    let fee_rate = (fee * 1e8) / (raw_tx_size / 1000.0);
    println!("Fee: {} BTC", fee);
    println!("Fee rate: {} sats/vB", fee_rate);
    println!("Fee rate: {} BTC/kvB", fee_rate / 1e8 * 1000.0);

    println!("Mining blocks...");
    for _ in 0..50 {
        run_command(&format!("generatetoaddress 1 {}", mining_address), true)?;
        std::thread::sleep(Duration::from_secs(3));
    }

    let balance_str = run_command("listaddressgroupings", true)?;
    let balance: Value = serde_json::from_str(&balance_str)?;
    println!("Wallet balances: {}", balance);

    run_command("rm -rf ../Library/Application\\ Support/Bitcoin/regtest", false)?;

    Ok(())
}

fn regtest_inscribe_ord() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wallet...");
    run_command("../ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password wallet create", false);

    println!("Generating mining address...");
    let result = run_command("../ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password wallet receive", false);
    let json_str = result?;
    let value: Value = serde_json::from_str(&json_str)?;
    let mining_address: String = value["addresses"][0].as_str().ok_or("No address found")?.to_string();
    println!("{}",mining_address);

    println!("Mining blocks...");
    run_command(&format!("-regtest generatetoaddress 101 {}", mining_address), true);
    thread::sleep(Duration::from_secs(2));

    let balance_output = run_command("-regtest -rpcwallet=ord getbalance", true)?;
    let balance: f64 = balance_output.trim().parse()?;
    println!("Wallet balance: {} BTC", balance);

    if balance < 50.0 {
        panic!("Failed to mine sufficient balance");
    }

    println!("Creating inscription...");
    run_command(&format!("../ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password wallet inscribe --fee-rate {}  --file ./mockOrdContent.txt", FEE_RATE), false);

    run_command(&format!("-regtest generatetoaddress 10 {}", mining_address), true);
    thread::sleep(Duration::from_secs(10));

    let inscriptions = run_command("../ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password wallet inscriptions", false);
    println!("Inscription Data: {:?}", inscriptions);

    let balance_output = run_command("-regtest listaddressgroupings", true)?;
    let balance_str = balance_output.trim();
    let balance: serde_json::Value = serde_json::from_str(balance_str)?;
    println!("Wallet bitcoin balances: {:?}", balance);

    let ord_balances = run_command("../ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password wallet balance", false);
    println!("Wallet ordinal balance: {:?}", ord_balances);

    Ok(())
}
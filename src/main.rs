use std::{thread, time::Duration, path::PathBuf};

use clap::Parser;
use log::{error, info};
use serde_json::{json, Value};

mod settings;
mod utils;

use crate::settings::Settings;
use crate::utils::{Target, run_command};

const MINING_BLOCKS: i32 = 101;
const MIN_BALANCE: f64 = 50.0;
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
            info!("Creating a new settings file at {}", args.settings_file.display());
            let settings = Settings::default();
            settings.to_toml_file(&args.settings_file)?;
            settings
        }
    };

    match args.action {
        Action::SignTx => regtest_sign_tx(&settings),
        Action::InscribeOrd => regtest_inscribe_ord(&settings),
    }
}

fn regtest_sign_tx(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wallet...");
    run_command("-named createwallet wallet_name=\"regtest_desc_wallet\" descriptors=true", Target::Bitcoin, settings);

    println!("Generating mining address...");
    let mining_address = run_command("getnewaddress", Target::Bitcoin, settings);

    println!("Mining blocks...");
    run_command(&format!("generatetoaddress {} {}", MINING_BLOCKS, mining_address), Target::Bitcoin, settings);
    thread::sleep(Duration::from_secs(2));

    let balance_str: String = run_command("getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_str.parse()?;
    println!("Wallet balance: {} BTC", balance);

    if balance < MIN_BALANCE {
        return Err(format!("Failed to mine sufficient balance. Current balance: {}", balance).into());
    }

    println!("Generating recipient address...");
    let recipient_address = run_command("getnewaddress", Target::Bitcoin, settings);

    println!("Creating raw transaction...");
    let unspent_str = run_command("listunspent 1 9999999", Target::Bitcoin, settings);
    let unspent: Value = serde_json::from_str(&unspent_str)?;
    let unspent_txid = &unspent[0]["txid"];
    let unspent_vout = &unspent[0]["vout"];
    let inputs = json!([{"txid": unspent_txid, "vout": unspent_vout}]).to_string();
    let outputs = format!(r#"{{"{}": 49.9999}}"#, recipient_address);
    let raw_tx = run_command(&format!("createrawtransaction '{}' '{}'", inputs, outputs), Target::Bitcoin, settings);
    println!("{}", raw_tx);

    println!("Signing raw transaction...");
    let signed_tx_str = run_command(&format!("signrawtransactionwithwallet {}", raw_tx), Target::Bitcoin, settings);
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
        run_command(&format!("generatetoaddress 1 {}", mining_address), Target::Bitcoin, settings);
        std::thread::sleep(Duration::from_secs(3));
    }

    let balance_str = run_command("listaddressgroupings", Target::Bitcoin, settings);
    let balance: Value = serde_json::from_str(&balance_str)?;
    println!("Wallet balances: {}", balance);

    Ok(())
}

fn regtest_inscribe_ord(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wallet...");
    run_command("wallet create", Target::Ord, settings);

    println!("Generating mining address...");
    let json_str = run_command("wallet receive", Target::Ord, settings);
    let value: Value = serde_json::from_str(&json_str)?;
    let mining_address: String = value["addresses"][0].as_str().ok_or("No address found")?.to_string();

    println!("Mining blocks...");
    run_command(&format!("generatetoaddress 101 {}", mining_address), Target::Bitcoin, settings);
    thread::sleep(Duration::from_secs(2));

    let balance_output = run_command("-rpcwallet=ord getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_output.trim().parse()?;
    println!("Wallet balance: {} BTC", balance);

    if balance < MIN_BALANCE {
        panic!("Failed to mine sufficient balance");
    }

    println!("Creating inscription...");
    run_command(&format!("wallet inscribe --fee-rate {}  --file ./mockOrdContent.txt", FEE_RATE), Target::Ord, settings);

    run_command(&format!("generatetoaddress 10 {}", mining_address), Target::Bitcoin, settings);
    thread::sleep(Duration::from_secs(10));

    let inscriptions = run_command("wallet inscriptions", Target::Ord, settings);
    println!("Inscription Data: {:?}", inscriptions);

    let balance_output = run_command("listaddressgroupings", Target::Bitcoin, settings);
    let balance_str = balance_output.trim();
    let balance: serde_json::Value = serde_json::from_str(balance_str)?;
    println!("Wallet bitcoin balances: {:?}", balance);

    let ord_balances = run_command("wallet balance", Target::Ord, settings);
    println!("Wallet ordinal balance: {:?}", ord_balances);

    Ok(())
}
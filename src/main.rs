use std::collections::HashMap;
use std::{fs, thread, time::Duration, path::PathBuf, path::Path};

use bitcoin::consensus::serialize;
use bitcoin::{Amount, Transaction};
use bitcoincore_rpc::json::CreateRawTransactionInput;
use clap::Parser;
use log::{error, info};
use serde_json::Value;

use crate::settings::Settings;
use crate::utils::{Target, run_command, parse_amount};
use crate::wallet::Wallet;

mod settings;
mod utils;
mod wallet;

const MIN_BALANCE: f64 = 50.0;
const FEE_RATE: i32 = 15;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "settings.toml")]
    settings_file: PathBuf,

    /// Name of the wallet
    #[arg(short, long, default_value = "miner")]
    wallet_name: String,

    /// Number of blocks to mine
    #[arg(short, long, default_value = "10")]
    blocks: u64,

    /// Transaction recipient address
    #[arg(short, long, default_value = "")]
    recipient: String,

    /// Transaction amount
    #[arg(short, long, value_parser = parse_amount, default_value = "1.0")]
    amount: Amount,

    /// Transaction fee
    #[arg(short, long, value_parser = parse_amount, default_value = "0.00015")]
    fee_amount: Amount,

    #[command(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    NewWallet,
    NewWalletAddress,
    GetBalance,
    MineToAddress,
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
        Action::NewWallet => new_wallet(&args.wallet_name, &settings),
        Action::NewWalletAddress => get_new_address(&args.wallet_name, &settings),
        Action::GetBalance => get_balance(&args.wallet_name, &settings),
        Action::MineToAddress => mine_to_address(&args.wallet_name, args.blocks, &settings),
        Action::SignTx => sign_tx(&args.wallet_name, args.recipient,  args.amount, args.fee_amount, &settings),
        Action::InscribeOrd => regtest_inscribe_ord(&settings),
    }
}

fn new_wallet(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    Wallet::new(wallet_name, settings);
    
    Ok(())
}

fn get_new_address(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let address = wallet.get_new_address();

    println!("{}",format!("{:?}", address));

    Ok(())
}

fn get_balance(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = Wallet::new(wallet_name, settings);

    let balance = miner_wallet.get_balance();
    println!("{}",format!("{:?}", balance));

    Ok(())
}

fn mine_to_address(wallet_name: &str, blocks: u64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = Wallet::new(wallet_name, settings);

    let address = miner_wallet.get_new_address()?;

    miner_wallet.mine_to_address(&address, Some(blocks))?;

    println!("Mined {} blocks to address: {}", blocks, address);

    Ok(())
}

fn sign_tx(wallet_name: &str, recipient: String, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);
    let balance = wallet.get_balance()?;

    // check if balance is sufficient
    if balance < amount {
        panic!("Insufficient balance to send tx. Current balance: {}", balance);
    }

    println!("Creating raw transaction...");
    let unspent_txs = wallet.list_unspent()?;
    if unspent_txs.is_empty() {
        panic!("No unspent transactions");
    }

    // get the first unspent transaction
    let unspent_txid = unspent_txs[0].txid;
    let unspent_vout = unspent_txs[0].vout;
    let unspent_amount = unspent_txs[0].amount;

    if unspent_amount < amount + fee_amount {
        panic!("Insufficient unspent amount. Current amount: {}", unspent_amount);
    }

    // array of utxos to spend
    let mut utxos: Vec<CreateRawTransactionInput> = Vec::new();
    utxos.push(bitcoincore_rpc::json::CreateRawTransactionInput {
        txid: unspent_txid,
        vout: unspent_vout,
        sequence: Some(0),
    });

    let mut outputs: HashMap<String, Amount> = HashMap::new();
    outputs.insert(recipient.clone(), amount);

    // Create raw transaction
    let raw_tx: Transaction = wallet.create_raw_transaction(&utxos, &outputs, None, None)?;

    // Serialize the transaction
    let raw_tx_hex = serialize(&raw_tx);
    let raw_tx_hex_str = hex::encode(&raw_tx_hex);

    println!("Raw transaction (hex): {:?}", raw_tx_hex_str);

    println!("Signing raw transaction...");
    let signed_tx_str = run_command(&format!("-rpcwallet='{}' signrawtransactionwithwallet {}", wallet_name, raw_tx_hex_str), Target::Bitcoin, settings);
    let signed_tx: Value = serde_json::from_str(&signed_tx_str)?;
    let signed_raw_tx = &signed_tx["hex"];
    if !signed_tx["complete"].as_bool().unwrap_or(false) {
        return Err("Failed to sign the transaction".into());
    }
    println!("Signed raw transaction: {}", signed_raw_tx);

    Ok(())
}

fn regtest_inscribe_ord(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    // Check if wallet already exists
    if !Path::new("data/bitcoin/regtest/wallets/ord").exists() {
        fs::create_dir_all("data/bitcoin/regtest/wallets/ord")?;

        println!("Creating wallet...");
        run_command("wallet create", Target::Ord, settings);
    } else {
        println!("Wallet already exists, using existing wallet.");
    }

    println!("Generating mining address...");
    let json_str = run_command("wallet receive", Target::Ord, settings);
    let value: Value = serde_json::from_str(&json_str)?;
    let mining_address: String = value["addresses"][0].as_str().ok_or("No address found")?.to_string();

    // Mine blocks only if balance is insufficient
    let balance_output = run_command("-rpcwallet=ord getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_output.trim().parse()?;
    if balance < MIN_BALANCE {
        println!("Mining blocks...");
        run_command(&format!("generatetoaddress 101 {}", mining_address), Target::Bitcoin, settings);
        thread::sleep(Duration::from_secs(2));
    }

    let balance_output = run_command("-rpcwallet=ord getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_output.trim().parse()?;
    println!("Wallet balance: {} BTC", balance);

    if balance < MIN_BALANCE {
        panic!("Failed to mine sufficient balance");
    }

    // Create inscription
    println!("Creating inscription...");
    run_command(&format!("wallet inscribe --fee-rate {}  --file ./mockOrdContent.txt", FEE_RATE), Target::Ord, settings);

    run_command(&format!("generatetoaddress 10 {}", mining_address), Target::Bitcoin, settings);
    thread::sleep(Duration::from_secs(10));

    let inscriptions = run_command("wallet inscriptions", Target::Ord, settings);
    println!("Inscription Data: {:?}", inscriptions);

    let balance_output = run_command("-rpcwallet=ord listaddressgroupings", Target::Bitcoin, settings);
    let balance_str = balance_output.trim();
    let balance: serde_json::Value = serde_json::from_str(balance_str)?;
    println!("Wallet bitcoin balances: {:?}", balance);

    let ord_balances = run_command("wallet balance", Target::Ord, settings);
    println!("Wallet ordinal balance: {:?}", ord_balances);

    Ok(())
}
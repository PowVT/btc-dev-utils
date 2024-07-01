use std::collections::HashMap;
use std::str::FromStr;
use std::{fs, thread, time::Duration, path::PathBuf, path::Path};

use bitcoin::consensus::serialize;
use bitcoin::{Address, Amount, Transaction};
use bitcoincore_rpc::json::{CreateRawTransactionInput, GetDescriptorInfoResult};
use bitcoincore_rpc::{RawTx, RpcApi};
use clap::Parser;
use log::{error, info};
use serde_json::{json, Value};
use utils::{extract_int_ext_xpubs, parse_address_type, string_to_address};

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
    #[arg(short, long, default_value = "default_wallet")]
    wallet_name: String,

    /// Name of the multisig wallet
    #[arg(short='m', long, default_value = "multisig_wallet")]
    multisig_name: String,

    /// list of wallet names
    #[arg(short='v', long, value_delimiter = ',', default_value = "default_wallet1,default_wallet2,default_wallet3")]
    wallet_names: Vec<String>,

    /// required number of signatures
    #[arg(short='n', long, default_value = "2")]
    required_signatures: u32,

    /// Address type
    #[arg(short='z', long, value_parser = parse_address_type, default_value = "bech32")]
    address_type: bitcoincore_rpc::json::AddressType,

    /// Number of blocks to mine
    #[arg(short, long, default_value = "1")]
    blocks: u64,

    /// Transaction recipient address
    #[arg(short, long, value_parser = string_to_address, default_value = "1F1tAaz5x1HUXrCNLbtMDqcw6o5GNn4xqX")] // dummy address, do not use
    recipient: Address,

    /// Wallet address
    #[arg(short, long, value_parser = string_to_address, default_value = "1F1tAaz5x1HUXrCNLbtMDqcw6o5GNn4xqX")] // dummy address, do not use
    address: Address,

    /// Transaction amount
    #[arg(short='x', long, value_parser = parse_amount, default_value = "49.9")]
    amount: Amount,

    /// Transaction fee
    #[arg(short, long, value_parser = parse_amount, default_value = "0.1")]
    fee_amount: Amount,

    /// Transaction hash
    #[arg(short, long, default_value = "c36d0c020577c2703dc0e202d8f1ac2626d29d81c449f81079b60c6b07263166")] // dummy tx, do not use
    txid: String,

    #[command(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    NewWallet,
    GetWalletInfo,
    ListDescriptors,
    NewMultisig,
    GetNewAddress,
    GetAddressInfo,
    GetBalance,
    MineBlocks,
    ListUnspent,
    GetTx,
    SignTx,
    SignAndBroadcastTx,
    SendBtc,
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
        Action::GetWalletInfo => get_wallet_info(&args.wallet_name, &settings),
        Action::ListDescriptors => list_descriptors_wrapper(&args.wallet_name, &settings),
        Action::NewMultisig=> new_multisig_wallet(args.required_signatures, &args.wallet_names, &args.multisig_name, &settings),
        Action::GetNewAddress => get_new_address(&args.wallet_name, &args.address_type, &settings),
        Action::GetAddressInfo => get_address_info(&args.wallet_name, &args.address, &settings),
        Action::GetBalance => get_balance(&args.wallet_name, &settings),
        Action::MineBlocks => mine_blocks(&args.wallet_name, args.blocks, &settings),
        Action::ListUnspent => list_unspent(&args.wallet_name, &settings),
        Action::GetTx => get_tx(&args.wallet_name, &args.txid, &settings),
        Action::SignTx => sign_tx_wrapper(&args.wallet_name, &args.recipient,  args.amount, args.fee_amount, &settings),
        Action::SignAndBroadcastTx => sign_and_broadcast_tx(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, &settings),
        Action::SendBtc => send_btc(&args.wallet_name, &args.recipient, args.amount, &settings),
        Action::InscribeOrd => regtest_inscribe_ord(&settings),
    }
}

fn new_wallet(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    Wallet::new(wallet_name, settings);
    
    Ok(())
}

fn get_wallet_info(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let wallet_info = wallet.get_wallet_info()?;
    let wallet_info_pretty = serde_json::to_string_pretty(&wallet_info)?;
    println!("{}", wallet_info_pretty);

    Ok(())
}

fn list_descriptors(wallet_name: &str, settings: &Settings) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = Wallet::create_rpc_client(settings, Some(wallet_name));

    let descriptors: serde_json::Value = client.call("listdescriptors", &[])?;

    Ok(descriptors)
}

fn list_descriptors_wrapper(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let descriptors = list_descriptors(wallet_name, settings)?;
    let descriptors_pretty = serde_json::to_string_pretty(&descriptors)?;
    println!("{}", descriptors_pretty);

    Ok(())
}

fn get_new_address(wallet_name: &str, address_type: &bitcoincore_rpc::json::AddressType, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let address = wallet.new_wallet_address(address_type)?;

    println!("{}",format!("{:?}", address));

    Ok(())
}

fn get_address_info(wallet_name: &str, address: &Address, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = Wallet::create_rpc_client(settings, Some(wallet_name));

    let address_info = client.get_address_info(address)?;
    let address_info_pretty = serde_json::to_string_pretty(&address_info)?;
    println!("{}", address_info_pretty);

    Ok(())
}

fn new_multisig_wallet(nrequired: u32, wallet_names: &Vec<String>, multisig_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    // if the length of wallet_names is gt or equal to nrequired, error
    if wallet_names.len() < nrequired as usize {
        return Err("Error: More required signers than wallets".into());
    }

    let mut xpubs: HashMap<String, String> = HashMap::new();

    // Create the descriptor wallets
    for wallet_name in wallet_names {
        let _ = new_wallet(wallet_name, settings);
    }

    // Extract the xpub of each wallet
    for (i, wallet_name) in wallet_names.iter().enumerate() {
        let descriptors: serde_json::Value = list_descriptors(wallet_name, settings)?;
        let descriptors_array: &Vec<serde_json::Value> = descriptors["descriptors"].as_array().unwrap();

        // Find the correct descriptors for external and internal xpubs
        xpubs = extract_int_ext_xpubs(xpubs, descriptors_array.clone(), i)?;
    }

    // Define the multisig descriptors
    let num_signers = nrequired.to_string();
    let external_desc = format!(
        "wsh(sortedmulti({}, {}, {}, {}))",
        num_signers, xpubs["external_xpub_1"], xpubs["external_xpub_2"], xpubs["external_xpub_3"]
    );
    let internal_desc = format!(
        "wsh(sortedmulti({}, {}, {}, {}))",
        num_signers, xpubs["internal_xpub_1"], xpubs["internal_xpub_2"], xpubs["internal_xpub_3"]
    );

    // Create RPC client without wallet name for general operations
    let client = Wallet::create_rpc_client(settings, None);

    // Get descriptor information
    let external_desc_info: GetDescriptorInfoResult = client.get_descriptor_info(&external_desc)?;
    let internal_desc_info: GetDescriptorInfoResult = client.get_descriptor_info(&internal_desc)?;

    // Extract the descriptors
    let external_descriptor: String = external_desc_info.descriptor;
    let internal_descriptor: String = internal_desc_info.descriptor;

    let multisig_ext_desc = json!({
        "desc": external_descriptor,
        "active": true,
        "internal": false,
        "timestamp": json!("now")
    });

    let multisig_int_desc = json!({
        "desc": internal_descriptor,
        "active": true,
        "internal": true,
        "timestamp": json!("now")
    });

    let multisig_desc = json!([multisig_ext_desc, multisig_int_desc]);  // Create an array with the JSON objects
    println!("Multisig descriptor: {}", serde_json::to_string_pretty(&multisig_desc)?);

    // Create the multisig wallet
    let _ = client.create_wallet(multisig_name, Some(true), Some(true), None, None);

    // import the descriptors
    let multisig_desc_vec: Vec<serde_json::Value> = serde_json::from_value(multisig_desc)?;
    let client2 = Wallet::create_rpc_client(settings, Some(multisig_name));
    client2.call::<serde_json::Value>("importdescriptors", &[json!(multisig_desc_vec)])?;

    // Get wallet info
    let wallet_info = get_wallet_info(multisig_name, settings)?;
    println!("{}", serde_json::to_string_pretty(&wallet_info)?);

    Ok(())
}

fn get_balance(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let balance = wallet.get_balance();
    println!("{}",format!("{:?}", balance));

    Ok(())
}

fn mine_blocks(wallet_name: &str, blocks: u64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = Wallet::new(wallet_name, settings);

    let address = miner_wallet.new_wallet_address(&bitcoincore_rpc::json::AddressType::Bech32)?;

    miner_wallet.mine_blocks(Some(blocks), &address)?;

    Ok(())
}

fn list_unspent(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let unspent_txs = wallet.list_all_unspent(None)?;
    let pretty_unspent_txs = serde_json::to_string_pretty(&unspent_txs)?;
    println!("{}", pretty_unspent_txs);

    Ok(())
}

fn get_tx(wallet_name: &str, txid: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    let txid_converted = bitcoin::Txid::from_str(txid)?;

    let tx = wallet.get_tx(&txid_converted)?;
    let pretty_tx = serde_json::to_string_pretty(&tx)?;
    println!("{}", pretty_tx);

    Ok(())
}

fn sign_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);
    let balance = wallet.get_balance()?;

    // check if balance is sufficient
    if balance < amount {
        panic!("Insufficient balance to send tx. Current balance: {}", balance);
    }

    println!("Creating raw transaction...");
    let unspent_txs = wallet.list_all_unspent(None)?; // TODO: add filter to only include txs with amount > 0
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
    outputs.insert(recipient.to_string(), amount);

    // Create raw transaction
    let tx: Transaction = wallet.create_raw_transaction(&utxos, &outputs, None, None)?;
    let raw_tx = serialize(&tx).raw_hex();
    println!("Raw transaction (hex): {:?}", raw_tx);

    println!("Signing raw transaction...");
    let signed_tx = wallet.sign_tx(&tx)?;
    let raw_tx = serialize(&signed_tx).raw_hex();
    println!("Signed raw transaction: {}", raw_tx);

    Ok(serialize(&signed_tx))
}

fn sign_tx_wrapper(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let signed_tx = sign_tx(wallet_name, recipient, amount, fee_amount, settings);
    if signed_tx.is_err(){
        return Err(signed_tx.unwrap_err());
    }

    Ok(())
}

fn sign_and_broadcast_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);
    
    let signed_tx = sign_tx(wallet_name, recipient, amount, fee_amount, settings)?;

    wallet.broadcast_tx(&signed_tx, None)?;

    Ok(())
}

fn send_btc(wallet_name: &str, recipient: &Address, amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new(wallet_name, settings);

    wallet.send(recipient, amount)?;

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
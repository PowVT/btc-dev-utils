use std::{fs, thread, time::Duration, path::Path};
use std::collections::HashMap;
use std::str::FromStr;

use bitcoin::consensus::serialize;
use bitcoin::{Address, Amount, Transaction};
use bitcoincore_rpc::json::{CreateRawTransactionInput, FinalizePsbtResult, GetAddressInfoResult, GetDescriptorInfoResult, GetRawTransactionResult, GetWalletInfoResult, ListUnspentResultEntry, WalletCreateFundedPsbtResult};
use bitcoincore_rpc::{Client, RawTx, RpcApi};
use log::{error, info};
use serde_json::{json, Value};
use clap::Parser;

use crate::settings::Settings;
use crate::utils::cli::{Cli, Action};
use crate::utils::utils::{Target, run_command};
use crate::utils::utils::extract_int_ext_xpubs;
use crate::modules::wallet::Wallet;
use crate::modules::client::create_rpc_client;

mod settings;
mod modules;
mod utils;

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
        Action::NewMultisig=> new_multisig_wallet(args.nrequired, &args.wallet_names, &args.multisig_name, &settings),
        Action::GetNewAddress => get_new_address(&args.wallet_name, &args.address_type, &settings),
        Action::GetAddressInfo => get_address_info(&args.address, &settings),
        Action::RescanBlockchain => rescan_blockchain(&settings),
        Action::GetBalances => get_balances(&args.wallet_name, &settings),
        Action::MineBlocks => mine_blocks(&args.wallet_name, args.blocks, &settings),
        Action::ListUnspent => list_unspent(&args.wallet_name, &settings),
        Action::GetTx => get_tx(&args.txid, &settings),
        Action::SignTx => sign_tx_wrapper(&args.wallet_name, &args.recipient,  args.amount, args.fee_amount, &settings),
        Action::BroadcastTx => broadcast_tx_wrapper( &args.tx_hex, args.max_fee_rate, &settings),
        Action::SignAndBroadcastTx => sign_and_broadcast_tx(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, args.max_fee_rate, &settings),
        Action::SendBtc => send_btc(&args.wallet_name, &args.recipient, args.amount, &settings),
        Action::CreatePsbt => create_psbt(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, &settings),
        Action::DecodePsbt => decode_psbt(&args.psbt_hex, &settings),
        Action::AnalyzePsbt => analyze_psbt(&args.psbt_hex, &settings),
        Action::WalletProcessPsbt => wallet_process_psbt(&args.wallet_name, &args.psbt_hex, &settings),
        Action::CombinePsbts => combine_psbts(&args.psbts, &settings),
        Action::FinalizePsbt => finalize_psbt(&args.psbt_hex, &settings),
        Action::FinalizePsbtAndBroadcast => finalize_psbt_and_broadcast(&args.psbt_hex, &settings),
        Action::InscribeOrd => regtest_inscribe_ord(&settings)
    }
}

fn new_wallet(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    Wallet::new(wallet_name, settings);
    
    Ok(())
}

fn get_wallet_info(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let wallet_info: GetWalletInfoResult = wallet.get_wallet_info()?;
    info!("{:#?}", wallet_info);

    Ok(())
}

fn list_descriptors(wallet_name: &str, settings: &Settings) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, Some(wallet_name));
    let descriptors: serde_json::Value = client.call("listdescriptors", &[])?;

    Ok(descriptors)
}

fn list_descriptors_wrapper(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let descriptors: Value = list_descriptors(wallet_name, settings)?;
    info!("{:#?}", descriptors);

    Ok(())
}

fn get_new_address(wallet_name: &str, address_type: &bitcoincore_rpc::json::AddressType, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let address: Address = wallet.new_wallet_address(address_type)?;
    info!("{}",format!("{:?}", address));

    Ok(())
}

fn get_address_info(address: &Address, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let address_info: GetAddressInfoResult = client.get_address_info(address)?;
    info!("{:#?}", address_info);

    Ok(())
}

fn rescan_blockchain(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);
    let _ = client.rescan_blockchain(Some(0), None);

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
    let client: Client = create_rpc_client(settings, None);

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

    // Create the multisig wallet
    let _ = client.create_wallet(multisig_name, Some(true), Some(true), None, None);

    // import the descriptors
    let multisig_desc_vec: Vec<serde_json::Value> = serde_json::from_value(multisig_desc)?;
    let client2 = create_rpc_client(settings, Some(multisig_name));
    client2.call::<serde_json::Value>("importdescriptors", &[json!(multisig_desc_vec)])?;

    // Get wallet info
    get_wallet_info(multisig_name, settings)?;

    Ok(())
}

fn get_balances(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let balances = wallet.get_balances()?;
    info!("{:#?}", balances);

    Ok(())
}

fn mine_blocks(wallet_name: &str, blocks: u64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = Wallet::new(wallet_name, settings);

    let address = miner_wallet.new_wallet_address(&bitcoincore_rpc::json::AddressType::Bech32)?;

    miner_wallet.mine_blocks(Some(blocks), &address)?;

    Ok(())
}

fn list_unspent(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    info!("{:#?}", unspent_txs);

    Ok(())
}

fn get_tx(txid: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let txid_converted = bitcoin::Txid::from_str(txid)?;
    let tx: GetRawTransactionResult = client.get_raw_transaction_info(&txid_converted, None)?;
    info!("{:#?}", tx);

    Ok(())
}

fn sign_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);
    let balances = wallet.get_balances()?;

    // check if balance is sufficient
    if balances.mine.trusted.to_sat() < amount.to_sat() {
        panic!("Insufficient balance to send tx. Current balance: {}", balances.mine.trusted);
    }

    info!("Creating raw transaction...");
    // TODO: add filter to only include txs with amount > 0
    let unspent_txs = wallet.list_all_unspent(None)?;
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
    info!("Raw transaction (hex): {:?}", raw_tx);

    info!("Signing raw transaction...");
    let signed_tx: Transaction = wallet.sign_tx(&tx)?;
    let raw_tx: String = serialize(&signed_tx).raw_hex();
    info!("Signed raw transaction: {}", raw_tx);

    Ok(serialize(&signed_tx))
}

fn sign_tx_wrapper(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let _ = sign_tx(wallet_name, recipient, amount, fee_amount, settings)?;

    Ok(())
}

fn sign_and_broadcast_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, max_fee_rate: f64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let signed_tx: Vec<u8> = sign_tx(wallet_name, recipient, amount, fee_amount, settings)?;

    let client: Client = create_rpc_client(settings, None);

    let _ = broadcast_tx(&client, &signed_tx.raw_hex(), Some(max_fee_rate))?;

    Ok(())
}

fn broadcast_tx(client: &Client, tx_hex: &str, max_fee_rate: Option<f64>) -> Result<String, Box<dyn std::error::Error>> {
    let max_fee_rate = match max_fee_rate {
        Some(fee_rate) => {
            let fee_rate = fee_rate as f64 / 100_000_000.0 * 1000.0;
            format!("{:.8}", fee_rate).parse::<f64>().unwrap()
        }
        None => 0.1, // the default fee rate is 0.1 BTC/kB
    };
    
    let tx_id: Value = client.call(
        "sendrawtransaction",
        &[json!(tx_hex), json!(max_fee_rate)],
    )?;
    let tx_id_str = tx_id.as_str().ok_or("Invalid tx_id response")?.to_string();
    info!("Broadcasted Tx ID: {}", tx_id_str);

    Ok(tx_id_str)
}

fn broadcast_tx_wrapper(tx_hex: &str, max_fee_rate: f64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let _ = broadcast_tx(&client, tx_hex, Some(max_fee_rate))?;

    Ok(())
}

fn send_btc(wallet_name: &str, recipient: &Address, amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    wallet.send(recipient, amount)?;

    Ok(())
}

fn create_psbt(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    // ensure the wallet is a multisig wallet
    if wallet.get_wallet_info()?.private_keys_enabled {
        panic!("Wallet is not a multisig wallet");
    }

    let bal = wallet.get_balances()?;
    if bal.mine.trusted.to_sat() < amount.to_sat() {
        panic!("Insufficient balance to send tx. Current balance: {}", bal.mine.trusted);
    }

    info!("Creating raw transaction...");
    // TODO: add filter to only include txs with amount > 0
    let unspent_txs = wallet.list_all_unspent(None)?;
    // get the first unspent transaction
    let unspent_txid = unspent_txs[0].txid;
    let unspent_vout = unspent_txs[0].vout;
    let unspent_amount = unspent_txs[0].amount;
    if unspent_txs.is_empty() {
        panic!("No unspent transactions");
    }
    if unspent_amount < amount + fee_amount {
        panic!("Insufficient unspent amount. Current amount: {}", unspent_amount);
    }
    
    let mut tx_inputs: Vec<bitcoincore_rpc::json::CreateRawTransactionInput> = Vec::new();
    tx_inputs.push(bitcoincore_rpc::json::CreateRawTransactionInput {
        txid: unspent_txid,
        vout: unspent_vout,
        sequence: None,
    });
    let mut tx_outputs: HashMap<String, Amount> = HashMap::new();
    tx_outputs.insert(recipient.to_string(), amount);
    let locktime = None;
    let options =  None; // TODO: can optionally specify the fee rate here, otherwise it will have the wallet estimate it
    let bip32derivs = None;
    let client = create_rpc_client(settings, Some(wallet_name));
    let psbt: WalletCreateFundedPsbtResult = client.wallet_create_funded_psbt(&tx_inputs[..], &tx_outputs, locktime, options, bip32derivs)?;

    info!("PSBT: {:#?}", psbt);

    Ok(())
}

fn decode_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let psbt: serde_json::Value = client.call("decodepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);

    Ok(())
}

fn analyze_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let psbt: serde_json::Value = client.call("analyzepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);

    Ok(())
}

fn wallet_process_psbt(wallet_name: &str, psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);
    let signed_psbt = wallet.process_psbt(&psbt)?;
    info!("Signed PSBT: {:#?}", signed_psbt);

    Ok(())
}

fn combine_psbts(psbts: &Vec<String>, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let res = client.combine_psbt(&psbts[..])?;
    info!("CombinedPSBT: {:#?}", res);

    Ok(())
}

fn finalize_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let res = client.finalize_psbt(psbt, None)?;
    info!("FinalizedPSBT: {:#?}", res);

    Ok(())
}

fn finalize_psbt_and_broadcast(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let res: FinalizePsbtResult = client.finalize_psbt(psbt, None)?;
    if !res.complete {
        return Err("PSBT does not have all necessary signatures".into());
    }
    let raw_hex: String = match res.hex {
        Some(hex) => hex.raw_hex(),
        None => return Err("No hex found in FinalizePsbtResult".into()),
    };

    info!("FinalizedPSBT: {:#?}", raw_hex);

    let tx_id: String = broadcast_tx(&client, &raw_hex, Some(0.0))?; // passing 0 as max fee rate will have the wallet estimate the fee rate
    info!("Tx broadcasted: {}", tx_id);

    Ok(())
}

fn regtest_inscribe_ord(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    const MIN_BALANCE: f64 = 50.0;
    const FEE_RATE: i32 = 15;

    // Check if wallet already exists
    if !Path::new("data/bitcoin/regtest/wallets/ord").exists() {
        fs::create_dir_all("data/bitcoin/regtest/wallets/ord")?;

        info!("Creating wallet...");
        run_command("wallet create", Target::Ord, settings);
    } else {
        info!("Wallet already exists, using existing wallet.");
    }

    info!("Generating mining address...");
    let json_str = run_command("wallet receive", Target::Ord, settings);
    let value: Value = serde_json::from_str(&json_str)?;
    let mining_address: String = value["addresses"][0].as_str().ok_or("No address found")?.to_string();

    // Mine blocks only if balance is insufficient
    let balance_output = run_command("-rpcwallet=ord getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_output.trim().parse()?;
    if balance < MIN_BALANCE {
        info!("Mining blocks...");
        run_command(&format!("generatetoaddress 101 {}", mining_address), Target::Bitcoin, settings);
        thread::sleep(Duration::from_secs(2));
    }

    let balance_output = run_command("-rpcwallet=ord getbalance", Target::Bitcoin, settings);
    let balance: f64 = balance_output.trim().parse()?;
    info!("Wallet balance: {} BTC", balance);

    if balance < MIN_BALANCE {
        panic!("Failed to mine sufficient balance");
    }

    // Create inscription
    info!("Creating inscription...");
    run_command(&format!("wallet inscribe --fee-rate {}  --file ./mockOrdContent.txt", FEE_RATE), Target::Ord, settings);

    run_command(&format!("generatetoaddress 10 {}", mining_address), Target::Bitcoin, settings);
    thread::sleep(Duration::from_secs(10));

    let inscriptions = run_command("wallet inscriptions", Target::Ord, settings);
    info!("Inscription Data: {:?}", inscriptions);

    let balance_output = run_command("-rpcwallet=ord listaddressgroupings", Target::Bitcoin, settings);
    let balance_str = balance_output.trim();
    let balance: serde_json::Value = serde_json::from_str(balance_str)?;
    info!("Wallet bitcoin balances: {:?}", balance);

    let ord_balances = run_command("wallet balance", Target::Ord, settings);
    info!("Wallet ordinal balance: {:?}", ord_balances);

    Ok(())
}
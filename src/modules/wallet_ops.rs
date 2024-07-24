use std::collections::HashMap;

use bitcoin::{Address, Amount, Transaction};
use bitcoincore_rpc::json::{CreateRawTransactionInput, GetAddressInfoResult, GetDescriptorInfoResult, GetWalletInfoResult, ListUnspentResultEntry, WalletCreateFundedPsbtResult};
use bitcoincore_rpc::{Client, RawTx, RpcApi};
use bitcoin::consensus::serialize;

use log::info;
use serde_json::{json, Value};

use crate::settings::Settings;
use crate::modules::wallet::Wallet;
use crate::modules::bitcoind_conn::create_rpc_client;
use crate::modules::bitcoind_client::mine_blocks;
use crate::utils::utils::{extract_int_ext_xpubs, strat_handler, UTXOStrategy};

/// General wallet operations

pub fn new_wallet(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    Wallet::new(wallet_name, settings);
    
    Ok(())
}

pub fn get_wallet_info(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let wallet_info: GetWalletInfoResult = wallet.get_wallet_info()?;
    info!("{:#?}", wallet_info);

    Ok(())
}

pub fn list_descriptors(wallet_name: &str, settings: &Settings) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, Some(wallet_name));
    let descriptors: serde_json::Value = client.call("listdescriptors", &[])?;

    Ok(descriptors)
}

pub fn list_descriptors_wrapper(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let descriptors: Value = list_descriptors(wallet_name, settings)?;
    info!("{:#?}", descriptors);

    Ok(())
}

pub fn get_new_address(wallet_name: &str, address_type: &bitcoincore_rpc::json::AddressType, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let address: Address = wallet.new_address(address_type)?;
    info!("{}",format!("{:?}", address));

    Ok(())
}

pub fn get_address_info(address: &Address, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let address_info: GetAddressInfoResult = client.get_address_info(address)?;
    info!("{:#?}", address_info);

    Ok(())
}

pub fn new_multisig_wallet(nrequired: u32, wallet_names: &Vec<String>, multisig_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn get_balances(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let balances = wallet.get_balances()?;
    info!("{:#?}", balances);

    Ok(())
}

pub fn list_unspent(wallet_name: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    info!("{:#?}", unspent_txs);

    Ok(())
}

/// Mine blocks

pub fn mine_blocks_wrapper(wallet_name: &str, blocks: u64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = Wallet::new(wallet_name, settings);
    let address = miner_wallet.new_address(&bitcoincore_rpc::json::AddressType::Bech32)?;

    mine_blocks(Some(blocks), &address, settings)?;

    Ok(())
}

/// Sign a transaction

pub fn sign_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);
    let balances = wallet.get_balances()?;

    // check if balance is sufficient
    if balances.mine.trusted.to_sat() < amount.to_sat() {
        panic!("Insufficient balance to send tx. Current balance: {}", balances.mine.trusted);
    }

    // List all unspent transactions
    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    if unspent_txs.is_empty() {
        panic!("No unspent transactions");
    }

    // Based on the strategy, select UTXOs
    let selected_utxos = strat_handler(&unspent_txs, amount, fee_amount, utxo_strat)?;

    let mut utxo_inputs: Vec<CreateRawTransactionInput> = Vec::new();
    let mut total_amount = Amount::from_sat(0);
    for utxo in &selected_utxos {
        utxo_inputs.push(CreateRawTransactionInput {
            txid: utxo.txid,
            vout: utxo.vout,
            sequence: Some(0),
        });
        total_amount += utxo.amount;
    }

    let mut outputs: HashMap<String, Amount> = HashMap::new();
    outputs.insert(recipient.to_string(), amount);

    // Add change output if there's any remaining amount
    let change_amount = total_amount - amount - fee_amount;
    if change_amount.to_sat() > 0 {
        let change_address: Address = wallet.new_address(&bitcoincore_rpc::json::AddressType::Bech32)?;
        outputs.insert(change_address.to_string(), change_amount);
    }

    // Create raw transaction
    let client: Client = create_rpc_client(settings, Some(wallet_name));
    let tx: Transaction = client.create_raw_transaction(&utxo_inputs[..], &outputs, None, None)?;

    let signed_tx: Transaction = wallet.sign_tx(&tx)?;
    let raw_tx: String = serialize(&signed_tx).raw_hex();
    info!("Signed raw transaction: {}", raw_tx);

    Ok(serialize(&signed_tx))
}

pub fn sign_tx_wrapper(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let _ = sign_tx(wallet_name, recipient, amount, fee_amount, utxo_strat, settings)?;

    Ok(())
}

/// Send BTC

pub fn send_btc(wallet_name: &str, recipient: &Address, amount: Amount, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    wallet.send(recipient, amount)?;

    Ok(())
}

/// PSBT operations

pub fn create_psbt(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);

    // ensure the wallet is a multisig wallet
    if wallet.get_wallet_info()?.private_keys_enabled {
        panic!("Wallet is not a multisig wallet");
    }

    let bal = wallet.get_balances()?;
    if bal.mine.trusted.to_sat() < amount.to_sat() {
        panic!("Insufficient balance to send tx. Current balance: {}", bal.mine.trusted);
    }

    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    if unspent_txs.is_empty() {
        panic!("No unspent transactions");
    }

    // Based on the strategy, select UTXOs
    let selected_utxos = strat_handler(&unspent_txs, amount, fee_amount, utxo_strat)?;

    let mut tx_inputs: Vec<CreateRawTransactionInput> = Vec::new();
    let mut total_amount = Amount::from_sat(0);
    for utxo in &selected_utxos {
        tx_inputs.push(CreateRawTransactionInput {
            txid: utxo.txid,
            vout: utxo.vout,
            sequence: None,
        });
        total_amount += utxo.amount;
    }

    let mut tx_outputs: HashMap<String, Amount> = HashMap::new();
    tx_outputs.insert(recipient.to_string(), amount);

    // Add change output if there's any remaining amount
    let change_amount = total_amount - amount - fee_amount;
    if change_amount.to_sat() > 0 {
        let change_address = wallet.new_address(&bitcoincore_rpc::json::AddressType::Bech32)?;
        tx_outputs.insert(change_address.to_string(), change_amount);
    }

    let locktime = None;
    let options =  None; // TODO: can optionally specify the fee rate here, otherwise it will have the wallet estimate it
    let bip32derivs = None;
    let client = create_rpc_client(settings, Some(wallet_name));
    let psbt: WalletCreateFundedPsbtResult = client.wallet_create_funded_psbt(&tx_inputs[..], &tx_outputs, locktime, options, bip32derivs)?;

    info!("PSBT: {:#?}", psbt);

    Ok(())
}

pub fn process_psbt(wallet_name: &str, psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let wallet: Wallet = Wallet::new(wallet_name, settings);
    let signed_psbt = wallet.process_psbt(&psbt)?;
    info!("Signed PSBT: {:#?}", signed_psbt);

    Ok(())
}



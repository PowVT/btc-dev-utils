use std::collections::HashMap;

use log::info;

use serde_json::{json, Value};

use bitcoin::{Address, Amount, Transaction, consensus::serialize};
use bitcoincore_rpc::json::{AddressType, CreateRawTransactionInput, GetAddressInfoResult, GetDescriptorInfoResult, GetWalletInfoResult, ListUnspentResultEntry, WalletCreateFundedPsbtResult};
use bitcoincore_rpc::{Client, RawTx, RpcApi};

use miniscript::bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};

use crate::settings::Settings;
use crate::modules::wallet::Wallet;
use crate::modules::bitcoind::create_rpc_client;
use crate::modules::client::mine_blocks;
use crate::utils::utils::{extract_int_ext_xpubs, strat_handler, UTXOStrategy};

use super::errors::WalletOpsError;

pub fn new_wallet(wallet_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    Wallet::new(wallet_name, settings)?;
    Ok(())
}

pub fn get_wallet_info(wallet_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let wallet_info: GetWalletInfoResult = wallet.get_wallet_info()?;
    info!("{:#?}", wallet_info);
    Ok(())
}

pub fn list_descriptors(wallet_name: &str, settings: &Settings) -> Result<serde_json::Value, WalletOpsError> {
    let client = create_rpc_client(settings, Some(wallet_name))?;
    let descriptors: serde_json::Value = client.call("listdescriptors", &[])?;
    Ok(descriptors)
}

pub fn list_descriptors_wrapper(wallet_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    let descriptors: Value = list_descriptors(wallet_name, settings)?;
    info!("{:#?}", descriptors);
    Ok(())
}

pub fn get_new_address(wallet_name: &str, address_type: &AddressType, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let address: Address = wallet.new_address(address_type)?;
    info!("{}", format!("{:?}", address));
    Ok(())
}

pub fn get_address_info(wallet_name: &str, address: &Address, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let address_info: GetAddressInfoResult = wallet.get_address_info(address)?;
    info!("{:#?}", address_info);
    Ok(())
}

pub fn derive_addresses(descriptor: &str, start: &u32, end: &u32, settings: &Settings) -> Result<(), WalletOpsError> {
    let client = create_rpc_client(settings, None)?;
    let range: [u32; 2] = [*start, *end];

    let desc = if !descriptor.contains('#') {
        let secp = Secp256k1::new();
        let (desc, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, descriptor)?;
        desc.to_string()
    } else {
        descriptor.to_string()
    };

    let addresses = client.derive_addresses(&desc, Some(range))?;
    info!("Derived addresses:");
    for (i, address) in addresses.iter().enumerate() {
        info!("  {}: {:#?}", i + *start as usize, address);
    }
    Ok(())
}

pub fn new_multisig_wallet(nrequired: u32, wallet_names: &Vec<String>, multisig_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    if wallet_names.len() < nrequired as usize {
        return Err(WalletOpsError::Other("More required signers than wallets".into()));
    }

    let mut xpubs: HashMap<String, String> = HashMap::new();

    for wallet_name in wallet_names {
        new_wallet(wallet_name, settings)?;
    }

    for (i, wallet_name) in wallet_names.iter().enumerate() {
        let descriptors: serde_json::Value = list_descriptors(wallet_name, settings)?;
        let descriptors_array: &Vec<serde_json::Value> = descriptors["descriptors"].as_array()
            .ok_or_else(|| WalletOpsError::Other("Invalid descriptor format".into()))?;
        xpubs = extract_int_ext_xpubs(xpubs, descriptors_array.clone(), i)?;
    }

    let num_signers = nrequired.to_string();
    let external_desc = format!(
        "wsh(sortedmulti({}, {}, {}, {}))",
        num_signers, xpubs["external_xpub_1"], xpubs["external_xpub_2"], xpubs["external_xpub_3"]
    );
    let internal_desc = format!(
        "wsh(sortedmulti({}, {}, {}, {}))",
        num_signers, xpubs["internal_xpub_1"], xpubs["internal_xpub_2"], xpubs["internal_xpub_3"]
    );

    let client: Client = create_rpc_client(settings, None)?;

    let external_desc_info: GetDescriptorInfoResult = client.get_descriptor_info(&external_desc)?;
    let internal_desc_info: GetDescriptorInfoResult = client.get_descriptor_info(&internal_desc)?;

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

    let multisig_desc = json!([multisig_ext_desc, multisig_int_desc]);

    client.create_wallet(multisig_name, Some(true), Some(true), None, None)?;

    let multisig_desc_vec: Vec<serde_json::Value> = serde_json::from_value(multisig_desc)?;
    let client2 = create_rpc_client(settings, Some(multisig_name))?;
    client2.call::<serde_json::Value>("importdescriptors", &[json!(multisig_desc_vec)])?;

    get_wallet_info(multisig_name, settings)?;

    Ok(())
}

pub fn get_balances(wallet_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let balances = wallet.get_balances()?;
    info!("{:#?}", balances);
    Ok(())
}

pub fn list_unspent(wallet_name: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    info!("{:#?}", unspent_txs);
    Ok(())
}

pub fn mine_blocks_wrapper(wallet_name: &str, blocks: u64, settings: &Settings) -> Result<(), WalletOpsError> {
    let miner_wallet = Wallet::new(wallet_name, settings)?;
    let address = miner_wallet.new_address(&AddressType::Bech32)?;
    mine_blocks(Some(blocks), &address, settings)?;
    Ok(())
}

pub fn sign_tx(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<Vec<u8>, WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let balances = wallet.get_balances()?;

    if balances.mine.trusted.to_sat() < amount.to_sat() {
        return Err(WalletOpsError::InsufficientBalance);
    }

    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    if unspent_txs.is_empty() {
        return Err(WalletOpsError::NoUnspentTransactions);
    }

    let selected_utxos = strat_handler(&unspent_txs, amount, fee_amount, utxo_strat)
        .map_err(|e| WalletOpsError::Other(e.to_string()))?;

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

    let change_amount = total_amount - amount - fee_amount;
    if change_amount.to_sat() > 0 {
        let change_address: Address = wallet.new_address(&AddressType::Bech32)?;
        outputs.insert(change_address.to_string(), change_amount);
    }

    let client: Client = create_rpc_client(settings, Some(wallet_name))?;
    let tx: Transaction = client.create_raw_transaction(&utxo_inputs[..], &outputs, None, None)?;

    let signed_tx: Transaction = wallet.sign_tx(&tx)?;
    let raw_tx: String = serialize(&signed_tx).raw_hex();
    info!("Signed raw transaction: {}", raw_tx);

    Ok(serialize(&signed_tx))
}

pub fn sign_tx_wrapper(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<(), WalletOpsError> {
    sign_tx(wallet_name, recipient, amount, fee_amount, utxo_strat, settings)?;
    Ok(())
}

pub fn send_btc(wallet_name: &str, recipient: &Address, amount: Amount, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    wallet.send(recipient, amount)?;
    Ok(())
}

pub fn create_psbt(wallet_name: &str, recipient: &Address, amount: Amount, fee_amount: Amount, utxo_strat: UTXOStrategy, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;

    // Ensure the wallet is a multisig wallet
    if wallet.get_wallet_info()?.private_keys_enabled {
        return Err(WalletOpsError::NotMultisigWallet);
    }

    let bal = wallet.get_balances()?;
    if bal.mine.trusted.to_sat() < amount.to_sat() {
        return Err(WalletOpsError::InsufficientBalance);
    }

    let unspent_txs: Vec<ListUnspentResultEntry> = wallet.list_all_unspent(None)?;
    if unspent_txs.is_empty() {
        return Err(WalletOpsError::NoUnspentTransactions);
    }

    // Based on the strategy, select UTXOs
    let selected_utxos = strat_handler(&unspent_txs, amount, fee_amount, utxo_strat)
        .map_err(|e| WalletOpsError::Other(e.to_string()))?;

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
        let change_address = wallet.new_address(&AddressType::Bech32)?;
        tx_outputs.insert(change_address.to_string(), change_amount);
    }

    let locktime = None;
    // TODO: can optionally specify the fee rate here, otherwise it will have the wallet estimate it
    let options = None;
    let bip32derivs = None;
    let client = create_rpc_client(settings, Some(wallet_name))?;
    let psbt: WalletCreateFundedPsbtResult = client
        .wallet_create_funded_psbt(&tx_inputs[..], &tx_outputs, locktime, options, bip32derivs)?;

    info!("PSBT: {:#?}", psbt);

    Ok(())
}

pub fn process_psbt(wallet_name: &str, psbt: &str, settings: &Settings) -> Result<(), WalletOpsError> {
    let wallet: Wallet = Wallet::new(wallet_name, settings)?;
    let signed_psbt = wallet.process_psbt(psbt)?;
    info!("Signed PSBT: {:#?}", signed_psbt);

    Ok(())
}

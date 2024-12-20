use std::str::FromStr;

use bitcoin::{Address, Amount};
use bitcoincore_rpc::json::{GetRawTransactionResult, GetTxOutResult, ScanTxOutRequest};
use bitcoincore_rpc::{json::FinalizePsbtResult, RpcApi, Client};

use log::info;
use serde_json::{json, Value};

use crate::settings::Settings;
use crate::modules::bitcoind::create_rpc_client;

use super::errors::BitcoindError;

/// Blockchain Ops

pub fn get_block_height(settings: &Settings) -> Result<(), BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    let height = client.get_block_count()?;
    info!("Block height: {}", height);
    Ok(())
}

pub fn mine_blocks(blocks: Option<u64>, address: &Address, settings: &Settings) -> Result<(), BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    let blocks_to_mine = blocks.unwrap_or(1);
    info!("Mining {} blocks", blocks_to_mine);
    client.generate_to_address(blocks_to_mine, address)?;
    info!("Mined {} blocks to address {}", blocks_to_mine, address);
    Ok(())
}

pub fn rescan_blockchain(settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    client.rescan_blockchain(Some(0), None)?;
    Ok(())
}

/// Transaction Ops

pub fn get_tx(txid: &str, settings: &Settings) -> Result<GetRawTransactionResult, BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    let txid_converted = bitcoin::Txid::from_str(txid).map_err(|_| BitcoindError::InvalidTxId)?;
    let tx = client.get_raw_transaction_info(&txid_converted, None)?;
    Ok(tx)
}

pub fn get_tx_wrapper(txid: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let tx = get_tx(txid, settings)?;
    info!("{:#?}", tx);
    Ok(())
}

pub fn get_tx_out(txid: &str, vout: u32, confirmations: Option<u32>, settings: &Settings) -> Result<GetTxOutResult, BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    let txid_converted = bitcoin::Txid::from_str(txid).map_err(|_| BitcoindError::InvalidTxId)?;
    let tx_out = client.get_tx_out(&txid_converted, vout, None)?
        .ok_or(BitcoindError::TxOutNotFound)?;

    if let Some(required_confirmations) = confirmations {
        if tx_out.confirmations < required_confirmations {
            return Err(BitcoindError::InsufficientConfirmations);
        }
    }

    Ok(tx_out)
}

pub fn get_tx_out_wrapper(txid: &str, vout: u32, confirmations: Option<u32>, settings: &Settings) -> Result<(), BitcoindError> {
    let tx_out = get_tx_out(txid, vout, confirmations, settings)?;
    info!("{:#?}", tx_out);
    Ok(())
}

pub fn broadcast_tx(client: &Client, tx_hex: &str, max_fee_rate: Option<f64>) -> Result<String, BitcoindError> {
    let max_fee_rate = max_fee_rate.map(|fee_rate| {
        (fee_rate / 100_000_000.0 * 1000.0).to_string().parse::<f64>().unwrap_or(0.1)
    }).unwrap_or(0.1);

    let tx_id: Value = client.call(
        "sendrawtransaction",
        &[json!(tx_hex), json!(max_fee_rate)],
    )?;
    let tx_id_str = tx_id.as_str()
        .ok_or_else(|| BitcoindError::Other("Invalid tx_id response".to_string()))?
        .to_string();
    info!("Broadcasted Tx ID: {}", tx_id_str);
    Ok(tx_id_str)
}

pub fn broadcast_tx_wrapper(tx_hex: &str, max_fee_rate: f64, settings: &Settings) -> Result<(), BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    broadcast_tx(&client, tx_hex, Some(max_fee_rate))?;
    Ok(())
}

pub fn decode_raw_tx(tx_hex: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    let tx = client.decode_raw_transaction(tx_hex, None)?;
    info!("{:#?}", tx);
    Ok(())
}

/// PSBT Ops

pub fn decode_psbt(psbt: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    let psbt: serde_json::Value = client.call("decodepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);
    Ok(())
}

pub fn analyze_psbt(psbt: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    let psbt: serde_json::Value = client.call("analyzepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);
    Ok(())
}

pub fn combine_psbts(psbts: &Vec<String>, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    let res = client.combine_psbt(&psbts[..])?;
    info!("CombinedPSBT: {:#?}", res);
    Ok(())
}

pub fn finalize_psbt(psbt: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;
    let res = client.finalize_psbt(psbt, None)?;
    info!("FinalizedPSBT: {:#?}", res);
    Ok(())
}

pub fn finalize_psbt_and_broadcast(psbt: &str, settings: &Settings) -> Result<(), BitcoindError> {
    let client: Client = create_rpc_client(settings, None)?;
    let res: FinalizePsbtResult = client.finalize_psbt(psbt, None)?;
    if !res.complete {
        return Err(BitcoindError::IncompletePsbt);
    }
    let raw_hex: String = res.hex.ok_or(BitcoindError::NoHexInFinalizedPsbt)?
        .iter().map(|b| format!("{:02x}", b)).collect();

    info!("FinalizedPSBT: {}", raw_hex);

    let tx_id: String = broadcast_tx(&client, &raw_hex, Some(0.0))?;
    info!("Tx broadcasted: {}", tx_id);
    Ok(())
}

/// Address Ops

/// NOTE: this function does not check if the UTXO is from coinbase rewards or not, it only
/// checks if the UTXO has greater than or equal to 6 confirmations.
pub fn get_spendable_balance(address: &Address, settings: &Settings) -> Result<(), BitcoindError> {
    let client = create_rpc_client(settings, None)?;

    let descriptor = format!("addr({})", address);
    let scan_request = ScanTxOutRequest::Single(descriptor);
    let result = client.scan_tx_out_set_blocking(&[scan_request])?;

    let current_height = client.get_block_count()?;

    let mut total_spendable = Amount::from_sat(0);
    let mut spendable_utxos = Vec::new();

    info!("Address: {}", address);
    info!("UTXOs:");

    for (index, utxo) in result.unspents.iter().enumerate() {
        let confirmations = current_height - utxo.height + 1;

        if confirmations >= 6 {
            total_spendable += utxo.amount;
            spendable_utxos.push(utxo);

            info!("  UTXO {}: {} BTC (txid: {}, vout: {}, confirmations: {})", 
                  index + 1, 
                  utxo.amount.to_btc(), 
                  utxo.txid, 
                  utxo.vout,
                  confirmations);
        }
    }

    info!("Total Spendable Balance: {} BTC", total_spendable.to_btc());
    info!("Number of Spendable UTXOs: {}", spendable_utxos.len());

    Ok(())
}

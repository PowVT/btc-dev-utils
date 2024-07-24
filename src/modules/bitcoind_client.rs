use std::str::FromStr;

use bitcoin::Address;
use bitcoincore_rpc::{json::FinalizePsbtResult, RpcApi, RawTx, Client};

use log::info;
use serde_json::{json, Value};

use crate::settings::Settings;
use crate::modules::bitcoind_conn::create_rpc_client;

/// Blockchain Ops

pub fn mine_blocks(blocks: Option<u64>, address: &Address, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    info!("Mining {} blocks", blocks.unwrap_or(1));
    client.generate_to_address(blocks.unwrap_or(1), address)?;
    info!("Mined {} blocks to address {}", blocks.unwrap_or(1), address);

    Ok(())
}

pub fn rescan_blockchain(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);
    let _ = client.rescan_blockchain(Some(0), None);

    Ok(())
}

/// Transaction Ops

pub fn get_tx(txid: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let txid_converted = bitcoin::Txid::from_str(txid)?;
    let tx = client.get_raw_transaction_info(&txid_converted, None)?;
    info!("{:#?}", tx);

    Ok(())
}

pub fn broadcast_tx(client: &Client, tx_hex: &str, max_fee_rate: Option<f64>) -> Result<String, Box<dyn std::error::Error>> {
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

pub fn broadcast_tx_wrapper(tx_hex: &str, max_fee_rate: f64, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let _ = broadcast_tx(&client, tx_hex, Some(max_fee_rate))?;

    Ok(())
}

/// PSBT Ops

pub fn decode_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let psbt: serde_json::Value = client.call("decodepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);

    Ok(())
}

pub fn analyze_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let psbt: serde_json::Value = client.call("analyzepsbt", &[json!(psbt)])?;
    info!("PSBT: {:#?}", psbt);

    Ok(())
}

pub fn combine_psbts(psbts: &Vec<String>, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let res = client.combine_psbt(&psbts[..])?;
    info!("CombinedPSBT: {:#?}", res);

    Ok(())
}

pub fn finalize_psbt(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let res = client.finalize_psbt(psbt, None)?;
    info!("FinalizedPSBT: {:#?}", res);

    Ok(())
}

pub fn finalize_psbt_and_broadcast(psbt: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
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

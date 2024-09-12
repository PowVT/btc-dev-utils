use std::str::FromStr;

use bitcoin::{Address, Amount};
use bitcoincore_rpc::json::{GetRawTransactionResult, GetTxOutResult, ScanTxOutRequest};
use bitcoincore_rpc::{json::FinalizePsbtResult, RpcApi, RawTx, Client};

use log::info;
use serde_json::{json, Value};

use crate::settings::Settings;
use crate::modules::bitcoind_conn::create_rpc_client;

/// Blockchain Ops

pub fn get_block_height(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let height = client.get_block_count()?;

    info!("Block height: {}", height);

    Ok(())
}

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
 
pub fn get_tx(txid: &str, settings: &Settings) -> Result<GetRawTransactionResult, Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None); 

    let txid_converted = bitcoin::Txid::from_str(txid)?;
    let tx = client.get_raw_transaction_info(&txid_converted, None)?;

    Ok(tx)
}

pub fn get_tx_wrapper(txid: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let tx = get_tx(txid, settings)?;

    info!("{:#?}", tx);

    Ok(())
}

pub fn get_tx_out(txid: &str, vout: u32, confirmations: Option<u32>, settings: &Settings) -> Result<GetTxOutResult, Box<dyn std::error::Error>> {
    let client: Client = create_rpc_client(settings, None);

    let txid_converted = bitcoin::Txid::from_str(txid)?;
    let tx_out = client.get_tx_out(&txid_converted, vout, None)?; // None = include_mempool

    match tx_out {
        Some(tx_out) => {
            if let Some(confirmations) = confirmations {
                if tx_out.confirmations >= confirmations {
                    Ok(tx_out)
                } else {
                    Err(format!("TxOut not enough confirmations").into())
                }
            } else {
                Ok(tx_out)
            }
        },
        None => {
            Err(format!("TxOut not found").into())
        },
    }
}

pub fn get_tx_out_wrapper(txid: &str, vout: u32, confirmations: Option<u32>, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let tx_out = get_tx_out(txid, vout, confirmations,settings)?;

    info!("{:#?}", tx_out);

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

pub fn decode_raw_tx(tx_hex: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

    let tx = client.decode_raw_transaction(tx_hex, None)?;
    info!("{:#?}", tx);

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

/// Address Ops

/// NOTE: this function does not check if the UTXO is from coinbase rewards or not, it only
/// checks if the UTXO has greater than or equal to 6 confirmations.
pub fn get_spendable_balance(address: &Address, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_rpc_client(settings, None);

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

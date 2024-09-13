use bitcoin::{consensus::deserialize, OutPoint, Transaction, TxOut};

use crate::{modules::bitcoind_client::get_tx, settings::Settings};

use super::bitcoind_client::get_tx_out;

pub fn verify_signed_tx(tx_hex: &str, settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let tx: Transaction = deserialize(&hex::decode(tx_hex)?)?;

    println!("Verifying transaction: {}", tx.txid());
    println!("Number of inputs: {}", tx.input.len());

    // Check if UTXOs are still unspent
    for (index, input) in tx.input.iter().enumerate() {
        println!("Checking UTXO for input {}", index);
        match is_utxo_unspent(&input.previous_output, settings) {
            Ok(true) => println!("UTXO for input {} is unspent", index), // UTXO is unspent, continue
            Ok(false) => return Err(format!("UTXO for input {} has already been spent", index).into()),
            Err(e) => return Err(format!("Error checking UTXO for input {}: {}", index, e).into()),
        }
    }

    // Closure to fetch previous transaction output (TxOut) for each input
    let mut spent = |outpoint: &OutPoint| -> Option<TxOut> {
        match get_tx(&outpoint.txid.to_string(), settings) {
            Ok(prev_tx) => prev_tx.vout.get(outpoint.vout as usize).map(|output| {
                TxOut {
                    value: output.value,
                    script_pubkey: bitcoin::ScriptBuf::from(output.script_pub_key.hex.clone()),
                }
            }),
            Err(_) => None,
        }
    };

    // Verify the transaction. For each input, check if unlocking script is valid based on the corresponding TxOut.
    tx.verify(&mut spent).map_err(|e| {format!("Transaction verification failed: {:?}", e)})?;

    println!("Transaction verified successfully");

    Ok(())
}

fn is_utxo_unspent(outpoint: &OutPoint, settings: &Settings) -> Result<bool, Box<dyn std::error::Error>> {
    let txid = outpoint.txid.to_string();

    match get_tx_out(&txid, outpoint.vout, None, settings) {
        Ok(_) => Ok(true),  // UTXO exists and is unspent
        Err(e) => {
            if e.to_string().contains("TxOut not found") {
                Ok(false)  // UTXO doesn't exist (already spent)
            } else {
                Err(format!("Error checking UTXO: {}", e).into())
            }
        }
    }
}
use log::info;

use bitcoin::{consensus::deserialize, OutPoint, Transaction, TxOut};

use crate::{
    modules::{
        client::get_tx,
        errors::VerificationError
    },
    settings::Settings
};

use super::client::get_tx_out;

pub fn verify_signed_tx(tx_hex: &str, settings: &Settings) -> Result<(), VerificationError> {
    let tx: Transaction = deserialize(&hex::decode(tx_hex)?)?;

    info!("Verifying transaction: {}", tx.txid());
    info!("Number of inputs: {}", tx.input.len());

    // Check if UTXOs are still unspent
    for (index, input) in tx.input.iter().enumerate() {
        info!("Checking UTXO for input {}", index);
        match is_utxo_unspent(&input.previous_output, settings) {
            Ok(true) => info!("UTXO for input {} is unspent", index), // UTXO is unspent, continue
            Ok(false) => return Err(VerificationError::UTXOAlreadySpent(index)),
            Err(e) => return Err(VerificationError::UTXOCheckError(index, e.to_string())),
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
            Err(_) => None
        }
    };

    // Verify the transaction. For each input, check if unlocking script is valid based on the corresponding TxOut.
    tx.verify(&mut spent).map_err(|e| VerificationError::TransactionVerificationFailed(e.to_string()))?;

    info!("Transaction verified successfully");

    Ok(())
}

fn is_utxo_unspent(outpoint: &OutPoint, settings: &Settings) -> Result<bool, VerificationError> {
    let txid = outpoint.txid.to_string();

    match get_tx_out(&txid, outpoint.vout, None, settings) {
        Ok(_) => Ok(true),  // UTXO exists and is unspent
        Err(e) => {
            if e.to_string().contains("TxOut not found") {
                Ok(false)  // UTXO doesn't exist (already spent)
            } else {
                Err(VerificationError::UTXOError(e.to_string()))
            }
        }
    }
}
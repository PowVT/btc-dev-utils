use std::error::Error;

use log::{error, info};

use clap::Parser;

use modules::client::{
    analyze_psbt,
    broadcast_tx_wrapper,
    combine_psbts, decode_psbt,
    decode_raw_tx,
    finalize_psbt,
    finalize_psbt_and_broadcast,
    get_block_height,
    get_spendable_balance,
    get_tx_out_wrapper,
    get_tx_wrapper,
    rescan_blockchain
};
use modules::wallet_ops::{
    create_psbt,
    get_address_info,
    derive_addresses,
    get_balances,
    get_new_address,
    get_wallet_info,
    list_descriptors_wrapper,
    list_unspent,
    mine_blocks_wrapper,
    new_multisig_wallet,
    new_wallet,
    process_psbt,
    send_btc,
    sign_tx_wrapper
};
use modules::verification::verify_signed_tx;

use settings::Settings;

use utils::cli::{Cli, Action};

mod modules;
mod settings;
mod utils;
fn main() -> Result<(), Box<dyn Error>> {
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
        Action::GetBlockHeight => get_block_height(&settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::NewWallet => new_wallet(&args.wallet_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetWalletInfo => get_wallet_info(&args.wallet_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::ListDescriptors => list_descriptors_wrapper(&args.wallet_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::NewMultisig => new_multisig_wallet(args.nrequired, &args.wallet_names, &args.multisig_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetNewAddress => get_new_address(&args.wallet_name, &args.address_type, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetAddressInfo => get_address_info(&args.wallet_name, &args.address, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::DeriveAddresses => derive_addresses(&args.descriptor, &args.start, &args.end, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::RescanBlockchain => rescan_blockchain(&settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetBalance => get_balances(&args.wallet_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetSpendableBalance => get_spendable_balance(&args.address, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::MineBlocks => mine_blocks_wrapper(&args.wallet_name, args.blocks, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::ListUnspent => list_unspent(&args.wallet_name, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetTx => get_tx_wrapper(&args.txid, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::GetTxOut => get_tx_out_wrapper(&args.txid, args.vout, Some(args.confirmations), &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::SignTx => sign_tx_wrapper(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, args.utxo_strat, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::DecodeRawTx => decode_raw_tx(&args.tx_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::BroadcastTx => broadcast_tx_wrapper(&args.tx_hex, args.max_fee_rate, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::SendBtc => send_btc(&args.wallet_name, &args.recipient, args.amount, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::CreatePsbt => create_psbt(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, args.utxo_strat, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::DecodePsbt => decode_psbt(&args.psbt_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::AnalyzePsbt => analyze_psbt(&args.psbt_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::WalletProcessPsbt => process_psbt(&args.wallet_name, &args.psbt_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::CombinePsbts => combine_psbts(&args.psbts, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::FinalizePsbt => finalize_psbt(&args.psbt_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::FinalizePsbtAndBroadcast => finalize_psbt_and_broadcast(&args.psbt_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
        Action::VerifySignedTx => verify_signed_tx(&args.tx_hex, &settings).map_err(|e| Box::new(e) as Box<dyn Error>)?,
    };

    Ok(())
}

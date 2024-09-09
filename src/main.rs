use std::{fs, thread, time::Duration, path::Path};

use log::{error, info};
use serde_json::Value;
use clap::Parser;

use modules::bitcoind_client::{
    analyze_psbt,
    broadcast_tx_wrapper,
    combine_psbts,
    decode_psbt,
    finalize_psbt,
    finalize_psbt_and_broadcast,
    get_block_height,
    get_spendable_balance,
    get_tx, rescan_blockchain
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

use settings::Settings;
use utils::cli::{Cli, Action};
use utils::utils::{Target, run_command};

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
        Action::GetBlockHeight => get_block_height(&settings),
        Action::NewWallet => new_wallet(&args.wallet_name, &settings),
        Action::GetWalletInfo => get_wallet_info(&args.wallet_name, &settings),
        Action::ListDescriptors => list_descriptors_wrapper(&args.wallet_name, &settings),
        Action::NewMultisig=> new_multisig_wallet(args.nrequired, &args.wallet_names, &args.multisig_name, &settings),
        Action::GetNewAddress => get_new_address(&args.wallet_name, &args.address_type, &settings),
        Action::GetAddressInfo => get_address_info(&args.wallet_name, &args.address, &settings),
        Action::DeriveAddresses => derive_addresses(&args.descriptor, &args.start, &args.end, &settings),
        Action::RescanBlockchain => rescan_blockchain(&settings),
        Action::GetBalance => get_balances(&args.wallet_name, &settings),
        Action::GetAddressBalance => get_spendable_balance(&args.address, &settings),
        Action::MineBlocks => mine_blocks_wrapper(&args.wallet_name, args.blocks, &settings),
        Action::ListUnspent => list_unspent(&args.wallet_name, &settings),
        Action::GetTx => get_tx(&args.txid, &settings),
        Action::SignTx => sign_tx_wrapper(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, args.utxo_strat, &settings),
        Action::BroadcastTx => broadcast_tx_wrapper( &args.tx_hex, args.max_fee_rate, &settings),
        Action::SendBtc => send_btc(&args.wallet_name, &args.recipient, args.amount, &settings),
        Action::CreatePsbt => create_psbt(&args.wallet_name, &args.recipient, args.amount, args.fee_amount, args.utxo_strat, &settings),
        Action::DecodePsbt => decode_psbt(&args.psbt_hex, &settings),
        Action::AnalyzePsbt => analyze_psbt(&args.psbt_hex, &settings),
        Action::WalletProcessPsbt => process_psbt(&args.wallet_name, &args.psbt_hex, &settings),
        Action::CombinePsbts => combine_psbts(&args.psbts, &settings),
        Action::FinalizePsbt => finalize_psbt(&args.psbt_hex, &settings),
        Action::FinalizePsbtAndBroadcast => finalize_psbt_and_broadcast(&args.psbt_hex, &settings),
        Action::InscribeOrd => regtest_inscribe_ord(&settings)
    }
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
use std::path::PathBuf;
use std::str::FromStr;

use bitcoincore_rpc::json::AddressType;
use bitcoin::{Amount, Address};
use bitcoin::amount::Denomination::Bitcoin;
use clap::Parser;

use super::utils::UTXOStrategy;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, default_value = "settings.toml")]
    pub settings_file: PathBuf,

    /// Name of the wallet
    #[arg(short='w', long, default_value = "default_wallet")]
    pub wallet_name: String,

    /// Name of the multisig wallet
    #[arg(short='m', long, default_value = "multisig_wallet")]
    pub multisig_name: String,

    /// list of wallet names
    #[arg(short='v', long, value_delimiter = ',', default_value = "default_wallet1,default_wallet2,default_wallet3")]
    pub wallet_names: Vec<String>,

    /// required number of signatures for multisig
    #[arg(short='n', long, default_value = "2")]
    pub nrequired: u32,

    /// Address type
    #[arg(short='z', long, value_parser = parse_address_type, default_value = "bech32")]
    pub address_type: AddressType,

    /// Number of blocks to mine
    #[arg(short='b', long, default_value = "1")]
    pub blocks: u64,

    /// Transaction recipient address
    #[arg(short='r', long, value_parser = string_to_address, default_value = "1F1tAaz5x1HUXrCNLbtMDqcw6o5GNn4xqX")] // dummy address, do not use
    pub recipient: Address,

    /// Wallet address
    #[arg(short='a', long, value_parser = string_to_address, default_value = "1F1tAaz5x1HUXrCNLbtMDqcw6o5GNn4xqX")] // dummy address, do not use
    pub address: Address,

    /// Wallet descriptor
    #[arg(short='d', long, default_value = "descriptor-here")]
    pub descriptor: String,

    /// Start index to derive
    #[arg(short='s', long, default_value = "0")]
    pub start: u32,

    /// End index to derive
    #[arg(short='e', long, default_value = "2")]
    pub end: u32,

    /// Transaction amount
    #[arg(short='x', long, value_parser = parse_amount, default_value = "49.9")]
    pub amount: Amount,

    /// Transaction fee
    #[arg(short='f', long, value_parser = parse_amount, default_value = "0.1")]
    pub fee_amount: Amount,

    /// Max transaction fee rate in sat/vB
    #[arg(short='u', long, default_value = "0.1")]
    pub max_fee_rate: f64,

    /// UTXO selection strategy
    #[arg(short='y', long, value_parser = parse_utxo_strategy, default_value = "fifo")]
    pub utxo_strat: UTXOStrategy,

    /// Transaction ID
    #[arg(short='i', long, default_value = "c36d0c020577c2703dc0e202d8f1ac2626d29d81c449f81079b60c6b07263166")] // dummy tx, do not use
    pub txid: String,

    /// Transaction hash
    #[arg(short='t', long, default_value = "dcaf015d7d6fdfc8a7f38f1a17991aa9975bd93109db2d3756e1533b519d4fae")] // dummy tx, do not use
    pub tx_hex: String,

    /// PSBT hash
    #[arg(short='p', long, default_value = "cHNidP8BAH0CAAAAAbleQkslv9ReG8S64ny+JbejMMyMKKNF2SOBOiqVAAAAD9///")] // dummy tx, do not use
    pub psbt_hex: String,

    /// Multiple PSBTs
    #[arg(short='l', long, value_delimiter = ',', default_value = "cHNidP8BAH0CAAAAAbAip9TqQ,cHNidP8BAH0CAAAAAbAip9TqQ")]
    pub psbts: Vec<String>,

    /// Vout
    #[arg(short='o', long, default_value = "0")]
    pub vout: u32,

    /// Transaction confirmations
    #[arg(short='c', long, default_value = "0")]
    pub confirmations: u32,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Parser)]
pub enum Action {
    GetBlockHeight,
    NewWallet,
    GetWalletInfo,
    ListDescriptors,
    NewMultisig,
    GetNewAddress,
    GetAddressInfo,
    DeriveAddresses,
    RescanBlockchain,
    GetBalance,
    GetSpendableBalance,
    MineBlocks,
    ListUnspent,
    GetTx,
    GetTxOut,
    SignTx,
    DecodeRawTx,
    BroadcastTx,
    SendBtc,
    CreatePsbt,
    DecodePsbt,
    AnalyzePsbt,
    WalletProcessPsbt,
    CombinePsbts,
    FinalizePsbt,
    FinalizePsbtAndBroadcast,
    VerifySignedTx,
    InscribeOrd
}

fn parse_amount(s: &str) -> Result<Amount, &'static str> {
    Amount::from_str_in(s, Bitcoin).map_err(|_| "invalid amount")
}

fn string_to_address(addr_str: &str) -> Result<Address, &'static str> {
    match Address::from_str(addr_str) {
        Ok(address) => Ok(address.assume_checked()),
        Err(_) => Err("Invalid address string"),
    }
}

fn parse_address_type(s: &str) -> Result<AddressType, &'static str> {
    match s {
        "legacy" => Ok(AddressType::Legacy),
        "p2sh-segwit" => Ok(AddressType::P2shSegwit),
        "bech32" => Ok(AddressType::Bech32),
        "bech32m" => Ok(AddressType::Bech32m),
        _ => Err("Unknown address type"),
    }
}

fn parse_utxo_strategy(s: &str) -> Result<UTXOStrategy, &'static str> {
    match s {
        "branch-and-bound" => Ok(UTXOStrategy::BranchAndBound),
        "fifo" => Ok(UTXOStrategy::Fifo),
        "largest-first" => Ok(UTXOStrategy::LargestFirst),
        "smallest-first" => Ok(UTXOStrategy::SmallestFirst),
        _ => Err("Unknown UTXO selection strategy"),
    }
}

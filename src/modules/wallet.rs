use log::info;

use serde::Deserialize;

use bitcoin::{Address, Amount, Network, OutPoint, Transaction, Txid};
use bitcoincore_rpc::json::{
    AddressType, GetAddressInfoResult, GetBalancesResult, GetWalletInfoResult, ListUnspentQueryOptions, ListUnspentResultEntry, WalletProcessPsbtResult
};
use bitcoincore_rpc::jsonrpc::serde_json::{json, Value};
use bitcoincore_rpc::{Client, RpcApi};

use crate::modules::errors::WalletError;
use crate::settings::Settings;
use crate::modules::bitcoind::create_rpc_client;

#[derive(Deserialize)]
struct SendResult {
    txid: Txid,
}

pub(crate) struct Wallet {
    client: Client,
    network: Network,
}

impl Wallet {
    pub(crate) fn new(name: &str, settings: &Settings) -> Result<Self, WalletError> {
        let name = name.to_string();
        let client = create_rpc_client(settings, None)?;

        let wallet_list = client.list_wallet_dir()?;

        if wallet_list.contains(&name) {
            let loaded_wallets = client.list_wallets()?;
            
            if !loaded_wallets.contains(&name) {
                info!("Loading wallet {}", name);
                client.load_wallet(&name)?;
            } else {
                info!("Wallet {} already loaded", name);
            }
        } else {
            if !settings.create_wallets {
                return Err(WalletError::WalletCreationDisabled(name));
            }
            info!("Creating wallet {}", name);
            client.create_wallet(&name, None, None, None, None)?;
        }

        Ok(Wallet {
            client: create_rpc_client(settings, Some(&name))?,
            network: settings.network,
        })
    }

    pub(crate) fn new_address(&self, address_type: &AddressType) -> Result<Address, WalletError> {
        let address = self.client
            .get_new_address(None, Some(*address_type))?;
        address.require_network(self.network)
            .map_err(|_| WalletError::AddressNetworkMismatch)
    }

    pub(crate) fn get_balances(&self) -> Result<GetBalancesResult, WalletError> {
        self.client.get_balances().map_err(WalletError::from)
    }

    pub(crate) fn send(&self, address: &Address, amount: Amount) -> Result<OutPoint, WalletError> {
        let output = json!([{
            address.to_string(): amount.to_btc()
        }]);
        let send_result: SendResult = self
            .client
            .call("send", &[output, Value::Null, "unset".into(), 1.into()])?;
        let txid = send_result.txid;
    
        let transaction_info = self.client
            .get_transaction(&txid, None)?;
        let target_vout = transaction_info.details.iter()
            .find_map(|details| {
                details.address.as_ref()
                    .filter(|&tx_address| tx_address == address)
                    .map(|_| details.vout)
            })
            .ok_or(WalletError::AddressNotFound)?;
    
        Ok(OutPoint {
            txid,
            vout: target_vout,
        })
    }

    pub(crate) fn sign_tx(&self, tx: &Transaction) -> Result<Transaction, WalletError> {
        let signed = self.client
            .sign_raw_transaction_with_wallet(tx, None, None)?;
        signed
            .transaction()
            .map_err(|e| WalletError::SigningFailed(e.to_string()))
    }

    pub(crate) fn get_wallet_info(&self) -> Result<GetWalletInfoResult, WalletError> {
        self.client.get_wallet_info().map_err(WalletError::from)
    }

    pub(crate) fn get_address_info(&self, address: &Address) -> Result<GetAddressInfoResult, WalletError> {
        self.client.get_address_info(address).map_err(WalletError::from)
    }
    
    pub(crate) fn list_all_unspent(&self, query_options: Option<ListUnspentQueryOptions>) -> Result<Vec<ListUnspentResultEntry>, WalletError> {
        self.client
            .list_unspent(Some(1), Some(9999999), None, None, query_options)
            .map_err(WalletError::from)
    }

    pub(crate) fn process_psbt(&self, psbt: &str) -> Result<WalletProcessPsbtResult, WalletError> {
        self.client
            .wallet_process_psbt(psbt, None, None, None)
            .map_err(WalletError::from)
    }
}

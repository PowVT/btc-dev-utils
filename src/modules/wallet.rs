use anyhow::{anyhow, Result};
use bitcoin::{Address, Amount, Network, OutPoint, Transaction, Txid};
use bitcoincore_rpc::json::{AddressType, GetBalancesResult, GetWalletInfoResult, ListUnspentQueryOptions, ListUnspentResultEntry, WalletProcessPsbtResult};
use bitcoincore_rpc::jsonrpc::serde_json::{json, Value};
use bitcoincore_rpc::{Client, RpcApi};
use log::{debug, info};
use serde::Deserialize;

use crate::settings::Settings;
use crate::modules::client::create_rpc_client;

pub(crate) struct Wallet {
    client: Client,
    network: Network,
}

impl Wallet {
    pub(crate) fn new(name: &str, settings: &Settings) -> Self {
        let name = name.to_string();

        let client = create_rpc_client(settings, None);
        if client
            .list_wallet_dir()
            .expect("Could not list wallet dir")
            .contains(&name)
        {
            if !client
                .list_wallets()
                .expect("Could not list wallets")
                .contains(&name)
            {
                info!("loading wallet {}", name);
                client.load_wallet(&name).unwrap();
            } else {
                info!("wallet {} already loaded", name);
            }
        } else {
            if !settings.create_wallets {
                panic!(
                    "wallet {} does not exist and the tool is configured to not create new wallets",
                    name
                );
            }
            info!("creating wallet {}", name);
            client
                .create_wallet(&name, None, None, None, None)
                .expect("Could not create wallet");
        }

        Wallet {
            client: create_rpc_client(settings, Some(&name)),
            network: settings.network,
        }
    }

    pub(crate) fn new_address(&self, address_type: &AddressType) -> Result<Address> {
        let address = self
            .client
            .get_new_address(None, Some(*address_type))?;
        Ok(address.require_network(self.network)?)
    }

    pub(crate) fn get_balances(&self) -> Result<GetBalancesResult> {
        let balance = self.client.get_balances()?;
        Ok(balance)
    }

    pub(crate) fn send(&self, address: &Address, amount: Amount) -> Result<OutPoint> {
        let output = json!([{
            address.to_string(): amount.to_float_in(bitcoin::Denomination::Bitcoin)
        }]);
        let send_result: SendResult = self
            .client
            .call("send", &[output, Value::Null, "unset".into(), 1.into()])?;
        let txid = send_result.txid;

        debug!("sent txid: {}", txid);
        let transaction_info = self.client.get_transaction(&txid, None)?;
        let mut target_vout = 0;
        for (_, details) in transaction_info.details.iter().enumerate() {
            if &details.address.clone().unwrap().assume_checked() == address {
                target_vout = details.vout;
                break;
            }
        }
        Ok(OutPoint {
            txid,
            vout: target_vout,
        })
    }

    pub(crate) fn sign_tx(&self, tx: &Transaction) -> Result<Transaction> {
        let signed = self
            .client
            .sign_raw_transaction_with_wallet(tx, None, None)?;
        signed
            .transaction()
            .map_err(|e| anyhow!("signing failed: {}", e))
    }

    pub(crate) fn get_wallet_info(&self) -> Result<GetWalletInfoResult> {
        let info = self.client.get_wallet_info()?;
        Ok(info)
    }
    
    pub(crate) fn list_all_unspent(&self, query_options: Option<ListUnspentQueryOptions>) -> Result<Vec<ListUnspentResultEntry>> {
        let unspent = self.client.list_unspent(
            Some(1),
            Some(9999999),
            None,
            None,
            query_options
        )?;
        Ok(unspent)
    }

    pub(crate) fn process_psbt(&self, psbt: &str) -> Result<WalletProcessPsbtResult> {
        let tx: WalletProcessPsbtResult = self.client.wallet_process_psbt(psbt, None, None, None)?;
        Ok(tx)
    }
}

#[derive(Deserialize)]
struct SendResult {
    txid: Txid,
}
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bitcoin::{Address, Amount, Network, OutPoint, Transaction, Txid};
use bitcoincore_rpc::json::{AddressType, GetBalancesResult, GetRawTransactionResult, GetWalletInfoResult, ListUnspentQueryOptions, ListUnspentResultEntry};
use bitcoincore_rpc::jsonrpc::serde_json::{json, Value};
use bitcoincore_rpc::{Auth, Client, RawTx, RpcApi};
use log::{debug, info};
use serde::Deserialize;

use crate::settings::Settings;

pub(crate) struct Wallet {
    client: Client,
    network: Network,
}

impl Wallet {
    pub(crate) fn new(name: &str, settings: &Settings) -> Self {
        let name = name.to_string();

        let client = Self::create_rpc_client(settings, None);
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
            client: Self::create_rpc_client(settings, Some(&name)),
            network: settings.network,
        }
    }

    pub(crate) fn create_rpc_client(settings: &Settings, wallet_name: Option<&str>) -> Client {
        let port = match settings.network {
            Network::Bitcoin => 8332,
            Network::Testnet => 18332,
            Network::Regtest => 18443,
            Network::Signet => 38332,
            _ => {
                unreachable!("unsupported network")
            }
        };
        // TODO: allow for other authentication
        let auth = Auth::UserPass(
            settings.bitcoin_rpc_username.clone(),
            settings.bitcoin_rpc_password.clone(),
        );

        //let auth = bitcoincore_rpc::Auth::CookieFile("/Users/alex/Library/Application Support/Bitcoin/regtest/.cookie".to_string().parse().unwrap());

        let url = match wallet_name {
            None => format!("http://127.0.0.1:{port}"),
            Some(name) => format!("http://127.0.0.1:{}/wallet/{name}", port),
        };

        Client::new(&url, auth.clone()).unwrap()
    }

    /// broadcast a raw bitcoin transaction (needs to already be network serialized)
    /// optionally specify a max fee rate in sat/vB. This function will automatically convert it to BTC/kB
    /// the sendrawtransaction rpc call takes fee rate in BTC/kB
    /// fee rates greater than 1BTC/kB will be automatically rejected by the rpc call
    pub(crate) fn broadcast_tx(&self, tx: &Vec<u8>, max_fee_rate: Option<f64>) -> Result<Txid> {
        let max_fee_rate = match max_fee_rate {
            Some(fee_rate) => {
                let fee_rate = fee_rate as f64 / 100_000_000.0 * 1000.0;
                format!("{:.8}", fee_rate).parse::<f64>().unwrap()
            }
            None => 0.1, // the default fee rate is 0.1 BTC/kB
        };
        println!("{:?}", max_fee_rate);
        let txid = self.client.call(
            "sendrawtransaction",
            &[json!(tx.raw_hex()), json!(max_fee_rate)],
        )?;
        Ok(txid)
    }

    pub(crate) fn new_wallet_address(&self, address_type: &AddressType) -> Result<Address> {
        let address = self
            .client
            .get_new_address(None, Some(*address_type))?;
        Ok(address.require_network(self.network)?)
    }

    pub(crate) fn mine_blocks(&self, blocks: Option<u64>, address: &Address) -> Result<()> {
        info!("Mining {} blocks", blocks.unwrap_or(1));
        self.client
            .generate_to_address(blocks.unwrap_or(1), &address)?;
        info!("Mined {} blocks to address {}", blocks.unwrap_or(1), address);
        Ok(())
    }

    /// do not use this method for multisig wallets
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

    pub(crate) fn create_raw_transaction(
        &self,
        utxos: &[bitcoincore_rpc::json::CreateRawTransactionInput],
        outs: &HashMap<String, Amount>,
        locktime: Option<i64>,
        replaceable: Option<bool>,
    ) -> Result<Transaction> {
        let tx = self
            .client
            .create_raw_transaction(utxos, outs, locktime, replaceable)?;
        Ok(tx)
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

    pub(crate) fn get_tx(&self, txid: &Txid) -> Result<GetRawTransactionResult> {
        let tx = self.client.get_raw_transaction_info(txid, None)?;
        Ok(tx)
    }
}

#[derive(Deserialize)]
struct SendResult {
    txid: Txid,
}
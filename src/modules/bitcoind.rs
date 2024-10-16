use bitcoincore_rpc::{Auth, Client};
use bitcoin::Network;
use crate::settings::Settings;

use super::errors::ClientError;

pub fn create_rpc_client(settings: &Settings, wallet_name: Option<&str>) -> Result<Client, ClientError> {
    let port = match settings.network {
        Network::Bitcoin => 8332,
        Network::Testnet => 18332,
        Network::Regtest => 18443,
        Network::Signet => 38332,
        _ => return Err(ClientError::UnsupportedNetwork),
    };
    // TODO: allow for other authentication
    let auth = Auth::UserPass(
        settings.bitcoin_rpc_username.clone(),
        settings.bitcoin_rpc_password.clone(),
    );

    // let auth = bitcoincore_rpc::Auth::CookieFile("/Users/alex/Library/Application Support/Bitcoin/regtest/.cookie".to_string().parse().unwrap());

    let url = match wallet_name {
        None => format!("{}:{}", settings.network_url, port),
        Some(name) => format!("{}:{}/wallet/{}", settings.network_url, port, name),
    };

    Client::new(&url, auth.clone()).map_err(ClientError::CannotConnect)
}
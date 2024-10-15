use std::{error::Error, fmt};

use bitcoincore_rpc::{Auth, Client, Error as RpcError};
use bitcoin::Network;
use crate::settings::Settings;

#[derive(Debug)]
pub enum ClientError {
    CannotConnect(RpcError),
    UnsupportedNetwork,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::CannotConnect(err) => write!(f, "Cannot connect to Bitcoin Core: {}", err),
            ClientError::UnsupportedNetwork => write!(f, "Unsupported network"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::CannotConnect(err) => Some(err),
            ClientError::UnsupportedNetwork => None,
        }
    }
}

impl From<RpcError> for ClientError {
    fn from(err: RpcError) -> Self {
        ClientError::CannotConnect(err)
    }
}

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
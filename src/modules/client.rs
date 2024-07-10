use bitcoincore_rpc::{Auth, Client};
use bitcoin::Network;
use crate::settings::Settings;


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

    // let auth = bitcoincore_rpc::Auth::CookieFile("/Users/alex/Library/Application Support/Bitcoin/regtest/.cookie".to_string().parse().unwrap());

    let url = match wallet_name {
        None => format!("http://127.0.0.1:{port}"),
        Some(name) => format!("http://127.0.0.1:{}/wallet/{name}", port),
    };

    Client::new(&url, auth.clone()).unwrap()
}
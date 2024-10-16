use std::path::PathBuf;

use bitcoin::Network;
use serde::{Deserialize, Serialize};

use crate::modules::errors::SettingsError;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub network: Network,
    pub network_url: String,
    pub bitcoin_rpc_username: String,
    pub bitcoin_rpc_password: String,
    pub create_wallets: bool,
}

impl Settings {
    pub(crate) fn from_toml_file(path: &PathBuf) -> Result<Self, SettingsError> {
        let toml = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&toml)?)
    }

    pub(crate) fn to_toml_file(&self, path: &PathBuf) -> Result<(), SettingsError> {
        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            network: Network::Regtest,
            network_url: "http://127.0.0.1".to_string(),
            bitcoin_rpc_username: "user".to_string(),
            bitcoin_rpc_password: "password".to_string(),
            create_wallets: true,
        }
    }
}

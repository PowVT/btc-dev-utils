use std::path::PathBuf;

use anyhow::Result;
use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub network: Network,
    pub bitcoin_rpc_username: String,
    pub bitcoin_rpc_password: String,
}

impl Settings {
    pub(crate) fn to_toml_file(&self, path: &PathBuf) -> Result<()> {
        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            network: Network::Regtest,
            bitcoin_rpc_username: "user".to_string(),
            bitcoin_rpc_password: "password".to_string(),
        }
    }
}

impl Settings {
    pub(crate) fn from_toml_file(path: &PathBuf) -> Result<Self> {
        let toml = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&toml)?)
    }
}
use std::{collections::HashMap, process::{exit, Command}};

use crate::settings::Settings;

pub enum Target {
    Bitcoin,
    Ord
}

pub fn run_command(command: &str, target: Target, settings: &Settings) -> String {
    let mut full_command = String::from(command);

    if let Target::Bitcoin = target {
        let btc_cmd: String = format!(
            "./bitcoin-core/src/bitcoin-cli -{} -rpcuser={} -rpcpassword={} ",
            settings.network,
            settings.bitcoin_rpc_username,
            settings.bitcoin_rpc_password
        );
        full_command = format!("{}{}", btc_cmd, command);
    }

    if let Target::Ord = target {
        let ord_cmd: String = format!(
            "./ord/target/release/ord --{} --bitcoin-rpc-username={} --bitcoin-rpc-password={} ",
            settings.network,
            settings.bitcoin_rpc_username,
            settings.bitcoin_rpc_password
        );
        full_command = format!("{}{}", ord_cmd, command);
    }

    let output = match Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                exit(1); // Terminate the program with exit code 1
            }
        };
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed: {}\nError: {}", full_command, error_message);
        exit(1); // Terminate the program with exit code 1
    };
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    return stdout;
}

pub fn extract_int_ext_xpubs(
    mut xpubs: HashMap<String,String>,
    descriptors_array: Vec<serde_json::Value>,
    i: usize
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Find the correct descriptors for external and internal xpubs
    let external_xpub = descriptors_array
        .iter()
        .find(|desc| {
            desc["desc"].as_str().unwrap_or_default().starts_with("wpkh") && desc["desc"].as_str().unwrap_or_default().contains("/0/*")
        })
        .ok_or("External xpub descriptor not found")?["desc"]
        .as_str().ok_or("External xpub descriptor not found")?
        .to_string();

    let internal_xpub = descriptors_array
        .iter()
        .find(|desc| {
            desc["desc"].as_str().unwrap_or_default().starts_with("wpkh") && desc["desc"].as_str().unwrap_or_default().contains("/1/*")
        })
        .ok_or("Internal xpub descriptor not found")?["desc"]
        .as_str().ok_or("Internal xpub descriptor not found")?
        .to_string();

    // formatting notes: https://bitcoincoredocs.com/descriptors.html
    // split at "]" and take the last part
    let external_xpub_no_path = external_xpub.split("]").last().unwrap().to_string();
    let internal_xpub_no_path = internal_xpub.split("]").last().unwrap().to_string();

    // split at ")" and take the first part
    let external_xpub_no_path = external_xpub_no_path.split(")").next().unwrap().to_string();
    let internal_xpub_no_path = internal_xpub_no_path.split(")").next().unwrap().to_string();

    xpubs.insert(format!("internal_xpub_{}", i + 1), internal_xpub_no_path);
    xpubs.insert(format!("external_xpub_{}", i + 1), external_xpub_no_path);

    Ok(xpubs)
}

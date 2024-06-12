use std::process::{Command, exit};

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
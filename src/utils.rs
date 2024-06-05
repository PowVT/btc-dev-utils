use std::process::{Command, exit};

const BTC_REGTEST: &str = "~/bitcoin/src/bitcoin-cli -regtest -rpcuser=user -rpcpassword=password ";

pub fn run_command(command: &str, include_btc_core_regtest: bool) -> String {
    let mut full_command = String::from(command);
    if include_btc_core_regtest {
        full_command = format!("{}{}", BTC_REGTEST, command);
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
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    stdout
}
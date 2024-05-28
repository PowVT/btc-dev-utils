# btc-dev-utils

## Overview

`btc-dev-utils` is a set of Python scripts designed to automate Bitcoin development tasks using Bitcoin Core in `regtest` or `testnet` mode on macOS. It is ideal for developers who want to test Bitcoin applications without using real Bitcoin on the mainnet.

## Features

- `regtest-mine-sign-tx.py`

  - Start `bitcoind` in regtest mode.
  - Create a Bitcoin wallet with descriptor support.
  - Generate a new address for mining rewards.
  - Mine blocks to accumulate Bitcoin.
  - Check wallet balance.
  - Create and sign raw transactions.
  - Calculate transaction fees and fee rates.
  - Automate mining and cleanup processes.

- `regtest-ord-tx.py`
  - Start `bitcoind` in regtest mode.
  - Create a Bitcoin wallet with descriptor support.
  - Generate a new address for mining rewards.
  - Mine blocks to accumulate Bitcoin.
  - Check wallet balance.
  - Create and sign raw transactions.
  - Calculate transaction fees and fee rates.
  - Automate mining and cleanup processes.

## Requirements

- Python
- Bitcoin Core installed and accessible at `../bitcoin`
- `bitcoind` and `bitcoin-cli` binaries
- Update `RPC_USER` and `RPC_PASSWORD` in the script to match your `bitcoin.conf`
- Ordinals Core installed and accessible at `../ord`
- `ord` binary

## Usage

### Setup

1. **Clone the Repository**

   ```sh
   git clone https://github.com/powvt/btc-dev-utils.git
   cd btc-dev-utils
   ```

2. **Update Configuration**
   If you perfer to use a configuration file, you can update the `RPC_USER` and `RPC_PASSWORD` in the script to match your `bitcoin.conf` file.
   Ensure that your `bitcoin.conf` file located at `~/Library/Application\ Support/Bitcoin/bitcoin.conf` contains the following lines:

   ```ini
   regtest=1
   rpcuser=yourusername
   rpcpassword=yourpassword
   ```

   Replace `yourusername` and `yourpassword` with your actual RPC username and password.

### Running the Script

1. **Run the Scripts**
   ```sh
   python regtest-mine-sign-tx.py
   python regtest-ord-tx.py
   ```

## Notes

- Ensure that the regtest folder at `~/Library/Application\ Support/Bitcoin/regtest` does not exist before starting the script.
- Make sure the `bitcoind` process is not running and the port is free before starting the script.
- Ensure that the regtest folder at `~/Library/Application\ Support/ord/regtest` does not exist before starting the script.
- Make sure the `ord` process is not running and the port (:80) is free before starting the script.
- The script includes sleep commands to allow sufficient time for processes to start and complete.

## License

This project is licensed under the MIT License.

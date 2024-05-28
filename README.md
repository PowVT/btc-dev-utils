# btc-dev

## Overview

`btc-dev` is a set of Python scripts designed to automate Bitcoin development tasks using Bitcoin Core in `regtest` or `testnet` mode on macOS. It is ideal for developers who want to test Bitcoin applications without using real Bitcoin on the mainnet or testnet.

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

## Requirements

- Bitcoin Core installed and accessible at `../bitcoin`
- Python
- `bitcoind` and `bitcoin-cli` binaries
- Update `RPC_USER` and `RPC_PASSWORD` in the script to match your `bitcoin.conf`

## Usage

### Setup

1. **Clone the Repository**

   ```sh
   git clone https://github.com/yourusername/btc-dev.git
   cd btc-dev
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

1. **Run the Script**
   ```sh
   python btc_dev.py
   ```

### Script Breakdown

- **Start `bitcoind` in Regtest Mode**

  ```python
  run_command("../bitcoin/src/bitcoind -regtest -daemon -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings")
  ```

  This starts the Bitcoin daemon in regtest mode with specific configurations to ensure compatibility.

- **Create a Wallet**

  ```python
  run_command('../bitcoin/src/bitcoin-cli -regtest -named createwallet wallet_name="regtest_desc_wallet" descriptors=true')
  ```

- **Generate Mining Address and Mine Blocks**

  ```python
  mining_address = run_command("../bitcoin/src/bitcoin-cli -regtest getnewaddress")
  run_command(f"../bitcoin/src/bitcoin-cli -regtest generatetoaddress 101 {mining_address}")
  ```

- **Check Wallet Balance**

  ```python
  balance = float(run_command("../bitcoin/src/bitcoin-cli -regtest getbalance"))
  ```

- **Create and Sign Raw Transaction**

  ```python
  raw_tx = run_command(f"../bitcoin/src/bitcoin-cli -regtest createrawtransaction '{json.dumps(inputs)}' '{json.dumps(outputs)}'")
  signed_tx = json.loads(run_command(f"../bitcoin/src/bitcoin-cli -regtest signrawtransactionwithwallet {raw_tx}"))
  ```

- **Mine Additional Blocks and Cleanup**
  ```python
  for i in range(50):
      run_command(f"../bitcoin/src/bitcoin-cli -regtest generatetoaddress 1 {mining_address}")
      time.sleep(3)
  run_command("../bitcoin/src/bitcoin-cli -regtest stop")
  run_command("rm -rf ../Library/Application\\ Support/Bitcoin/regtest")
  ```

## Notes

- Ensure that the regtest folder at `~/Library/Application\ Support/Bitcoin/regtest` does not exist before starting the script.
- Make sure the `bitcoind` process is not running and the port is free before starting the script.
- The script includes sleep commands to allow sufficient time for processes to start and complete.

## License

This project is licensed under the MIT License.

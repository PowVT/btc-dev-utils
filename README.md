# btc-dev-utils

## Overview

`btc-dev-utils` is designed to automate Bitcoin development tasks using Bitcoin Core in `regtest` mode on macOS. It is ideal for developers who want to test Bitcoin applications without using real Bitcoin on the mainnet and need a bootstrapped BTC execution environment in which to send test transactions.

Inspiration for this repository came from the [taproot-wizards/purrfect_vault](https://github.com/taproot-wizards/purrfect_vault)

## Requirements

- Bitcoin Core installed and accessible at `~/bitcoin`
   - `bitcoind` and `bitcoin-cli` binaries
- Ordinals Core installed and accessible at `~/ord`
   - `ord` binary

## Running the repository

1. **Clone the Repository**

   ```sh
   git clone https://github.com/powvt/btc-dev-utils.git
   cd btc-dev-utils
   ```

2. **Run the Scripts**

These steps use `just` as a command wrapper. See the `justfile` for executing the commands directly.

   ```sh
   just bootstrap-btc
   just boostrap-ord
   
   # create a btc signed tx
   just sign-tx

   # inscribe ordinal
   just inscribe-ord
   ```

## Notes

- Ensure that the regtest folder at `~/Library/Application\ Support/Bitcoin/regtest` does not exist before starting the script.
- Make sure the `bitcoind` process is not running and the port is free before starting the script.
- Ensure that the regtest folder at `~/Library/Application\ Support/ord/regtest` does not exist before starting the script.
- Make sure the `ord` process is not running and the port (:80) is free before starting the script.
- The script includes sleep commands to allow sufficient time for processes to start and complete.

## License

This project is licensed under the MIT License.

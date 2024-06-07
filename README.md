# btc-dev-utils

## Overview

`btc-dev-utils` is designed to automate Bitcoin development tasks. It is ideal for developers who want to test Bitcoin applications without using real Bitcoin on the mainnet and need a bootstrapped BTC execution environment in which to send test transactions.

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

2. **Run the Demo Scripts**

These steps use `just` as a command wrapper. See the `justfile` for executing the commands directly. Run `just -l` to see a list of all the justfile commands.

The commands `just bootstrap-btc` and `just bootstrap-ord` will need to run in the background in separate terminals. The demo commands will need to be run in a third terminal. After each of the demo commands run, you will need to restart the btc and ord services. You can use `just kill-all` to stop the btc and ord services as well as delete the cache they created in the 'data' folder.

   ```sh

   just bootstrap-btc
   just bootstrap-ord
   
   # create a btc signed tx
   just sign-tx

   # inscribe ordinal
   just inscribe-ord
   ```

3. **Settings**

The `settings.toml` file is a way to configure the Bitcoin network and the network credentials to use.

## TODO

1. Configure a RPC URL for making external BTC network calls for either Testnet or Mainnet
2. BTC network (regtest) to run indefinitely
3. Native Bitcoin and Ord binaries

## License

This project is licensed under the MIT License.

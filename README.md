# A Collection of Bitcoin Development Tools

## Overview

`btc-dev-utils` is designed to automate Bitcoin development tasks. It is ideal for developers who want to test Bitcoin applications without using real Bitcoin on the mainnet and need a bootstrapped BTC execution environment in which to send test transactions.

Inspiration for this repository came from the [taproot-wizards/purrfect_vault](https://github.com/taproot-wizards/purrfect_vault)

## Prerequisites

You will need to be able to build bitcoin-core. Get set up with a C++ compiler for your platform. Additionally, you will need to be able to build rust binaries for running the ord services and the build-in scripts. Be sure you have a working rust installation. Both of these installations are outside the scope of this document.

For reference, here are all the required dependencies for bitcoin (v27.0) [build docs](https://github.com/bitcoin/bitcoin/blob/master/doc/build-osx.md#preparation).

From there, there is a script in this project to install a copy of bitcoin-core and ord locally, and then you can use [Just](https://github.com/casey/just) as a command runner to build and run the demo.

## How to run it

### **Clone the Repository**

   ```sh
   git clone https://github.com/powvt/btc-dev-utils.git
   cd btc-dev-utils
   ```

### **Building and running the demo**

These steps use `just` as a command wrapper. See the `justfile` for executing the commands directly. Type `just -l` to see a list of all the justfile commands.

First run the `just install-deps` command which will clone and setup the [bitcoin-core](https://github.com/bitcoin/bitcoin) and [ordinal](https://github.com/ordinals/ord) repos. Once this command finishes with "Setup complete", you can move on to running the bitcoin and ordinal services. After installing the depenancies build the rust binaries.
```sh
   just install-deps
   just build
```

The commands `just bootstrap-btc` and `just bootstrap-ord` will need to run in the background in separate terminals. The demo commands will need to be run in a third terminal. After each of the demo commands run, you will need to restart the btc and ord services. You can use `just kill-all` to stop the btc and ord services as well as delete the cache they created in the 'data' folder.

#### BTC Examples:
- In one terminal start the bitcoin daemon: 
   ```sh
   just start-bitcoind
   ```

- In another terminal execute commands against the local bitcoin node:
   ```sh
   # create a btc wallet named satoshi
   just create-wallet satoshi

   # get a new address for the wallet
   just get-new-address satoshi

   # kill all services
   just kill-all
   ```

#### All Wallet Commands

> Reference [bitcoin RPC docs](https://developer.bitcoin.org/reference/rpc/index.html) for example inputs of each command

| Command | Inputs | Description |
| ------- | ------ | ----------- |
| `just new-wallet` | `<wallet_name>` | Create a new bitcoin wallet |
| `just get-balance` | `<wallet_name>` | Get the balance of your bitcoin wallet |
| `just get-new-address` | `<wallet_name>` | Using the specified wallet, generate a new receive address |
| `just list-unspent` | `<wallet_name>` | List all UTXOs for the specified wallet |
| `just list-descriptors` | `<wallet_name>` | List all wallet descriptors for the specified wallet |
| `just get-wallet-info` | `<wallet_name>` | Retrieve information related to the specified wallet |
| `just get-address-info` | `<wallet_name> <wallet_address>` | Retrieve information related to a specific address |
| `just derive-addresses` | `<descriptor> <start> <end>` | Derives one or more addresses corresponding to an output descriptor |
| `just sign-tx` | `<wallet_name> <recipient_address> <amount_in_btc> <fee_amount_in_btc> <utxo_selection_strategy>` | Using the specified wallet, sign a transaction sending an amount of BTC to a recipient address. The UTXO selection strategy defaults to FIFO. |
| `just send-btc` | `<wallet_name> <recipient_address> <amount_in_btc>` | Using the specified wallet, this will automatically create, sign, and broadcast a BTC transaction to the network. The wallet will find the appropriate UTXO to use (the stategy cannot be specified here), calculate an appropriate fee for the tx, and send the change back to the sender. |
| `just process-psbt` | `<wallet_name> <psbt_hash>` | Using the specified wallet, sign a PSBT. |

#### All Multisig Commands

| Command | Inputs | Description |
| ------- | ------ | ----------- |
| `just new-multisig` | `<num_required_signatures> <comma_separated_wallet_names> <multisig_name>` | Create a new multisig wallet. The first input is the number of required signatures for the wallet to spend UTXOs. The second input is a comma-separated list of wallet names (no spaces) that will be the signers on the multisig. The last parameter is a name for the multisig wallet. |
| `just create-psbt` | `<multisig_wallet_name> <recipient_address> <amount_in_btc> <fee_amount_in_btc> <utxo_selection_strategy>` | Create a multisig transaction that will need to be signed by the signers on the multisig. The UTXO selection strategy defaults to FIFO. Refer to the wallet command `process-psbt` for signing a PSBT. |
| `just decode-psbt` | `<psbt_hash>` | Retrieve the inputs and outputs for a specific PSBT. |
| `just analyze-psbt` | `<psbt_hash>` | Retrieve network-related information related to a specific PSBT. |

#### All BTC Network Commands

| Command | Inputs | Description |
| ------- | ------ | ----------- |
| `just start-bitcoind` | - | Start a local Regtest bitcoin network for testing purposes. |
| `just kill-all` | - | Terminate the local Regtest chain and clear all cached data. This will also stop the ordinals server and clear that data as well if it is present. |
| `just mine-blocks` | `<wallet_name> <number_of_blocks_to_mine>` | On the Regtest network, mine the specified number of blocks. The program will generate a recipient address for the block rewards. Remember, coinbase transactions are only available for spending after 100 block confirmations. |
| `just get-tx` | `<tx_hash>` | Get information related to a specific transaction that was broadcast to the network. |
| `just broadcast-tx` | `<signed_tx_hash> <max-fee-rate>` | Broadcast a signed transaction to the network. Optionally, pass a fee rate in sats/vByte that is the max fee rate you are willing to broadcast transactions for. Passing a fee rate of `0` will bypass all fee rate restrictions implemented by the wallet. |

#### Oridinals Examples:
- In one terminal start the bitcoin daemon: 
   ```sh
   just start-bitcoind
   ```

- In another terminal start the ordinals server:
   ```sh
   just start-ord
   ```

- In another terminal execute commands against the local bitcoin node and ordinals server:
   ```sh
   # create an inscription
   just inscribe-ord

   # kill both btc and ord services
   just kill-all
   ```

### Note about UTXO Selection Strategies

When generating a signed transaction, you have four options for selecting which UTXOs to spend. These strategies can result in different outcomes, especially if you have many UTXOs in your wallet. Here are the available strategies and some considerations for choosing the right one:

1. **`branch-and-bound`**:
   - **Description**: This strategy exhaustively searches for the optimal combination of UTXOs to minimize change.
   - **Pros**: Can provide the most efficient use of UTXOs by finding the best combination.
   - **Cons**: Computationally intensive and may be slow when dealing with a large number of UTXOs.

2. **`fifo` (First In, First Out)**:
   - **Description**: Selects the oldest UTXOs first.
   - **Pros**: Simple and efficient; can help reduce the number of UTXOs over time.
   - **Cons**: May result in larger transaction sizes if older UTXOs are small.

3. **`largest-first`**:
   - **Description**: Selects the largest UTXOs first.
   - **Pros**: Results in smaller transaction payloads, potentially reducing transaction fees.
   - **Cons**: May leave a large number of small UTXOs in your wallet, leading to inefficiencies later.

4. **`smallest-first`**:
   - **Description**: Selects the smallest UTXOs first.
   - **Pros**: Helps consolidate many small UTXOs, which can be useful for cleanup.
   - **Cons**: Leads to larger transaction sizes, increasing transaction fees.

#### Choosing a Strategy

When selecting a UTXO strategy, consider the following factors:

- **Transaction Size and Fees**: Smaller UTXOs result in larger transaction data and higher fees, while larger UTXOs minimize transaction size and fees.
- **Wallet Cleanup**: Using strategies like `smallest-first` can help clean up many small UTXOs.
- **Performance**: The `branch-and-bound` strategy, while optimal, can be slow with many UTXOs. Simpler strategies like `fifo` and `largest-first` are faster.

By carefully choosing your UTXO selection strategy, you can optimize your transactions for size, fees, or performance based on your specific needs.

### **Settings**

The `settings.toml` file is a way to configure the Bitcoin network and the network credentials to use. If you choose to update the username and/or password be sure to update the justfile as well.


## License

This project is licensed under the MIT License.

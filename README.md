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

#### Bitcoin commands:
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

#### Oridinals commands:
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
   # create a btc wallet named satoshi
   just inscribe-ord

   # kill both btc and ord services
   just kill-all
   ```

### **Settings**

The `settings.toml` file is a way to configure the Bitcoin network and the network credentials to use. If you choose to update the username and/or password be sure to update the justfile as well.


## License

This project is licensed under the MIT License.

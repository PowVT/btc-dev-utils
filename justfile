#################
# Demo Commands #
#################

# list all commands
help:
    RUST_LOG=info ./target/release/btc-dev-utils -h

# get new wallet
new-wallet wallet_name="default_wallet" address_type="bech32m":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -z {{ address_type }} new-wallet

# get wallet info
get-wallet-info wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} get-wallet-info

# list descriptors
list-descriptors wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} list-descriptors

# create a new multisig wallet
new-multisig required_signatures="2" wallet_names="default_wallet1,default_wallet2,default_wallet3" multisig_name="default_multisig_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -n {{ required_signatures }} -v {{ wallet_names }} -m {{ multisig_name }} new-multisig

# get new wallet address
get-new-address wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} get-new-address

# get address info
get-address-info wallet_name="default_wallet" address="address":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -a {{ address }} get-address-info

# derive address
derive-addresses descriptor="descriptor-here" start="0" end="2":
    RUST_LOG=info ./target/release/btc-dev-utils -d "{{ descriptor }}" -s {{ start }} -e {{ end }} derive-addresses

# Rescan the local blockchain for wallet related transactions. Use to import multisig wallet balances
rescan-blockchain:
    RUST_LOG=info ./target/release/btc-dev-utils rescan-blockchain

# get wallet balance
get-balance wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} get-balance

# mine blocks to a particular wallet
mine-blocks wallet_name="default_wallet" blocks="20":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -b {{ blocks }} mine-blocks

# list unspent transactions
list-unspent wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} list-unspent

# get transaction data from transaction ID
get-tx txid="txid":
    RUST_LOG=info ./target/release/btc-dev-utils -i {{ txid }} get-tx

# create a signed BTC transaction
sign-tx wallet_name="default_wallet" recipient="recpient_address" amount="49.99" fee_amount="0.01" utxo_strat="fifo":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -r {{ recipient }} -x {{ amount }} -f {{ fee_amount }} -y {{ utxo_strat }} sign-tx

# broadcast a signed BTC transaction
broadcast-tx tx_hex="tx_hex"  max_fee_rate="10000":
    RUST_LOG=info ./target/release/btc-dev-utils -t {{ tx_hex }} -u {{ max_fee_rate }} broadcast-tx

# send BTC to recipient address
send-btc wallet_name="default_wallet" recipient="recpient_address" amount="10.0":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -r {{ recipient }} -x {{ amount }} send-btc

# create partially signed BTC transaction
create-psbt wallet_name="default_wallet" recipient="recpient_address" amount="49.99" fee_amount="0.01" utxo_strat="fifo":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -r {{ recipient }} -x {{ amount }} -f {{ fee_amount }} -y {{ utxo_strat }} create-psbt

# decode partially signed BTC transaction (gets information about inputs and outputs)
decode-psbt psbt="psbt_hex":
    RUST_LOG=info ./target/release/btc-dev-utils -p {{ psbt }} decode-psbt

# analyze partially signed BTC transaction (provides current state of psbt)
analyze-psbt psbt="psbt_hex":
    RUST_LOG=info ./target/release/btc-dev-utils -p {{ psbt }} analyze-psbt

# Sign partially signed BTC transaction
wallet-process-psbt wallet_name="default_wallet" psbt="psbt_hex":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -p {{ psbt }} wallet-process-psbt

# Combine partially signed BTC transactions
combine-psbts psbts="signed_psbt_1,signed_psbt_2":
    RUST_LOG=info ./target/release/btc-dev-utils -l {{ psbts }} combine-psbts

# Finalize combined partially signed BTC transactions
finalize-psbt psbt="combined_psbt_hex":
    RUST_LOG=info ./target/release/btc-dev-utils -p {{ psbt }} finalize-psbt

# Finalize combined partially signed BTC transactions and broadcast it
finalize-psbt-and-broadcast psbt="combined_psbt_hex":
    RUST_LOG=info ./target/release/btc-dev-utils -p {{ psbt }} finalize-psbt-and-broadcast

# create and ordinals inscription
inscribe-ord:
    RUST_LOG=info ./target/release/btc-dev-utils inscribe-ord

###################################
# Build and boostrapping commands #
###################################

bitcoin_datadir := "./data/bitcoin"
bitcoin_cli := "./bitcoin-core/src/bitcoin-cli -regtest -rpcuser=user -rpcpassword=password"
bitcoind := "./bitcoin-core/src/bitcoind -regtest -rpcuser=user -rpcpassword=password"

ord_datadir := "./data/ord"
ord := "./ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password"

# start Bitcoind server
start-bitcoind *ARGS:
    mkdir -p {{ bitcoin_datadir }}
    {{ bitcoind }} -timeout=15000 -server=1 -txindex=1 -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings -datadir={{bitcoin_datadir}} {{ ARGS }}

# stop Bitcoind server
stop-bitcoind:
    @if lsof -ti :18443 >/dev/null 2>&1; then \
        {{ bitcoin_cli }} stop; \
        echo "bitcoind server on port 18443 stopped."; \
    else \
        echo "No bitcoind server found running on port 18443."; \
    fi

# remove Bitcoind data
clean-bitcoin-data:
    rm -rf {{ bitcoin_datadir }}

# bootstrap BTC chain
bootstrap-btc:
    just clean-bitcoin-data
    just stop-bitcoind
    just start-bitcoind

# start the Ordinal server
start-ord *ARGS:
    mkdir -p {{ ord_datadir }}
    @if lsof -ti :18443 >/dev/null 2>&1; then \
        {{ ord }} --data-dir={{ord_datadir}} {{ ARGS }} server; \
        echo "ord server on port 80 started."; \
    else \
        echo "run just boostrap-btc before starting ord server."; \
    fi 

# kill the Ordinal server
stop-ord:
    @if lsof -ti :80 >/dev/null 2>&1; then \
        kill $(lsof -t -i:80); \
        echo "ord server on port 80 killed."; \
    else \
        echo "No ord server found running on port 80."; \
    fi

# remove Ordinals data
clean-ord-data:
    rm -rf {{ ord_datadir }}

# bootstrap Ordinals server
bootstrap-ord:
    just clean-ord-data
    just stop-ord
    just start-ord

# stop all services and remove all cached data
kill-all:
    just stop-bitcoind
    just stop-ord
    just clean-bitcoin-data
    just clean-ord-data

# build rust binary
build:
    cargo build --release

# install bitcoin and ord dependencies
install-deps:
    bash ./scripts/build_bitcoincore_and_ord.sh
    just build

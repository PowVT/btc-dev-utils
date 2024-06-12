#################
# Demo Commands #
#################

# create a signed BTC transaction
sign-tx wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} sign-tx

# create and ordinals inscription
inscribe-ord:
    RUST_LOG=info ./target/release/btc-dev-utils inscribe-ord

# get new wallet address
get-new-address wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} new-wallet-address

# get wallet balance
get-balance wallet_name="default_wallet":
    RUST_LOG=info ./target/release/btc-dev-utils get-balance

# mine blocks to a particular address
mine-to-address wallet_name="default_wallet" blocks="20":
    RUST_LOG=info ./target/release/btc-dev-utils -w {{ wallet_name }} -b {{ blocks }} mine-to-address


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

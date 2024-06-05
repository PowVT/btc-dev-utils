#################
# Demo Commands #
#################

# create a signed BTC transaction
sign-tx:
    RUST_LOG=info ./target/release/btc-dev-utils sign-tx

# create and ordinals inscription
inscribe-ord:
    RUST_LOG=info ./target/release/btc-dev-utils inscribe-ord


###################################
# Build and boostrapping commands #
###################################

bitcoin_datadir := "./data/bitcoin"
ord_datadir := "./data/ord"
bcli := "~/bitcoin/src/bitcoin-cli -regtest -rpcuser=user -rpcpassword=password"

# start Bitcoind server
start-bitcoind *ARGS:
    mkdir -p {{ bitcoin_datadir }}
    ~/bitcoin/src/bitcoind -regtest -timeout=15000 -server=1 -txindex=1 -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings -rpcuser=user -rpcpassword=password -datadir={{bitcoin_datadir}} {{ ARGS }}

# stop Bitcoind server
stop-bitcoind:
    {{ bcli }} stop

# remove Bitcoind data
clean-bitcoin-data:
    rm -rf {{ bitcoin_datadir }}

# start the Ordinal server
start-ord *ARGS:
    mkdir -p {{ ord_datadir }}
    @if lsof -ti :18443 >/dev/null 2>&1; then \
        ~/ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password --data-dir={{ord_datadir}} {{ ARGS }} server; \
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

# build rust binary
build:
    cargo build --release

# bootstrap BTC chain
bootstrap-btc:
    just build
    just clean-bitcoin-data
    just start-bitcoind

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

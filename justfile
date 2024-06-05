#################
# Demo Commands #
#################

sign-tx:
    RUST_LOG=info ./target/release/btc-dev-utils sign-tx

inscribe-ord:
    RUST_LOG=info ./target/release/btc-dev-utils inscribe-ord


###################################
# Build and boostrapping commands #
###################################

bitcoin_datadir := "./bitcoin-data"
ord_datadir := "~/Library/Application\\ Support/ord/regtest"
bcli := "~/bitcoin/src/bitcoin-cli -regtest -rpcuser=user -rpcpassword=password"

start-bitcoind *ARGS:
    mkdir -p {{ bitcoin_datadir }}
    ~/bitcoin/src/bitcoind -regtest -timeout=15000 -server=1 -txindex=1 -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings -rpcuser=user -rpcpassword=password -datadir={{bitcoin_datadir}} {{ ARGS }}

stop-bitcoind:
    {{ bcli }} stop

clean-bitcoin-data:
    rm -rf {{ bitcoin_datadir }}

start-ord *ARGS:
    @if lsof -ti :18443 >/dev/null 2>&1; then \
        ~/ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password server; \
        echo "ord server on port 80 started."; \
    else \
        echo "run just boostrap-btc before starting ord server."; \
    fi
    

stop-ord:
    @if lsof -ti :80 >/dev/null 2>&1; then \
        kill $(lsof -t -i:80); \
        echo "ord server on port 80 killed."; \
    else \
        echo "No ord server found running on port 80."; \
    fi

clean-ord-data:
    rm -rf {{ ord_datadir }}

build:
    cargo build --release

bootstrap-btc:
    just build
    just clean-bitcoin-data
    just start-bitcoind

bootstrap-ord:
    just clean-ord-data
    just stop-ord
    just start-ord

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
    ~/ord/target/release/ord --regtest --bitcoin-rpc-username=user --bitcoin-rpc-password=password server

stop-ord:
    kill $(lsof -t -i:80)

clean-ord-data:
    rm -rf {{ ord_datadir }}

build:
    cargo build --release

bootstrap:
    just build
    just clean-bitcoin-data
    just start-bitcoind


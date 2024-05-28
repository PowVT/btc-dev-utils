```
# setup ord
git clone git@github.com:ordinals/ord.git
cd ord

cargo build --release

# run a bitcoind server on regtest (hasbitcoin.conf file already)
./bitcoin/src/bitcoind -regtest -daemon -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings -txindex=1

# start the ord server on regtest
./target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword server

# create a ord wallet
./target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet create

# generate a new receiveaddress
./target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet receive

# mine blocks to new address
./bitcoin/src/bitcoin-cli -regtest generatetoaddress 101 bcrt1ps6uffz2n4ehpjkrercehvdf6qdlx7rcs0949n0y6seprr9y0pursz22r3k
```

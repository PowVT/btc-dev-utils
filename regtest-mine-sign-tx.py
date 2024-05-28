import subprocess
import time
import json

BTC_CORE_DIR = "~/bitcoin/src/"

def run_command(command, include_btc_core_dir=False):
    if include_btc_core_dir:
        command = f"{BTC_CORE_DIR}{command}"
    result = subprocess.run(command, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Command failed: {command}\nError: {result.stderr}")
    return result.stdout.strip()

def main():
    # NOTE(powvt): ensure that a regtest folder DOES NOT exists at ~/Library/Application\ Support/Bitcoin/regtest (rm -rf ~/Library/Application\ Support/Bitcoin/regtest)
    # NOTE(powvt): if running the script multiple times, ensure that the regtest chain has been stopped and the port is is running on is killed (bitcoin-cli -regtest stop)
    # NOTE(powvt): ensure the RPC_USER and RPC_PASSWORD in this script match the values in your bitcoin.conf at ~/Library/Application\ Support/Bitcoin/bitcoin.conf

    # Start the bitcoind daemon in regtest mode
    print("Starting regtest bitcoind...")
    # NOTE(powvt): if using regtest, must include -fallbackfee=1.0 -maxtxfee=1.1
    #              on regtest, usually there are not enough transactions so bitcoind cannot give a reliable estimate and, without it, the wallet will not create transactions unless
    #              it is explicitly set the fee.
    # NOTE(powvt): -deprecatedrpc=warnings is needed because when the rpc calls `getnetworkinfo` the warnings field int he sctruct that is returned on regtest is an array
    #              of strings and on testnet/mainnet it is a string. https://github.com/bitcoin/bitcoin/blob/413844f1c2a3d8f7cfef822f348f26df488b03c7/doc/release-notes-29845.md?plain=1
    run_command(f"bitcoind -regtest -daemon -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings")
    time.sleep(2)  # Give bitcoind some time to start

    # Create a wallet
    print("Creating wallet...")
    run_command(f'bitcoin-cli -regtest -named createwallet wallet_name="regtest_desc_wallet" descriptors=true', include_btc_core_dir=True)

    # Generate a new address for mining rewards
    print("Generating mining address...")
    mining_address = run_command(f"bitcoin-cli -regtest getnewaddress", include_btc_core_dir=True)

    # Mine enough blocks to get a balance of at least 50 BTC (each block gives 50 BTC reward)
    # blocks confirm after 100 blocks
    print("Mining blocks...")
    run_command(f"bitcoin-cli -regtest generatetoaddress 101 {mining_address}", include_btc_core_dir=True)
    time.sleep(2)  # Wait a bit for the blocks to be mined

    # Check the wallet balance
    balance = float(run_command(f"bitcoin-cli -regtest getbalance", include_btc_core_dir=True))
    print(f"Wallet balance: {balance} BTC")

    if balance < 50:
        raise Exception("Failed to mine sufficient balance")

    # Generate a new address to send 1 BTC to
    print("Generating recipient address...")
    recipient_address = run_command(f"bitcoin-cli -regtest getnewaddress", include_btc_core_dir=True)

    # Create a raw transaction
    print("Creating raw transaction...")
    unspent = json.loads(run_command(f"bitcoin-cli -regtest listunspent 1 9999999", include_btc_core_dir=True))
    if len(unspent) == 0:
        raise Exception("No unspent transactions found")

    txid = unspent[0]['txid']
    vout = unspent[0]['vout']
    inputs = [{"txid": txid, "vout": vout}]
    outputs = {recipient_address: 49.9999}

    # Bitcoin config default: maxfeerate = 1 BTC/kvB
    # UTXO's cannot be partially spent
    # fee = inputs - outputs = 50 BTC - 49.9999 BTC = 0.0001 BTC

    raw_tx = run_command(f"bitcoin-cli -regtest createrawtransaction '{json.dumps(inputs)}' '{json.dumps(outputs)}'", include_btc_core_dir=True)

    # Sign the raw transaction
    print("Signing raw transaction...")
    signed_tx = json.loads(run_command(f"bitcoin-cli -regtest signrawtransactionwithwallet {raw_tx}", include_btc_core_dir=True))
    if not signed_tx['complete']:
        raise Exception("Failed to sign the transaction")

    signed_raw_tx = signed_tx['hex']
    print(f"Signed raw transaction: {signed_raw_tx}")

    # Calculate the fee rate
    fee = balance - outputs[recipient_address]
    raw_tx_size = len(signed_raw_tx) // 2  # hex string length // 2: converts string to bytes and round down
    fee_rate = (fee * 1e8) / (raw_tx_size / 1000)  # satoshis per byte

    print(f"Fee: {fee} BTC")
    print(f"Fee rate: {fee_rate} sats/vB")
    print(f"Fee rate: {fee_rate / 1e8 * 1000} BTC/kvB")

    # mine 50 blocks, one every 3 seconds
    print("Mining blocks...")
    for i in range(50):
        run_command(f"bitcoin-cli -regtest generatetoaddress 1 {mining_address}", include_btc_core_dir=True)
        time.sleep(3)

    # View wallet balances
    balance = run_command(f"bitcoin-cli -regtest listaddressgroupings", include_btc_core_dir=True)
    balance = json.loads(balance)
    print(f"Wallet balances: {balance}")

    # Stop the bitcoind daemon
    print("Stopping regtest bitcoind...")
    run_command(f"bitcoin-cli -regtest stop", include_btc_core_dir=True)
    time.sleep(3)

    # Remove the regtest folder
    run_command("rm -rf ../Library/Application\\ Support/Bitcoin/regtest")

if __name__ == "__main__":
    main()

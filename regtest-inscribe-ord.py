import subprocess
import time
import json

BTC_CORE_DIR = "~/bitcoin/src/"

FEE_RATE=15

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
    # NOTE(powvt): -deprecatedrpc=warnings is needed because when the rpc calls `getnetworkinfo` the warnings field in the struct that is returned on regtest is an array
    #              of strings and on testnet/mainnet it is a string. https://github.com/bitcoin/bitcoin/blob/413844f1c2a3d8f7cfef822f348f26df488b03c7/doc/release-notes-29845.md?plain=1
    # NOTE(powvt): -txindex=1 is needed to enable blockchain transaction queries in the ord wallet service
    run_command(f"bitcoind -regtest -daemon -fallbackfee=1.0 -maxtxfee=1.1 -deprecatedrpc=warnings -txindex=1", include_btc_core_dir=True)
    time.sleep(2)  # Give bitcoind some time to start

    # Run the ord service in a new process
    print("Starting ord service...")
    # new process to run ord in regtest mode
    subprocess.Popen(["./target/release/ord", "--regtest", "--bitcoin-rpc-username=yourusername", "--bitcoin-rpc-password=yourpassword", "server"], cwd="../ord")
    time.sleep(2)  # Give ord some time to start

    # Utility for checking the ord server settings, uncomment to view ord server settings
    # print(run_command(f"../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword settings"))

    # Create a wallet
    print("Creating wallet...")
    run_command(f'../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet create')

    # Generate a new address for mining rewards
    print("Generating mining address...")
    mining_address = json.loads(run_command(f"../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet receive"))['addresses'][0]

    # Mine enough blocks to get a balance of at least 50 BTC (each block gives 50 BTC reward)
    # blocks confirm after 100 blocks
    print("Mining blocks...")
    run_command(f"bitcoin-cli -regtest generatetoaddress 101 {mining_address}", include_btc_core_dir=True)
    time.sleep(2)  # Wait a bit for the blocks to be mined

    # Check the wallet balance
    balance = float(run_command(f"bitcoin-cli -regtest -rpcwallet=ord getbalance", include_btc_core_dir=True))
    print(f"Wallet balance: {balance} BTC")

    if balance < 50:
        raise Exception("Failed to mine sufficient balance")
    
    # create inscription commit and reveal transactions
    print(f"Creating inscription...")
    run_command(f"../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet inscribe --fee-rate {FEE_RATE}  --file ./mockOrdContent.txt")

    # mine 10 blocks to confirm the commit and reveal transactions
    run_command(f"bitcoin-cli -regtest generatetoaddress 10 {mining_address}", include_btc_core_dir=True)
    time.sleep(10) # give time for server to catch up, ord server default polling rate is 5s

    # check that the inscription is in the wallet
    inscriptions = run_command(f"../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet inscriptions")
    print(f"Inscription Data: {inscriptions}")

    # View wallet bitcoin balances
    balance = run_command(f"bitcoin-cli -regtest listaddressgroupings", include_btc_core_dir=True)
    balance = json.loads(balance)
    print(f"Wallet bitcoin balances: {balance}")

    # get wallet ordinal balance
    ord_balances = run_command(f"../ord/target/release/ord --regtest --bitcoin-rpc-username=yourusername --bitcoin-rpc-password=yourpassword wallet balance")
    print(f"Wallet ordinal balance: {ord_balances}")

    # kill the ord service on port :80
    print("Stopping ord service...")
    run_command(f"kill $(lsof -t -i:80)")
    time.sleep(5)  # Wait a bit for the ord service to gracefully stop

    # Remove the regtest folder in ord data folder
    run_command("rm -rf ../Library/Application\\ Support/ord/regtest")

    # Stop the bitcoind daemon
    print("Stopping regtest bitcoind...")
    run_command(f"bitcoin-cli -regtest stop", include_btc_core_dir=True)
    time.sleep(3)

    # Remove the regtest folder in bitcoind data folder
    run_command("rm -rf ../Library/Application\\ Support/Bitcoin/regtest")

if __name__ == "__main__":
    main()

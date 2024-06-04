# HAKIFI

## Setup Environement

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Assuming your rust installation is complete, you will require the Wasm rust compiler in order to build Wasm binaries from your smart contracts.

```sh
rustup target add wasm32-unknown-unknown
```
## Installing Nibiru CLI

For installation, you can use the curl command below. For other installations methode please refer to Nibiru Binary Installation Guide.

```sh
curl -s https://get.nibiru.fi/! | bash
```

## Setup Nibiru Testnet

Documentation on connecting Nibiru's networks can be found here. For the purpose of this guide, to connect to Nibur's most stable network, testnet-1, run the following:
```sh
nibid config chain-id nibiru-testnet-1 && \                                         
nibid config broadcast-mode sync && \
nibid config node "https://rpc.testnet-1.nibiru.fi:443" && \
nibid config keyring-backend os && \
nibid config output json
```
## Nibid Keys & Faucet

### Nibid Keys
First setup a wallet with the command below.
```sh
# add wallets for testing
nibid keys add wallet
```
To check which wallet is currently setup, run
```sh
nibid keys show -a wallet
```
## Faucet
Currently, only way to acquire funds for Nibiru's Testnets is via the app.nibiru.fi/faucet (opens new window). First connect your wallet to the recommends Wallet extensions, Leap or Keplr. Then you should be able to request funds. You are limited to once per day.
In order to verify that your funds have been added to your account, you can query your balance by running:
```sh
nibid query bank balances $(nibid keys show -a wallet)
```
## Compile
Then compile your contract using:
```sh
cargo wasm
```

Finally, we need to optimize our generated wasm binary file using CosmWasm Rust Optimizer by running:
```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.0 

```

## Store

Next step for bringing your contract to live is storing it onto the chain:
```sh
FROM=<your-wallet-address>
```
```sh
TXHASH="$(nibid tx wasm store artifacts/hakifi.wasm \
    --from $FROM \
    --gas auto \
    --gas-adjustment 1.5 \
    --gas-prices 0.025unibi \
    --yes | jq -rcs '.[0].txhash')"
```
Now we have the transaction hash stored under $TXHASH. Next we need to obtain the contract's code id and store it in $CODE_ID.

Query the transaction hash:
```sh
nibid q tx $TXHASH > txhash.json
```
Save the CODE_ID for later usage:
```sh
CODE_ID="$(cat txhash.json | jq -r '.logs[0].events[1].attributes[1].value')"
```
##Instantiate
To instantiate your contract, we need to know of the required instantiate message. We can store it locally for future usage:
```sh
echo "<json-message-to-instantiate>" | jq . | tee instantiate_args.json
```
Now that we gave the $FROM address, the contract $CODE_ID and the instantiate json message. We can instantiate by running:
```sh
TXHASH_INIT="$(nibid tx wasm instantiate $CODE_ID \
    "$(cat input/instantiate_args.json)" \
    --admin "$FROM" \
    --label "hakifi" \
    --from $FROM \
    --gas auto \
    --gas-adjustment 1.5 \
    --gas-prices 0.025unibi \
    --yes | jq -rcs '.[0].txhash')"
```
Query the transaction:
```sh
nibid q tx $TXHASH_INIT > txhash.init.json
```
Save the contract address:
```sh
CONTRACT_ADDRESS="$(cat txhash.init.json | jq -r '.logs[0].events[1].attributes[0].value')"
```

Call create insurance:

```sh
nibid tx wasm execute $CONTRACT_ADDRESS '{"create_insurance": {"id_insurance": "id_insurance", "margin": "1000000"}}' --from $FROM --gas auto --gas-adjustment 1.5 --gas-prices 0.025unibi -y  
```

Query insurance:
```sh
nibid query wasm contract-state smart $CONTRACT_ADDRESS '{"get_insurance_info": {"id_insurance": "id_insurance"}}' --output json
```
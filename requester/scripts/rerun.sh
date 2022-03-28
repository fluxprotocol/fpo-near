sh scripts/build.sh

accountId=${accountId:-requester.YOUR_TESTNET_ACCOUNT_ID.testnet}
master=${master:-YOUR_TESTNET_ACCOUNT_ID.testnet}


# default params
network=${network:-testnet}
initialBalance=${initialBalance:-5}
oracle=${oracle:-fpo3.franklinwaller2.testnet}
paymentToken=${paymentToken:-v2.wnear.flux-dev}

# request default params
pair=${pair:-ETH / USD}
provider=${provider:-amberdata.testnet}
amount=${amount:-0}
min_last_update=${min_last_update:-1646094683}

# reset account
NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance

# set up account and deploy requester
# Register account with wNEAR contract and Oracle contract, give 1 NEAR to store with oracle to allow for multiple Data Requests to be made
near call $paymentToken storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000
near call $paymentToken storage_deposit "{\"account_id\": \"$oracle\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000

# Deposit 2 NEAR to get 2 wNEAR tokens to use in your contract
near call $paymentToken near_deposit "{}" --accountId $accountId --amount 2 --gas=300000000000000

NEAR_ENV=$network near deploy --accountId $accountId --wasmFile ./res/requester.wasm 

# initialize the contract
near call $accountId new "{\"oracle\": \"$oracle\",\"payment_token\": \"$paymentToken\"}" --accountId $accountId

# perform first data request
near call $accountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000
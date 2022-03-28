#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-YOUR_TESTNET_ACCOUNT_ID.testnet}
oracle=${oracle:-fpo3.franklinwaller2.testnet}
paymentToken=${paymentToken:-v2.wnear.flux-dev}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
   fi

  shift
done

# Register account with wNEAR contract and Oracle contract, give 1 NEAR to store with oracle to allow for multiple Data Requests to be made
near call $paymentToken storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000
near call $paymentToken storage_deposit "{\"account_id\": \"$oracle\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000

# Deposit 2 NEAR to get 2 wNEAR tokens to use in your contract
near call $paymentToken near_deposit "{}" --accountId $accountId --amount 2 --gas=300000000000000

NEAR_ENV=$network near deploy --accountId $accountId --wasmFile ./res/requester.wasm 

# initialize the contract
near call $accountId new "{\"oracle\": \"$oracle\",\"payment_token\": \"$paymentToken\"}" --accountId $accountId
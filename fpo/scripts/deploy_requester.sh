#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-account.testnet}
oracle=${oracle:-07.oracle.flux-dev}
paymentToken=${paymentToken:-v2.wnear.flux-dev}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

# Register account with wNEAR contract and Oracle contract, give 1 NEAR to store with oracle to allow for multiple Data Requests to be made
near call $paymentToken storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000
near call $oracle storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 1 --gas=300000000000000

# Deposit 2 NEAR to get 2 wNEAR tokens to use in your contract
near call $paymentToken near_deposit "{}" --accountId $accountId --amount 2 --gas=300000000000000

NEAR_ENV=$network near deploy --accountId $accountId --wasmFile ./res/request_interface.wasm --initFunction new --initArgs '{"oracle": "'$oracle'", "payment_token": "'$paymentToken'"}'
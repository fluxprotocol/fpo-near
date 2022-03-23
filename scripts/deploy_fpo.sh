#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-fpo.mennat0.testnet}
# oracle=${oracle:-07.oracle.flux-dev}
# paymentToken=${paymentToken:-v2.wnear.flux-dev}
master=${master:-mennat0.testnet}
initialBalance=${initialBalance:-5}

# reset fpo account
NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance

# Register account with wNEAR contract and Oracle contract, give 1 NEAR to store with oracle to allow for multiple Data Requests to be made
# near call $paymentToken storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 0.00125 --gas=300000000000000
# near call $oracle storage_deposit "{\"account_id\": \"$accountId\"}" --accountId $accountId --amount 1 --gas=300000000000000

# Deposit 2 NEAR to get 2 wNEAR tokens to use in your contract
# near call $paymentToken near_deposit "{}" --accountId $accountId --amount 2 --gas=300000000000000

#deploy fpo
NEAR_ENV=$network near deploy --accountId $accountId --wasmFile target/wasm32-unknown-unknown/release/near_fpo.wasm

#initialize fpo
NEAR_ENV=$network near call $accountId new --accountId $master --gas=300000000000000

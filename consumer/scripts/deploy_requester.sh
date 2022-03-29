#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-req0.mennat0.testnet}
oracle=${oracle:-fpo.mennat0.testnet}
master=${master:-mennat0.testnet}


# reset consumer account
NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance


NEAR_ENV=$network near deploy --accountId $accountId --wasmFile ./res/consumer.wasm 

# initialize the contract
near call $accountId new "{\"oracle\": \"$oracle\"}" --accountId $accountId
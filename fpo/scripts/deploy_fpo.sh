#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-fpo.mennat0.testnet}
master=${master:-mennat0.testnet}
initialBalance=${initialBalance:-5}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

# reset fpo account
NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance

# deploy fpo
NEAR_ENV=$network near deploy --accountId $accountId --wasmFile target/wasm32-unknown-unknown/release/near_fpo.wasm

# initialize fpo
NEAR_ENV=$network near call $accountId new --accountId $master --gas=300000000000000

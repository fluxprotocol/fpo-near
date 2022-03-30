#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-req0.mennat0.testnet}
oracle=${oracle:-fpo.mennat0.testnet}
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

# reset consumer account
NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance

# deployer consumer
NEAR_ENV=$network near deploy --accountId $accountId --wasmFile ./res/consumer.wasm 

# initialize the contract
near call $accountId new "{\"oracle\": \"$oracle\"}" --accountId $accountId
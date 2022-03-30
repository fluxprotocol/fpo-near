#!/bin/bash

accountId=${accountId:-consumer.mennat0.testnet} 
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}
pairs=${pairs:-[\"ETH / USD\",\"ETH / USD\"]}
providers=${providers:-[\"provider0.mennat0.testnet\",\"provider1.mennat0.testnet\"]}
min_last_update=${min_last_update:-0}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

NEAR_ENV=$network near call $fpoAccountId aggregate_median "{\"pairs\": $pairs, \"providers\": $providers, \"min_last_update\":$min_last_update}" --accountId $accountId --gas=300000000000000

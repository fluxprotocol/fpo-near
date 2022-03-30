#!/bin/bash

accountId=${accountId:-provider0.mennat0.testnet} 
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}
pair=${pair:-ETH / USD}
decimals=${decimals:-8}
initialPrice=${initialPrice:-2000}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done


#create pair
NEAR_ENV=$network near call $fpoAccountId create_pair "{\"pair\": \"$pair\", \"decimals\": $decimals, \"initial_price\": \"$initialPrice\"}" --accountId $accountId --gas=300000000000000

#get entry to check
NEAR_ENV=$network near call $fpoAccountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$accountId\"}" --accountId $accountId --gas=300000000000000

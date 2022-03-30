#!/bin/bash

accountId=${accountId:-consumer.mennat0.testnet} 
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}
pair=${pair:-ETH / USD}
provider=${provider:-provider.mennat0.testnet}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

NEAR_ENV=$network near call $fpoAccountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000

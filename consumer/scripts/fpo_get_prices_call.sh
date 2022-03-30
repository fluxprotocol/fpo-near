#!/bin/bash
pairs=${pairs:-[\"ETH / USD\",\"ETH / USD\"]}
providers=${providers:-[\"provider0.mennat0.testnet\",\"provider1.mennat0.testnet\"]}
receiverId=${receiverId:-req0.mennat0.testnet}
accountId=${accountId:-fpo.mennat0.testnet}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

near call $accountId get_prices_call "{\"pairs\": $pairs, \"providers\": $providers, \"receiver_id\": \"$receiverId\"}" --accountId $accountId --gas=300000000000000
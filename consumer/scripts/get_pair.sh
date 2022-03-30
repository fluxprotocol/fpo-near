#!/bin/bash
pair=${pair:-ETH / USD}
provider=${provider:-provider1.mennat0.testnet}
accountId=${accountId:-req0.mennat0.testnet}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

near call $accountId get_pair "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000
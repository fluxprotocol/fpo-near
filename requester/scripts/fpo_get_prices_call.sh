#!/bin/bash

pairs=${pairs:-[\"ETH / USD\",\"ETH / USD\"]}

providers=${providers:-[\"provider0.mennat0.testnet\",\"provider1.mennat0.testnet\"]}

receiver_id=${receiver_id:-req0.mennat0.testnet}


amount=${amount:-0}
accountId=${accountId:-fpo.mennat0.testnet}

near call $accountId get_prices_call "{\"pairs\": $pairs, \"providers\": $providers, \"receiver_id\": \"$receiver_id\"}" --accountId $accountId --gas=300000000000000
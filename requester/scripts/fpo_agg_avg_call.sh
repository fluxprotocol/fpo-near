
accountId=${accountId:-requester.mennat0.testnet} 

masterAccountId=${masterAccountId:-mennat0.testnet}
initialBalance=${initialBalance:-5}
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}

pairs=${pairs:-[\"ETH / USD\",\"ETH / USD\"]}

providers=${providers:-[\"provider0.mennat0.testnet\",\"provider1.mennat0.testnet\"]}

receiver_id=${receiver_id:-req0.mennat0.testnet}

min_last_update=${min_last_update:-0}

# reset requester account
NEAR_ENV=$network near delete $accountId $masterAccountId
NEAR_ENV=$network near create-account $accountId --masterAccount $masterAccountId --initialBalance $initialBalance

# NEAR_ENV=$network near call $fpoAccountId aggregate_avg "{\"pairs\": [\"$pair0\", \"$pair1\"], \"providers\": [\"$provider0\",\"$provider1\"], \"min_last_update\":\"$min_last_update\"}" --accountId $accountId --gas=300000000000000

NEAR_ENV=$network near call $fpoAccountId aggregate_avg_call "{\"pairs\": $pairs, \"providers\": $providers, \"min_last_update\":\"$min_last_update\", \"receiver_id\": \"$receiver_id\"}" --accountId $accountId --gas=300000000000000

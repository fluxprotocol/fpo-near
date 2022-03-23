
accountId=${accountId:-requester.mennat0.testnet} 

masterAccountId=${masterAccountId:-mennat0.testnet}
initialBalance=${initialBalance:-5}
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}

# pair0=${pair0:-ETH / USD}
# pair1=${pair1:-ETH / USD}

pairs=${pairs:-[\"ETH / USD\",\"ETH / USD\"]}

# provider0=${provider0:-provider0.mennat0.testnet}
# provider1=${provider1:-provider1.mennat0.testnet}

providers=${providers:-[\"provider0.mennat0.testnet\",\"provider1.mennat0.testnet\"]}


min_last_update=${min_last_update:-0}

# reset requester account
NEAR_ENV=$network near delete $accountId $masterAccountId
NEAR_ENV=$network near create-account $accountId --masterAccount $masterAccountId --initialBalance $initialBalance

# NEAR_ENV=$network near call $fpoAccountId aggregate_median "{\"pairs\": [\"$pair0\", \"$pair1\"], \"providers\": [\"$provider0\",\"$provider1\"], \"min_last_update\":\"$min_last_update\"}" --accountId $accountId --gas=300000000000000

NEAR_ENV=$network near call $fpoAccountId aggregate_median "{\"pairs\": $pairs, \"providers\": $providers, \"min_last_update\":\"$min_last_update\"}" --accountId $accountId --gas=300000000000000

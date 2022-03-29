
accountId=${accountId:-consumer.mennat0.testnet} 

masterAccountId=${masterAccountId:-mennat0.testnet}
initialBalance=${initialBalance:-5}
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}

pair=${pair:-ETH / USD}
provider=${provider:-provider.mennat0.testnet}

# reset consumer account
NEAR_ENV=$network near delete $accountId $masterAccountId
NEAR_ENV=$network near create-account $accountId --masterAccount $masterAccountId --initialBalance $initialBalance

NEAR_ENV=$network near call $fpoAccountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000

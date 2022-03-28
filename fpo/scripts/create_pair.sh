
accountId=${accountId:-provider0.mennat0.testnet} 
# accountId=${accountId:-provider1.mennat0.testnet} 

masterAccountId=${masterAccountId:-mennat0.testnet}
initialBalance=${initialBalance:-5}
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}

pair=${pair:-ETH / USD}
decimals=${decimals:-8}
initial_price=${initial_price:-2000}
# initial_price=${initial_price:-4000}




# reset provider account
NEAR_ENV=$network near delete $accountId $masterAccountId
NEAR_ENV=$network near create-account $accountId --masterAccount $masterAccountId --initialBalance $initialBalance

#create pair
NEAR_ENV=$network near call $fpoAccountId create_pair "{\"pair\": \"$pair\", \"decimals\": $decimals, \"initial_price\": \"$initial_price\"}" --accountId $accountId --gas=300000000000000


#get entry to check
NEAR_ENV=$network near call $fpoAccountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$accountId\"}" --accountId $accountId --gas=300000000000000

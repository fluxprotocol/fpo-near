
accountId=${accountId:-provider.mennat0.testnet} 

initialBalance=${initialBalance:-5}
fpoAccountId=${fpoAccountId:-fpo.mennat0.testnet}

pair=${pair:-ETH / USD}
price=${price:-3200}

#push price
NEAR_ENV=$network near call $fpoAccountId push_data "{\"pair\": \"$pair\",  \"price\": \"$price\"}" --accountId $accountId --gas=300000000000000

#get entry to check
NEAR_ENV=$network near call $fpoAccountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$accountId\"}" --accountId $accountId --gas=300000000000000

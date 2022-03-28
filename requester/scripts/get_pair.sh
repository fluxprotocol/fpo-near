#!/bin/bash
pair=${pair:-ETH / USD}
# provider=${provider:-provider1.mennat0.testnet}
provider=${provider:-provider0.mennat0.testnet}

amount=${amount:-0}
accountId=${accountId:-req0.mennat0.testnet}

near call $accountId get_pair "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000
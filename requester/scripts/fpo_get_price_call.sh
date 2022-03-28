#!/bin/bash
pair=${pair:-ETH / USD}
# provider=${provider:-provider1.mennat0.testnet}
provider=${provider:-provider0.mennat0.testnet}

receiver_id=${receiver_id:-req0.mennat0.testnet}


amount=${amount:-0}
accountId=${accountId:-fpo.mennat0.testnet}

near call $accountId get_price_call "{\"pair\": \"$pair\", \"provider\": \"$provider\", \"receiver_id\": \"$receiver_id\"}" --accountId $accountId --gas=300000000000000
#!/bin/bash
pair=${pair:-ETH / USD}
provider=${provider:-amberdata.testnet}
amount=${amount:-0}
accountId=${accountId:-YOUR_TESTNET_ACCOUNT_ID.testnet}
min_last_update=${min_last_update:-1646094683}

near call $accountId get_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId --gas=300000000000000
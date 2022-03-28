#!/bin/bash

[[ -z "$1" ]] && echo "Expected syntax: $0 YOUR_REQUESTER_ID" >&2 && exit 1

JSON="{\"sources\": [],\"tags\":[\"sports\",\"nfl\"],\"description\":\"Which team won the NFL Super Bowl in 1996?\",\"outcomes\":[\"Cowboys\",\"Steelers\"],\"challenge_period\":\"120000000000\",\"settlement_time\":\"1\",\"data_type\":\"String\",\"creator\":\"your_account_id.flux-dev\"}"
env NEAR_ENV=testnet near call $1 create_data_request "{\"amount\": \"1000000000000000000000000\", \"payload\": $JSON}" --accountId $1 --amount 0.000000000000000000000001 --gas=300000000000000
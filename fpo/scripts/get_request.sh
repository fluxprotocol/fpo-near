#!/bin/bash

[[ -z "$1" ]] && echo "Expected syntax: $0 YOUR_REQUESTER_ID YOUR_REQUEST_ID" >&2 && exit 1
[[ -z "$2" ]] && echo "Expected syntax: $0 YOUR_REQUESTER_ID YOUR_REQUEST_ID" >&2 && exit 1

near call $1 get_data_request "{\"request_id\": \"$2\"}" --accountId $1

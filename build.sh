#!/usr/bin/env bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release

if [ ! -d ./res ]; then
    mkdir ./res
fi

cp ./target/wasm32-unknown-unknown/release/near_fpo.wasm ./res
cp ./target/wasm32-unknown-unknown/release/consumer.wasm ./res

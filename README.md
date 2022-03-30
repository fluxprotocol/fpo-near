# fpo-near

This repository contains the contract for the Flux first-party oracle on NEAR, as well as a consumer contract for demonstrating its use.

## Setup & test

```bash
$ bash build.sh
$ cargo test
```

## FPO Scripts

First, set your environment variables:

```bash
$ ACCOUNT=myAccount.testnet
$ FPO=fpo.myAccount.testnet
```

### Deploying a new fpo contract

```bash
$ bash fpo/scripts/deploy_fpo.sh --accountId $FPO --master $ACCOUNT
```

### Creating a new price pair

```bash
$ bash fpo/scripts/create_pair.sh --fpoAccountId $FPO --accountId $ACCOUNT --pair ETH/USD --initialPrice 4000
```

### Pushing data

```bash
$ bash fpo/scripts/push_data.sh --fpoAccountId $FPO --accountId $ACCOUNT --pair ETH/USD --price 4000
```

### Getting an entry

```bash
$ bash fpo/scripts/get_entry.sh --fpoAccountId $FPO --accountId $ACCOUNT --pair ETH/USD --provider $ACCOUNT

> { price: '4000', decimals: 8, last_update: 1648651165744573200 }
```

### Fetching median of multiple entries

```bash
$ bash fpo/scripts/agg_median.sh --fpoAccountId $FPO --accountId $ACCOUNT --pairs [\"ETH/USD\"] --providers [\"$ACCOUNT\"]

> '4000'
```

### Fetching mean of multiple entries

```bash
$ bash fpo/scripts/agg_avg.sh --fpoAccountId $FPO --accountId $ACCOUNT --pairs [\"ETH/USD\"] --providers [\"$ACCOUNT\"]

> '4000'
```

## Consumer scripts

First, set your environment variables:

```bash
$ ACCOUNT=myAccount.testnet
$ FPO=fpo.myAccount.testnet
$ CONSUMER=consumer.myAccount.testnet
```

### Deploying a new consumer contract

```bash
$ bash consumer/scripts/deploy_consumer.sh --accountId $CONSUMER --oracle $FPO --master $ACCOUNT
```

### Performing a get_price_call

```bash
$ bash consumer/scripts/fpo_get_price_call.sh --pair "ETH/USD" --provider $ACCOUNT --receiverId $CONSUMER --accountId $FPO
```

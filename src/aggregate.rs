use crate::*;

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Returns the mean of given price pairs from given providers
    pub fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> U128 {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let min_last_update: u64 = min_last_update.into();
        let mut amount_of_providers = providers.len();

        let cumulative = providers.iter().enumerate().fold(0, |s, (i, account_id)| {
            let provider = self.get_provider_expect(&account_id);
            let pair_name = format!("{}-{}", pairs[i], account_id);
            let entry = provider.get_entry_expect(&pair_name);

            // If this entry was updated after the min_last_update take it out of the average
            if u64::from(entry.last_update) < min_last_update {
                amount_of_providers -= 1;
                s
            } else {
                s + u128::from(entry.price)
            }
        });

        U128(cumulative / amount_of_providers as u128)
    }

    /// Returns the median of given price pairs from given providers
    pub fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> U128 {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let min_last_update: u64 = min_last_update.into();
        let mut amount_of_providers = providers.len();

        let mut cumulative =
            providers
                .iter()
                .enumerate()
                .fold(vec![], |mut arr: Vec<u128>, (i, account_id)| {
                    let provider = self.get_provider_expect(&account_id);
                    let pair_name = format!("{}-{}", pairs[i], account_id);
                    let entry = provider.get_entry_expect(&pair_name);

                    // If this entry was updated after the min_last_update take it out of the average
                    if u64::from(entry.last_update) < min_last_update {
                        amount_of_providers -= 1;
                        return arr;
                    } else {
                        arr.push(u128::from(entry.price));
                        return arr;
                    }
                });
        math::median(&mut cumulative)
    }

    /// Returns multiple prices given by specified pairs and providers
    pub fn aggregate_collect(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );
        let min_last_update: u64 = min_last_update.into();
        pairs
            .iter()
            .enumerate()
            .map(|(i, account_id)| {
                let provider = self
                    .providers
                    .get(&account_id)
                    .expect("no provider with account id");
                let pair_name = format!("{}-{}", pairs[i], account_id);
                let entry = provider.get_entry_expect(&pair_name);

                // If this entry was updated after the min_last_update take it out of the average
                if u64::from(entry.last_update) < min_last_update {
                    None
                } else {
                    Some(entry.price)
                }
            })
            .collect()
    }
}

/// Price aggregation tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::U64;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn get_context(
        input: Vec<u8>,
        is_view: bool,
        predecessor_account_id: AccountId,
        current_account_id: AccountId,
    ) -> VMContext {
        VMContext {
            current_account_id,
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn aggregate_avg() {
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .unwrap()
                .price
        );

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), bob())
                .unwrap()
                .price
        );
        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), alice())
                .unwrap()
                .price
        );

        let pairs = vec!["ETH/USD".to_string(), "ETH/USD".to_string()];
        assert_eq!(
            U128(3000),
            fpo_contract.aggregate_avg(pairs, vec![alice(), bob()], U64(0))
        );
    }

    #[test]
    fn aggregate_median() {
        let pair = "ETH/USD".to_string();
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair(pair.clone(), 8, U128(2000));
        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry(pair.clone(), env::predecessor_account_id())
                .unwrap()
                .price
        );

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

        fpo_contract.create_pair(pair.clone(), 8, U128(4000));
        assert_eq!(
            U128(4000),
            fpo_contract.get_entry(pair.clone(), bob()).unwrap().price
        );
        assert_eq!(
            U128(2000),
            fpo_contract.get_entry(pair.clone(), alice()).unwrap().price
        );

        let pairs = vec![pair.clone(), pair];
        assert_eq!(
            U128(3000),
            fpo_contract.aggregate_median(pairs, vec![alice(), bob()], U64(0))
        );
    }
}

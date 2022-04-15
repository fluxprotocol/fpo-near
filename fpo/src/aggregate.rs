use crate::*;
// use near_account_id::AccountId;
use near_sdk::Timestamp;
/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Returns the mean of given price pairs from given providers
    pub fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Option<U128> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let min_last_update: u64 = min_last_update;
        let mut amount_of_providers = providers.len();

        let cumulative = providers
            .iter()
            .zip(pairs.iter())
            .fold(0, |s, (account_id, pair)| {
                let provider = self.get_provider_expect(account_id);
                let pair_name = format!("{}:{}", pair, account_id);
                let entry = provider.get_entry_expect(&pair_name);

                // If this entry was updated after the min_last_update take it out of the average
                if entry.last_update < min_last_update {
                    amount_of_providers -= 1;
                    s
                } else {
                    s + u128::from(entry.price)
                }
            });

        if amount_of_providers as u128 == 0 {
            return None;
        }

        Some(U128::from(cumulative / amount_of_providers as u128))
    }

    /// Returns the median of given price pairs from given providers
    pub fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Option<U128> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let min_last_update: u64 = min_last_update;
        let mut amount_of_providers = providers.len();

        let mut cumulative = providers.iter().zip(pairs.iter()).fold(
            vec![],
            |mut arr: Vec<u128>, (account_id, pair)| {
                let provider = self.get_provider_expect(account_id);
                let pair_name = format!("{}:{}", pair, account_id);
                let entry = provider.get_entry_expect(&pair_name);

                // If this entry was updated after the min_last_update take it out of the average
                if entry.last_update < min_last_update {
                    amount_of_providers -= 1;
                    arr
                } else {
                    arr.push(u128::from(entry.price));
                    arr
                }
            },
        );

        if cumulative.is_empty() {
            return None;
        }

        Some(math::median(&mut cumulative))
    }

    /// Returns multiple prices given by specified pairs and providers
    pub fn aggregate_collect(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );
        let min_last_update: u64 = min_last_update;
        providers // Was pairs??
            .iter()
            .zip(pairs.iter())
            .map(|(account_id, pair)| {
                let provider = self
                    .providers
                    .get(account_id)
                    .expect("no provider with account id");
                let pair_name = format!("{}:{}", pair, account_id);
                let entry = provider.get_entry_expect(&pair_name);

                // If this entry was updated after the min_last_update take it out of the average
                if entry.last_update < min_last_update {
                    None
                } else {
                    Some(entry.price)
                }
            })
            .collect()
    }

    /// Wrapper around `aggregate_avg` to return the average prices of multiple pairs
    pub fn aggregate_avg_many(
        &self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<AccountId>>,
        min_last_update: Timestamp,
    ) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        pairs
            .iter()
            .zip(providers.iter())
            .map(|(pairs, providers)| self.aggregate_avg(pairs.to_vec(), providers.to_vec(), min_last_update))
            .collect()
    }

    /// Wrapper around `aggregate_median` to return the median prices of multiple pairs
    pub fn aggregate_median_many(
        &self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<AccountId>>,
        min_last_update: Timestamp,
    ) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        pairs
            .iter()
            .zip(providers.iter())
            .map(|(pairs, providers)| self.aggregate_median(pairs.to_vec(), providers.to_vec(), min_last_update))
            .collect()
    }

    /// Wrapper around `aggregate_collect` to return the prices of multiple pairs
    pub fn aggregate_collect_many(
        &self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<AccountId>>,
        min_last_update: Timestamp,
    ) -> Vec<Vec<Option<U128>>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        pairs
            .iter()
            .zip(providers.iter())
            .map(|(pairs, providers)| self.aggregate_collect(pairs.to_vec(), providers.to_vec(), min_last_update))
            .collect()
    }
}

/// Price aggregation tests
#[cfg(test)]
mod tests {

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use price_pair::STORAGE_COST;

    use super::*;

    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn bob() -> AccountId {
        "bob.near".parse().unwrap()
    }
    fn carol() -> AccountId {
        "carol.near".parse().unwrap()
    }
    fn dina() -> AccountId {
        "dina.near".parse().unwrap()
    }

    fn get_context(
        predecessor_account_id: AccountId,
        current_account_id: AccountId,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(current_account_id.clone())
            .signer_account_id("robert.testnet".parse().unwrap())
            .predecessor_account_id(predecessor_account_id.clone())
            .attached_deposit(STORAGE_COST);
        builder
    }

    #[test]
    fn aggregate_avg() {
        // alice is the signer
        let mut context = get_context(alice(), alice());
        testing_env!(context.build());

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));

        // switch to bob as signer
        context = get_context(bob(), bob());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        // switch to carol as signer
        context = get_context(carol(), carol());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        // switch to dina as signer
        context = get_context(dina(), dina());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), alice())
                .unwrap()
                .price
        );

        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), bob())
                .unwrap()
                .price
        );

        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), carol())
                .unwrap()
                .price
        );
        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), carol())
                .unwrap()
                .price
        );

        let pairs = vec![
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
        ];
        assert_eq!(
            Some(U128(3500)),
            fpo_contract.aggregate_avg(pairs, vec![alice(), bob(), carol(), dina()], 0)
        );
    }

    #[test]
    fn aggregate_median() {
        // alice is the signer
        let mut context = get_context(alice(), alice());
        testing_env!(context.build());

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));

        // switch to bob as signer
        context = get_context(bob(), bob());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));

        // switch to carol as signer
        context = get_context(carol(), carol());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        // switch to dina as signer
        context = get_context(dina(), dina());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), alice())
                .unwrap()
                .price
        );

        assert_eq!(
            U128(2000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), bob())
                .unwrap()
                .price
        );

        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), carol())
                .unwrap()
                .price
        );
        assert_eq!(
            U128(4000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), dina())
                .unwrap()
                .price
        );

        let pairs = vec![
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
        ];
        assert_eq!(
            Some(U128(3000)),
            fpo_contract.aggregate_median(pairs, vec![alice(), bob(), carol(), dina()], 0)
        );
    }

    #[test]
    fn aggregate_median_many() {
        // alice is the signer
        let mut context = get_context(alice(), alice());
        testing_env!(context.build());

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(30000));
        
        // switch to bob as signer
        context = get_context(bob(), bob());
        testing_env!(context.build());
        
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(30000));
        
        // switch to carol as signer
        context = get_context(carol(), carol());
        testing_env!(context.build());
        
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(40000));
        
        // switch to dina as signer
        context = get_context(dina(), dina());
        testing_env!(context.build());
        
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(40000));

        let pairs_eth = vec![
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
            "ETH/USD".to_string(),
        ];
        let pairs_btc = vec![
            "BTC/USD".to_string(),
            "BTC/USD".to_string(),
            "BTC/USD".to_string(),
            "BTC/USD".to_string(),
        ];
        let providers = vec![
            alice(),
            bob(),
            carol(),
            dina(),
        ];
        assert_eq!(
            vec![Some(U128(3000)), Some(U128(35000))],
            fpo_contract.aggregate_median_many(vec![pairs_eth, pairs_btc], vec![providers.clone(), providers], 0)
        );
    }

}

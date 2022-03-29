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

        if cumulative.len() == 0 {
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
        let min_last_update: u64 = min_last_update.into();
        providers // Was pairs??
            .iter()
            .enumerate()
            .map(|(i, account_id)| {
                let provider = self
                    .providers
                    .get(account_id)
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

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

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
            .predecessor_account_id(predecessor_account_id.clone());
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
}

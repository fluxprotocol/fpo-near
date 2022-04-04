use crate::*;
use near_sdk::{
    serde::{Deserialize, Serialize},
    Timestamp,
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub struct PriceEntry {
    pub price: U128,            // Last reported price
    pub decimals: u16,          // Amount of decimals (e.g. if 2, 100 = 1.00)
    pub last_update: Timestamp, // Time of report
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(&mut self, pair: &str, decimals: u16, initial_price: U128) {
        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(Provider::new);

        let pair_name = format!("{}-{}", pair, env::predecessor_account_id());
        assert!(
            provider.pairs.get(&pair_name).is_none(),
            "pair already exists"
        );
        provider.pairs.insert(
            &pair_name,
            &PriceEntry {
                price: initial_price,
                decimals,
                last_update: env::block_timestamp(),
            },
        );

        self.providers
            .insert(&env::predecessor_account_id(), &provider);
    }

    /// Sets the price for a given price pair by a provider
    #[payable]
    pub fn push_data(&mut self, pair: &str, price: U128) {
        let mut provider = self.get_provider_expect(&env::predecessor_account_id());
        let pair_name = format!("{}-{}", pair, env::predecessor_account_id());
        provider.set_price(pair_name, price, env::block_timestamp());
        self.providers
            .insert(&env::predecessor_account_id(), &provider);
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: &str, provider: AccountId) -> Option<PriceEntry> {
        let pair_name = format!("{}-{}", pair, provider);
        let provider = self.get_provider_option(&provider);
        match provider {
            Some(provider) => provider.get_entry_option(&pair_name),
            None => None,
        }
    }

    /// Returns only the price of a price pair by a provider
    pub fn get_price(&self, pair: &str, provider: &AccountId) -> Option<U128> {
        let pair_name = format!("{}-{}", pair, provider);
        let provider = self.get_provider_option(provider);
        match provider {
            Some(provider) => provider
                .get_entry_option(&pair_name)
                .map(|entry| entry.price),
            None => None,
        }
    }

    /// Returns all the data associated with multiple price pairs by associated providers
    pub fn get_prices(&self, pairs: &[String], providers: &[AccountId]) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let mut result = vec![];
        for (i, provider) in providers.iter().enumerate() {
            let pair_name = format!("{}-{}", pairs[i], provider);
            result.push(
                self.get_provider_expect(provider)
                    .get_entry_option(&pair_name)
                    .map(|entry| entry.price),
            );
        }
        result
    }

    /// Checks if a given price pair exists
    pub fn pair_exists(&self, pair: &str, provider: AccountId) -> bool {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider)
            .pairs
            .get(&pair_name)
            .is_some()
    }
}

/// Price pair tests
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
    fn create_pair() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD", env::predecessor_account_id())
        );
    }

    #[test]
    fn create_diff_pairs() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD", env::predecessor_account_id())
        );

        fpo_contract.create_pair("BTC/USD", 8, U128(42000));
        assert_eq!(
            true,
            fpo_contract.pair_exists("BTC/USD", env::predecessor_account_id())
        );

        assert_eq!(
            vec![U128(2500), U128(42000)],
            fpo_contract
                .get_prices(
                    &["ETH/USD", "BTC/USD"],
                    &[env::predecessor_account_id(), env::predecessor_account_id()]
                )
                .into_iter()
                .map(|entry| entry.unwrap())
                .collect::<Vec<U128>>()
        );
    }

    #[test]
    #[should_panic]
    fn create_same_pair() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD", env::predecessor_account_id())
        );

        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
    }

    #[test]
    fn push_data() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD", env::predecessor_account_id())
                .unwrap()
                .price
        );

        fpo_contract.push_data("ETH/USD", U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD", env::predecessor_account_id())
                .unwrap()
                .price
        );
    }

    #[test]
    fn push_data_multiple_providers() {
        let mut context = get_context(alice(), alice());
        testing_env!(context.build());

        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD", 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD", env::predecessor_account_id())
                .unwrap()
                .price
        );

        // switch to bob as signer
        context = get_context(bob(), bob());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD", 8, U128(2700));
        assert_eq!(
            U128(2700),
            fpo_contract.get_entry("ETH/USD", bob()).unwrap().price
        );
        assert_eq!(
            U128(2500),
            fpo_contract.get_entry("ETH/USD", alice()).unwrap().price
        );

        fpo_contract.push_data("ETH/USD", U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD", env::predecessor_account_id())
                .unwrap()
                .price
        );
    }
}

use crate::*;
use near_sdk::{
    serde::{Deserialize, Serialize},
    Timestamp,
};

// maximum cost of storing a new entry in create_pair() - 170 * yocto per byte (1e19 as of 2022-04-14)
#[allow(dead_code)]
pub const STORAGE_COST: u128 = 1_700_000_000_000_000_000_000;

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
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        let initial_storage_usage = env::storage_usage();

        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(Provider::new);

        let pair_name = format!("{}:{}", pair, env::predecessor_account_id());
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

        // check for storage deposit
        let storage_cost =
            env::storage_byte_cost() * u128::from(env::storage_usage() - initial_storage_usage);
        assert!(
            storage_cost <= env::attached_deposit(),
            "Insufficient storage, need {}",
            storage_cost
        );
    }

    /// Sets the price for a given price pair by a provider
    #[payable]
    pub fn push_data(&mut self, pair: String, price: U128) {
        let mut provider = self.get_provider_expect(&env::predecessor_account_id());
        let pair_name = format!("{}:{}", pair, env::predecessor_account_id());
        provider.set_price(pair_name, price, env::block_timestamp());
        self.providers
            .insert(&env::predecessor_account_id(), &provider);
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: String, provider: AccountId) -> Option<PriceEntry> {
        let pair_name = format!("{}:{}", pair, provider);
        let provider = self.get_provider_option(&provider);
        match provider {
            Some(provider) => provider.get_entry_option(&pair_name),
            None => None,
        }
    }

    /// Returns only the price of a price pair by a provider
    pub fn get_price(&self, pair: String, provider: &AccountId) -> Option<U128> {
        let pair_name = format!("{}:{}", pair, provider);
        let provider = self.get_provider_option(provider);
        match provider {
            Some(provider) => provider
                .get_entry_option(&pair_name)
                .map(|entry| entry.price),
            None => None,
        }
    }

    /// Returns all the data associated with multiple price pairs by associated providers
    pub fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let mut result = vec![];
        for (i, provider) in providers.iter().enumerate() {
            let pair_name = format!("{}:{}", pairs[i], provider);
            result.push(
                self.get_provider_expect(provider)
                    .get_entry_option(&pair_name)
                    .map(|entry| entry.price),
            );
        }
        result
    }

    /// Checks if a given price pair exists
    pub fn pair_exists(&self, pair: String, provider: AccountId) -> bool {
        let pair_name = format!("{}:{}", pair, provider);
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
            .predecessor_account_id(predecessor_account_id.clone())
            .attached_deposit(STORAGE_COST);
        builder
    }

    #[should_panic]
    #[test]
    fn pair_name_too_long() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair(
            "1234567890123".to_string(),
            u16::max_value(),
            U128(u128::max_value()),
        );
    }

    #[test]
    fn measure_storage_cost() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();

        let storage_used_before = env::storage_usage();
        fpo_contract.create_pair(
            "123456789012".to_string(),
            u16::max_value(),
            U128(u128::max_value()),
        );

        let storage_used_after = env::storage_usage();
        assert_eq!(storage_used_after - storage_used_before, 170);
    }

    #[test]
    fn create_pair() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id())
        );
    }

    #[test]
    fn create_diff_pairs() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id())
        );

        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(42000));
        assert_eq!(
            true,
            fpo_contract.pair_exists("BTC/USD".to_string(), env::predecessor_account_id())
        );

        assert_eq!(
            vec![U128(2500), U128(42000)],
            fpo_contract
                .get_prices(
                    vec!["ETH/USD".to_string().to_string(), "BTC/USD".to_string()],
                    vec![env::predecessor_account_id(), env::predecessor_account_id()]
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
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id())
        );

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
    }

    #[test]
    fn push_data() {
        let context = get_context(alice(), alice());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .unwrap()
                .price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .unwrap()
                .price
        );
    }

    #[test]
    fn push_data_multiple_providers() {
        let mut context = get_context(alice(), alice());
        testing_env!(context.build());

        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .unwrap()
                .price
        );

        // switch to bob as signer
        context = get_context(bob(), bob());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2700));
        assert_eq!(
            U128(2700),
            fpo_contract
                .get_entry("ETH/USD".to_string(), bob())
                .unwrap()
                .price
        );
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), alice())
                .unwrap()
                .price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .unwrap()
                .price
        );
    }
}

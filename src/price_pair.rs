use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub struct PriceEntry {
    pub price: U128,                   // Last reported price
    pub decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    pub last_update: WrappedTimestamp, // Time of report
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(|| Provider::new());

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
                last_update: env::block_timestamp().into(),
            },
        );

        self.providers
            .insert(&env::predecessor_account_id(), &provider);
    }

    /// Sets the price for a given price pair by a provider
    #[payable]
    pub fn push_data(&mut self, pair: String, price: U128) {
        let mut provider = self.get_provider_expect(&env::predecessor_account_id());
        let pair_name = format!("{}-{}", pair, env::predecessor_account_id());
        provider.set_price(pair_name, price, env::block_timestamp().into());
        self.providers
            .insert(&env::predecessor_account_id(), &provider);
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: String, provider: AccountId) -> Option<PriceEntry> {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider)
            .get_entry_option(&pair_name)
    }

    /// Returns only the price of a price pair by a provider
    pub fn get_price(&self, pair: String, provider: AccountId) -> Option<U128> {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider)
            .get_entry_option(&pair_name)
            .map(|entry| entry.price)
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
    pub fn pair_exists(&self, pair: String, provider: AccountId) -> bool {
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
    use super::*;
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
    fn create_pair() {
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id())
        );
    }

    #[test]
    fn create_diff_pairs() {
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
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
                    vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
                    vec![env::predecessor_account_id(), env::predecessor_account_id()]
                )
                .iter()
                .map(|entry| entry.unwrap())
                .collect::<Vec<U128>>()
        );
    }

    #[test]
    #[should_panic]
    fn create_same_pair() {
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
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
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
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
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

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
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

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

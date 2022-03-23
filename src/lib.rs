mod provider;
mod storage_manager;
mod utils;
use crate::provider::{PriceEntry, Provider};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use storage_manager::AccountStorageBalance;
near_sdk::setup_alloc!();

/// GLOBAL VARIABLES
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
    pub accounts: LookupMap<AccountId, AccountStorageBalance>, // storage map
}

/// STORAGE KEYS
#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Providers,
    Accounts,
}

/// PUBLIC CONTRACT METHODS
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(FPOStorageKeys::Providers),
            accounts: LookupMap::new(FPOStorageKeys::Accounts),
        }
    }

    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        // assert!(
        //     self.providers.get(&env::predecessor_account_id()).is_none(),
        //     "provider already exists"
        // );

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

    /// Checks if a given price pair exists
    pub fn pair_exists(&self, pair: String, provider: AccountId) -> bool {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider)
            .pairs
            .get(&pair_name)
            .is_some()
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
    pub fn get_entry(&self, pair: String, provider: AccountId) -> PriceEntry {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider)
            .get_entry_expect(&pair_name)
    }

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
        println!("SUM OF PRICES{}", cumulative);
        println!("amount_of_providers{}", amount_of_providers);

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
        utils::median(&mut cumulative)
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

    /// Returns all the data associated with a provider
    fn get_provider_expect(&self, account_id: &AccountId) -> Provider {
        self.providers
            .get(account_id)
            .expect("no provider with this account id")
    }
}

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
    fn carol() -> AccountId {
        "carol.near".to_string()
    }
    fn dina() -> AccountId {
        "dina.near".to_string()
    }

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
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

    // mark individual unit tests with #[test] for them to be registered and fired
    #[test]
    fn create_pair() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id())
        );
    }

    #[test]
    fn create_diff_pairs() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        // instantiate a contract variable
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


    }

    #[test]
    #[should_panic]
    fn create_same_pair() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        // instantiate a contract variable
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
        // set up the mock context into the testing environment
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .price
        );
    }

    #[test]
    fn create_different_providers() {
        // set up the mock context into the testing environment
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .price
        );

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2700));
        assert_eq!(
            U128(2700),
            fpo_contract.get_entry("ETH/USD".to_string(), bob()).price
        );
        assert_eq!(
            U128(2500),
            fpo_contract.get_entry("ETH/USD".to_string(), alice()).price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::predecessor_account_id())
                .price
        );
    }

    #[test]
    fn aggregate_avg() {
        // alice is the signer
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
       
        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));


        // switch to carol as signer
        context = get_context(vec![], false, carol(), carol());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        
        // switch to dina as signer
        context = get_context(vec![], false, dina(), dina());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        assert_eq!(
            U128(2000),
            fpo_contract.get_entry("ETH/USD".to_string(), alice()).price
        );

        assert_eq!(
            U128(4000),
            fpo_contract.get_entry("ETH/USD".to_string(), bob()).price
        );

        assert_eq!(
            U128(4000),
            fpo_contract.get_entry("ETH/USD".to_string(), carol()).price
        );
        assert_eq!(
            U128(4000),
            fpo_contract.get_entry("ETH/USD".to_string(), carol()).price
        );

        let pairs = vec!["ETH/USD".to_string(), "ETH/USD".to_string(),"ETH/USD".to_string(), "ETH/USD".to_string()];
        assert_eq!(
            U128(3500),
            fpo_contract.aggregate_avg(pairs, vec![alice(), bob(), carol(), dina()], U64(0))
        );
    }

    #[test]
    fn aggregate_median() {
        // alice is the signer
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        
        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));


        // switch to carol as signer
        context = get_context(vec![], false, carol(), carol());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));

        // switch to dina as signer
        context = get_context(vec![], false, dina(), dina());
        testing_env!(context);

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
 

        assert_eq!(
            U128(2000),
            fpo_contract.get_entry("ETH/USD".to_string(), alice()).price
        );

        assert_eq!(
            U128(2000),
            fpo_contract.get_entry("ETH/USD".to_string(), bob()).price
        );

        assert_eq!(
            U128(4000),
            fpo_contract.get_entry("ETH/USD".to_string(), carol()).price
        );
        assert_eq!(
            U128(4000),
            fpo_contract.get_entry("ETH/USD".to_string(), dina()).price
        );

        let pairs = vec!["ETH/USD".to_string(), "ETH/USD".to_string(),"ETH/USD".to_string(),"ETH/USD".to_string() ];
        assert_eq!(
            U128(3000),
            fpo_contract.aggregate_median(pairs, vec![alice(), bob(), carol(), dina()], U64(0))
        );
    }
}

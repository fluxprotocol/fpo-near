mod helpers;
mod storage_manager;
use std::fmt::Debug;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use storage_manager::AccountStorageBalance;
near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[derive(Debug)]
pub struct PriceEntry {
    price: U128,                   // Last reported price
    decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    last_update: WrappedTimestamp, // Time or report
}

/// PROVIDER VARIABLES
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Provider {
    pub query_fee: u128,
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}-{PROVIDER}" => PriceEntry - e.g.: ETHUSD => PriceEntry
}

/// PROVIDER STORAGE KEYS
#[derive(BorshStorageKey, BorshSerialize)]
enum ProviderStorageKeys {
    Pairs,
}

/// PROVIDER IMPLEMENTATION (INTERNAL)
impl Provider {
    pub fn new() -> Self {
        println!("CREATEING NEW PROVIDER: ACC: {}", env::predecessor_account_id());
        Self {
            query_fee: 0,
            pairs: LookupMap::new(ProviderStorageKeys::Pairs)
        }
    }

    /// Returns all data associated with a price pair
    pub fn get_entry_expect(&self, pair: &String) -> PriceEntry {
        self.pairs
            .get(pair)
            .expect(format!("no price available for {}", pair).as_str())
    }

    /// Sets the fee for querying prices (not yet implemented)
    pub fn set_fee(&mut self, fee: u128) {
        self.query_fee = fee
    }

    /// Sets the answer for a given price pair by a provider
    pub fn set_price(&mut self, pair: String, price: U128) {
        let mut entry = self.pairs.get(&pair).expect("pair does not exist");
        entry.last_update = env::block_timestamp().into();
        entry.price = price;

        self.pairs.insert(&pair, &entry);
    }
}

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
    StorageAccounts
}

/// PUBLIC CONTRACT METHODS
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(FPOStorageKeys::Providers),
            accounts: LookupMap::new(FPOStorageKeys::StorageAccounts)
        }
    }

    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        println!("+++predecessor_account_id = {}", env::predecessor_account_id());
        println!("+++current_account_id = {}", env::current_account_id());
        assert!(self.providers.get(&env::predecessor_account_id()).is_none(), "provider already exists");

        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(||Provider::new());
        
        let pair_name = format!("{}-{}", pair, env::predecessor_account_id());
        println!("PROVIDER PAIR: {:?}", &provider.pairs.get(&pair_name));
        assert!(provider.pairs.get(&pair_name).is_none(), "pair already exists");
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
        let initial_storage_usage = env::storage_usage();

        let mut provider = self.get_provider_expect(&env::predecessor_account_id());
        let pair_name = format!("{}-{}", pair, env::predecessor_account_id());
        provider.set_price(pair_name, price);
        self.providers
            .insert(&env::predecessor_account_id(), &provider);
        
        helpers::refund_storage(initial_storage_usage, env::predecessor_account_id());
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: String, provider: AccountId) -> PriceEntry {
        let pair_name = format!("{}-{}", pair, provider);
        self.get_provider_expect(&provider).get_entry_expect(&pair_name)
    }

    
    // /// Returns an average of prices given by specified pairs and providers
    // pub fn aggregate_avg(
    //     &self,
    //     pairs: Vec<String>,
    //     providers: Vec<AccountId>,
    //     min_last_update: WrappedTimestamp,
    // ) -> U128 {
    //     assert_eq!(
    //         pairs.len(),
    //         providers.len(),
    //         "pairs and provider should be of equal length"
    //     );
    //     let min_last_update: u64 = min_last_update.into();
    //     let mut amount_of_providers = providers.len();

    //     let cum = pairs.iter().enumerate().fold(0, |s, (i, account_id)| {
    //         let provider = self.get_provider_expect(&account_id);
    //         let pair_name = format!("{}-{}", pairs[i], account_id);
    //         let entry = provider.get_entry_expect(&pair_name);

    //         // If this entry was updated after the min_last_update take it out of the average
    //         if u64::from(entry.last_update) < min_last_update {
    //             amount_of_providers -= 1;
    //             return s;
    //         } else {
    //             return s + u128::from(entry.price);
    //         }
    //     });

    //     U128(cum / amount_of_providers as u128)
    // }

    pub fn aggregate_avg(
        &self,
        pair: String,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> U128 {
     
        let min_last_update: u64 = min_last_update.into();
        let mut amount_of_providers = providers.len();

        let cum = providers.iter().fold(0, |s, account_id| {
            let provider = self.get_provider_expect(&account_id);
            let pair_name = format!("{}-{}", pair, account_id);
            let entry = provider.get_entry_expect(&pair_name);

            // If this entry was updated after the min_last_update take it out of the average
            if u64::from(entry.last_update) < min_last_update {
                amount_of_providers -= 1;
                return s;
            } else {
                return s + u128::from(entry.price);
            }
        });
        println!("+++++++++AVERAGE: {:?}", U128(cum / amount_of_providers as u128));

        U128(cum / amount_of_providers as u128)
    }

    pub fn aggregate_median(
        &self,
        pair: String,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> U128 {
     
        let min_last_update: u64 = min_last_update.into();
        let mut amount_of_providers = providers.len();

        let mut cum = providers.iter().fold(vec![], |mut arr: Vec<u128>, account_id| {
            let provider = self.get_provider_expect(&account_id);
            let pair_name = format!("{}-{}", pair, account_id);
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
        println!("Aggregated prices array: {:?}", cum);
        println!("Calculated median {:?}", self.median(&mut cum));
        return self.median(&mut cum);
       
    }


    pub fn mean(&self, numbers: &Vec<u128>) -> U128 {

        let sum: u128 = numbers.iter().sum();
    
       U128::from( sum as u128 / numbers.len() as u128)
    
    }
    pub fn median(&self, numbers: &mut Vec<u128>) -> U128 {

        numbers.sort();
    
        let mid = numbers.len() / 2;
        if numbers.len() % 2 == 0 {
            return self.mean(&vec![numbers[mid - 1], numbers[mid]]) as U128
        } else {
            U128::from(numbers[mid])
        }
    
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
                    return None;
                } else {
                    return Some(entry.price);
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
    use near_sdk::MockedBlockchain;
    use near_sdk::json_types::U64;
    use near_sdk::{testing_env, VMContext};
    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool, predecessor_account_id: AccountId, current_account_id: AccountId) -> VMContext {
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
        assert_eq!(true, fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id()));
    }

    #[test]
    fn push_data() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false, alice(), alice());
        testing_env!(context);
        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(U128(2500), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

        fpo_contract.push_data("ETH/USD".to_string(),  U128(3000));
       
        assert_eq!(U128(3000), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

    }

    #[test]
    fn create_different_providers() {
        // set up the mock context into the testing environment
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(U128(2500), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);
       
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2700));
        assert_eq!(U128(2700), fpo_contract.get_entry("ETH/USD".to_string(), bob()).price);
        assert_eq!(U128(2500), fpo_contract.get_entry("ETH/USD".to_string(), alice()).price);

        fpo_contract.push_data("ETH/USD".to_string(),  U128(3000));
       
        assert_eq!(U128(3000), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

    }

    #[test]
    fn aggregate_avg() {
        // set up the mock context into the testing environment
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        assert_eq!(U128(2000), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);
       
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        assert_eq!(U128(4000), fpo_contract.get_entry("ETH/USD".to_string(), bob()).price);
        assert_eq!(U128(2000), fpo_contract.get_entry("ETH/USD".to_string(), alice()).price);


        
        assert_eq!(U128(3000), fpo_contract.aggregate_avg("ETH/USD".to_string(), vec![alice(), bob()], U64(0)));


    }

    #[test]
    fn aggregate_median() {
        // set up the mock context into the testing environment
        let mut context = get_context(vec![], false, alice(), alice());
        testing_env!(context);

        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2000));
        assert_eq!(U128(2000), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

        // switch to bob as signer
        context = get_context(vec![], false, bob(), bob());
        testing_env!(context);
       
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(4000));
        assert_eq!(U128(4000), fpo_contract.get_entry("ETH/USD".to_string(), bob()).price);
        assert_eq!(U128(2000), fpo_contract.get_entry("ETH/USD".to_string(), alice()).price);


        println!("MEDIAAANN{:?}", fpo_contract.aggregate_median("ETH/USD".to_string(), vec![alice(), bob()], U64(0)));
        assert_eq!(U128(3000), fpo_contract.aggregate_median("ETH/USD".to_string(), vec![alice(), bob()], U64(0)));


    }


   
}
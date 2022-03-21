mod helpers;
mod storage_manager;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use storage_manager::AccountStorageBalance;
near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct PriceEntry {
    price: U128,                   // Last reported price
    decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    last_update: WrappedTimestamp, // Time or report
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Provider {
    pub query_fee: u128,
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}" => PriceEntry - e.g.: ETHUSD => PriceEntry
}

#[derive(BorshStorageKey, BorshSerialize)]
enum ProviderStorageKeys {
    Pairs,
}

impl Provider {
    pub fn new() -> Self {
        Self {
            query_fee: 0,
            pairs: LookupMap::new(ProviderStorageKeys::Pairs)
        }
    }

    pub fn get_entry_expect(&self, pair: &String) -> PriceEntry {
        self.pairs
            .get(pair)
            .expect("no price available for this pair")
    }

    pub fn set_fee(&mut self, fee: u128) {
        self.query_fee = fee
    }

    pub fn set_price(&mut self, pair: String, price: U128) {
        let mut entry = self.pairs.get(&pair).expect("pair does not exist yet");
        entry.last_update = env::block_timestamp().into();
        entry.price = price;

        self.pairs.insert(&pair, &entry);
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
    pub accounts: LookupMap<AccountId, AccountStorageBalance>, // storage map
}

#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Providers,
    StorageAccounts
}

#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(FPOStorageKeys::Providers),
            accounts: LookupMap::new(FPOStorageKeys::StorageAccounts)
        }
    }

    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        // let initial_storage_usage = env::storage_usage();
        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(|| Provider::new());

        assert!(provider.pairs.get(&pair).is_none(), "pair already exists");

        provider.pairs.insert(
            &pair,
            &PriceEntry {
                price: initial_price,
                decimals,
                last_update: env::block_timestamp().into(),
            },
        );

        self.providers
            .insert(&env::predecessor_account_id(), &provider);

        // helpers::refund_storage(initial_storage_usage, env::predecessor_account_id());
    }

    pub fn pair_exists(&self, pair: String, provider: AccountId) -> bool {
        self.get_provider_expect(&provider)
            .pairs
            .get(&pair)
            .is_some()
    }

    #[payable]
    pub fn push_data(&mut self, pair: String, price: U128) {
        let initial_storage_usage = env::storage_usage();

        let mut provider = self.get_provider_expect(&env::predecessor_account_id());
        provider.set_price(pair, price);
        self.providers
            .insert(&env::predecessor_account_id(), &provider);
        
        helpers::refund_storage(initial_storage_usage, env::predecessor_account_id());
    }

    pub fn get_entry(&self, pair: String, provider: AccountId) -> PriceEntry {
        self.get_provider_expect(&provider).get_entry_expect(&pair)
    }

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

        let cum = pairs.iter().enumerate().fold(0, |s, (i, account_id)| {
            let provider = self.get_provider_expect(&account_id);
            let entry = provider.get_entry_expect(&pairs[i]);

            // If this entry was updated after the min_last_update take it out of the average
            if u64::from(entry.last_update) < min_last_update {
                amount_of_providers -= 1;
                return s;
            } else {
                return s + u128::from(entry.price);
            }
        });

        U128(cum / amount_of_providers as u128)
    }

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
                let entry = provider.get_entry_expect(&pairs[i]);

                // If this entry was updated after the min_last_update take it out of the average
                if u64::from(entry.last_update) < min_last_update {
                    return None;
                } else {
                    return Some(entry.price);
                }
            })
            .collect()
    }

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
    use near_sdk::{testing_env, VMContext};

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "jane.testnet".to_string(),
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
        let context = get_context(vec![], false);
        testing_env!(context);
        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(true, fpo_contract.pair_exists("ETH/USD".to_string(), env::predecessor_account_id()));
    }

    #[test]
    fn push_data() {
        // set up the mock context into the testing environment
        let context = get_context(vec![], false);
        testing_env!(context);
        // instantiate a contract variable
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(U128(2500), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

        fpo_contract.push_data("ETH/USD".to_string(),  U128(3000));
       
        assert_eq!(U128(3000), fpo_contract.get_entry("ETH/USD".to_string(), env::predecessor_account_id()).price);

    }

   
}
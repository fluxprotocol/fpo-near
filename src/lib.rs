mod helpers;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
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

impl Provider {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            query_fee: 0,
            pairs: LookupMap::new(format!("p_{}", account_id).as_bytes().to_vec()),
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

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeys {
    Providers,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(StorageKeys::Providers),
        }
    }

    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        let initial_storage_usage = env::storage_usage();
        let mut provider = self
            .providers
            .get(&env::predecessor_account_id())
            .unwrap_or_else(|| Provider::new(&env::predecessor_account_id()));

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

        helpers::refund_storage(initial_storage_usage, env::predecessor_account_id());
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

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use flux_sdk::DataRequestDataType;
//     use near_sdk::json_types::U128;
//     use near_sdk::serde_json;
//     use near_sdk::MockedBlockchain;
//     use near_sdk::{testing_env, VMContext};

//     fn alice() -> AccountId {
//         "alice.near".to_string()
//     }

//     fn oracle() -> AccountId {
//         "oracle.near".to_string()
//     }

//     fn token() -> AccountId {
//         "token.near".to_string()
//     }

//     fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
//         VMContext {
//             current_account_id: alice(),
//             signer_account_id: alice(),
//             signer_account_pk: vec![0, 1, 2],
//             predecessor_account_id: alice(),
//             input,
//             block_index: 0,
//             block_timestamp: 0,
//             account_balance: 10000 * 10u128.pow(24),
//             account_locked_balance: 0,
//             storage_usage: 0,
//             attached_deposit: 0,
//             prepaid_gas: 10u64.pow(18),
//             random_seed: vec![0, 1, 2],
//             is_view,
//             output_data_receivers: vec![],
//             epoch_height: 0,
//         }
//     }

//     #[test]
//     #[should_panic(expected = "ERR_INVALID_ORACLE_ADDRESS")]
//     fn ri_not_oracle() {
//         // let context = get_context(vec![], false);
//         // testing_env!(context);
//         // let contract = FPOContract::new(oracle(), token(), None);
//         // contract.request_ft_transfer(token(), 100, alice());
//     }

// }

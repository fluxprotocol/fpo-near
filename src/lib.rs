mod aggregate;
mod math;
mod price_pair;
mod provider;
mod storage_manager;
use crate::provider::{PriceEntry, Provider};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use storage_manager::AccountStorageBalance;
near_sdk::setup_alloc!();

/// Global variables
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
    pub accounts: LookupMap<AccountId, AccountStorageBalance>, // storage map
}

/// LookupMap keys
#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Providers,
    Accounts,
}

/// Constructor
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(FPOStorageKeys::Providers),
            accounts: LookupMap::new(FPOStorageKeys::Accounts),
        }
    }
}

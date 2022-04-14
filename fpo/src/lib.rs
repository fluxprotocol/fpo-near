mod aggregate;
mod callbacks;
mod math;
mod price_pair;
mod provider;
use crate::provider::Provider;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

/// Global variables
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

/// LookupMap keys
#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Providers,
}

/// Constructor
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new() -> Self {
        Self {
            providers: LookupMap::new(FPOStorageKeys::Providers),
        }
    }
}

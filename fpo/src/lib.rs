mod admin;
mod aggregate;
mod callbacks;
mod math;
mod price_pair;
mod provider;
mod registry;
use crate::provider::Provider;
use crate::registry::Registry;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, PublicKey};

/// Global variables
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub providers: LookupMap<PublicKey, Provider>, // maps:  AccountId => Provider
    pub registries: LookupMap<AccountId, Registry>, // maps:  AccountId => Registry
    pub admin: AccountId,
}

/// LookupMap keys
#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Providers,
    Registries,
}

/// Constructor
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        Self {
            admin,
            providers: LookupMap::new(FPOStorageKeys::Providers),
            registries: LookupMap::new(FPOStorageKeys::Registries),
        }
    }
}

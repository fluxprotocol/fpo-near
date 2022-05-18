mod admin;
mod callbacks;
mod price_pair;


use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, PublicKey};
use price_pair::PriceEntry;

/// Global variables
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FPOContract {
    pub pairs: LookupMap<String, PriceEntry>,
    pub admin: AccountId,
}

/// LookupMap keys
#[derive(BorshStorageKey, BorshSerialize)]
enum FPOStorageKeys {
    Pairs,
}

/// Constructor
#[near_bindgen]
impl FPOContract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        Self {
            admin,
            pairs: LookupMap::new(FPOStorageKeys::Pairs),
        }
    }
}

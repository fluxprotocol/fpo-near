use crate::*;
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub struct PriceEntry {
    pub price: U128,                   // Last reported price
    pub decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    pub last_update: WrappedTimestamp, // Time of report
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Provider {
    pub query_fee: u128,
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}-{PROVIDER}" => PriceEntry - e.g.: ETHUSD => PriceEntry
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum ProviderStorageKeys {
    Pairs,
}

/// Provider methods (internal)
impl Provider {
    pub fn new() -> Self {
        Self {
            query_fee: 0,
            pairs: LookupMap::new(ProviderStorageKeys::Pairs),
        }
    }

    /// Returns all data associated with a price pair
    pub fn get_entry_expect(&self, pair: &String) -> PriceEntry {
        self.pairs
            .get(pair)
            .expect(format!("no price available for {}", pair).as_str())
    }

    /// Returns all data associated with a price pair, returning None if no price is available
    pub fn get_entry_option(&self, pair: &String) -> Option<PriceEntry> {
        self.pairs.get(pair)
    }

    /// Sets the fee for querying prices (not yet implemented)
    pub fn set_fee(&mut self, fee: u128) {
        self.query_fee = fee
    }

    /// Sets the answer for a given price pair by a provider
    pub fn set_price(&mut self, pair: String, price: U128, updated: U64) {
        let mut entry = self.pairs.get(&pair).expect("pair does not exist");
        entry.last_update = updated;
        entry.price = price;

        self.pairs.insert(&pair, &entry);
    }
}

/// Private contract methods
impl FPOContract {
    /// Returns all the data associated with a provider (non-serializable because LookupMap)
    pub fn get_provider_expect(&self, account_id: &AccountId) -> Provider {
        self.providers
            .get(account_id)
            .expect("no provider with this account id")
    }
}

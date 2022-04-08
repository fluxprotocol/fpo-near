use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Balance;
use near_sdk::Timestamp;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Gas, PanicOnDefault, Promise};

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);

#[ext_contract(fpo)]
trait FPO {
    fn get_price(&self, pair: String, provider: AccountId) -> Option<U128>;
    fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Vec<Option<U128>>;
    fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Option<U128>;
    fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Option<U128>;
}

#[ext_contract(ext_self)]
trait RequestResolver {
    fn price_callback(&self) -> Option<U128>;
    fn prices_callback(&self) -> Vec<Option<U128>>;
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub struct PriceEntry {
    price: U128,
    sender: AccountId,
    price_type: PriceType,
}

#[derive(BorshDeserialize, BorshSerialize)]

pub struct Provider {
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}-{PROVIDER}" => PriceEntry - e.g.: ETH/USD => PriceEntry
}

impl Provider {
    pub fn new() -> Self {
        Self {
            pairs: LookupMap::new("ps".as_bytes()),
        }
    }
    pub fn set_pair(&mut self, pair: String, price: &PriceEntry) {
        self.pairs.insert(&pair, price);
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self::new()
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Consumer {
    oracle: AccountId,
    providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone, Copy, PartialEq,
)]
pub enum PriceType {
    Single,
    Multiple,
    Mean,
    Median,
    Collect, // same as multiple but with min_last_update
}

#[near_bindgen]
impl Consumer {
    #[init]
    pub fn new(oracle: AccountId) -> Self {
        Self {
            oracle,
            providers: LookupMap::new("p".as_bytes()),
        }
    }

    /// @dev Called by FPO contract after a `call()` call to forward a price to the consumer.
    pub fn on_price_received(
        &mut self,
        sender_id: AccountId,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    ) {
        for index in 0..providers.len() {
            let provider_account_id = &providers[index];
            let mut provider = self
                .providers
                .get(provider_account_id)
                .unwrap_or_else(Provider::new);
            let pair_name = format!("{}:{}", pairs[index], provider_account_id);

            if price_type == PriceType::Mean || price_type == PriceType::Median {
                match results[0] {
                    Some(result) => {
                        let entry: PriceEntry = PriceEntry {
                            price: result,
                            sender: sender_id.clone(),
                            price_type,
                        };
                        provider.set_pair(pair_name, &entry.clone());
                    }
                    None => log!("Not found"),
                }
            } else {
                match results[index] {
                    Some(result) => {
                        let entry: PriceEntry = PriceEntry {
                            price: result,
                            sender: sender_id.clone(),
                            price_type,
                        };
                        provider.set_pair(pair_name, &entry.clone());
                    }
                    None => log!("Not found"),
                }
            }

            self.providers.insert(provider_account_id, &provider);
        }
    }

    /// @dev Gets a cached price from this contract.
    pub fn get_pair(&self, provider: AccountId, pair: String) -> PriceEntry {
        let pair_name = format!("{}:{}", pair, provider);

        let prov = self
            .providers
            .get(&provider)
            .expect("no provider with this account id");
        prov.pairs.get(&pair_name).expect("No pair found")
    }

    /// @dev Fetches a price from the FPO with the answer forwarded to `price_callback()`.
    pub fn get_price(&self, pair: String, provider: AccountId) -> Promise {
        fpo::get_price(
            pair,
            provider,
            self.oracle.clone(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::price_callback(
            env::current_account_id(),
            0,                      // yocto NEAR to attach to the callback
            Gas(5_000_000_000_000), // gas to attach to the callback
        ))
    }

    /// @dev Fetches prices from the FPO with the answer forwarded to `prices_callback()`.
    pub fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Promise {
        fpo::get_prices(
            pairs,
            providers,
            self.oracle.clone(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::prices_callback(
            env::current_account_id(),
            0,                      // yocto NEAR to attach to the callback
            Gas(5_000_000_000_000), // gas to attach to the callback
        ))
    }

    /// @dev Fetches an averaged price from the FPO with the answer forwarded to `price_callback()`.
    pub fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Promise {
        fpo::aggregate_avg(
            pairs,
            providers,
            min_last_update,
            self.oracle.clone(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::price_callback(
            env::current_account_id(),
            0,                      // yocto NEAR to attach to the callback
            Gas(5_000_000_000_000), // gas to attach to the callback
        ))
    }

    /// @dev Fetches a median price from the FPO with the answer forwarded to `price_callback()`.
    pub fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
    ) -> Promise {
        fpo::aggregate_median(
            pairs,
            providers,
            min_last_update,
            self.oracle.clone(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::price_callback(
            env::current_account_id(),
            0,                      // yocto NEAR to attach to the callback
            Gas(5_000_000_000_000), // gas to attach to the callback
        ))
    }

    /// @dev Handles the callback from the FPO after a price is received.
    #[private]
    pub fn price_callback(
        #[callback_result] result: Result<U128, near_sdk::PromiseError>,
    ) -> Option<U128> {
        if let Ok(res) = result.as_ref() {
            Some(*res)
        } else {
            None
        }
    }

    /// @dev Handles the callback from the FPO after prices are received.
    #[private]
    pub fn prices_callback(
        #[callback_result] result: Result<U128, near_sdk::PromiseError>,
    ) -> Vec<Option<U128>> {
        if let Ok(res) = result.as_ref() {
            vec![Some(*res)]
        } else {
            vec![None]
        }
    }
}

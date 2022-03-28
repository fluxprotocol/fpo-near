use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Balance;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Gas, PanicOnDefault, Promise, PromiseResult,
};

near_sdk::setup_alloc!();

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_RESOLVE_TRANSFER: Gas = 5_000_000_000_000;

#[ext_contract(fpo)]
trait FPO {
    fn get_price(&self, pair: String, provider: AccountId) -> Option<U128>;
    fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Vec<Option<U128>>;
    fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> Option<U128>;
    fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> Option<U128>;
}

#[ext_contract(ext_self)]
trait RequestResolver {
    fn get_price_callback(&self) -> Option<U128>;
    fn get_prices_callback(&self) -> Vec<Option<U128>>;
    fn aggregate_avg_callback(&self) -> Option<U128>;
    fn aggregate_median_callback(&self) -> Option<U128>;
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct PriceEntry {
    price: U128,
    sender: AccountId,
    price_type: PriceType,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Provider {
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}" => PriceEntry - e.g.: ETH/USD => PriceEntry
}

impl Provider {
    pub fn new() -> Self {
        Self {
            pairs: LookupMap::new("ps".as_bytes()),
        }
    }
    pub fn set_pair(&mut self, pair: &String, price: &PriceEntry) {
        self.pairs.insert(pair, price);
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Requester {
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
impl Requester {
    #[init]
    pub fn new(oracle: AccountId) -> Self {
        Self {
            oracle,
            providers: LookupMap::new("p".as_bytes()),
        }
    }

    pub fn on_price_received(
        &mut self,
        sender_id: AccountId,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    ) {
        log!("HELLO FROM REQUESTER on_price_received");
        for provider in providers.iter() {
            let provider_account_id = provider.clone();
            let mut provider = self.providers.get(&provider).unwrap_or(Provider::new());
            for (index, pair) in pairs.iter().enumerate() {
                if price_type == PriceType::Mean || price_type == PriceType::Median {
                    match results[0] {
                        Some(result) => {
                            let entry: PriceEntry = PriceEntry {
                                price: result,
                                sender: sender_id.clone(),
                                price_type: price_type.clone(),
                            };

                            provider.set_pair(&pair, &entry.clone());
                        }
                        None => log!("Not found"),
                    }
                } else {
                    match results[index] {
                        Some(result) => {
                            let entry: PriceEntry = PriceEntry {
                                price: result,
                                sender: sender_id.clone(),
                                price_type: price_type.clone(),
                            };

                            provider.set_pair(&pair, &entry.clone());
                        }
                        None => log!("Not found"),
                    }
                }
            }
            self.providers.insert(&provider_account_id, &provider);
        }
    }
    pub fn get_pair(&self, provider: AccountId, pair: String) -> PriceEntry {
        let prov = self
            .providers
            .get(&provider)
            .expect("no provider with this account id");
        prov.pairs.get(&pair).expect("No pair found")
    }

    pub fn get_price(&self, pair: String, provider: AccountId) -> Promise {
        log!("REQUESTER GET_PRICE");
        fpo::get_price(
            pair.clone(),
            provider.clone(),
            &self.oracle,
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::get_price_callback(
            &env::current_account_id(),
            0,                 // yocto NEAR to attach to the callback
            5_000_000_000_000, // gas to attach to the callback
        ))
    }
    #[private]
    pub fn get_price_callback(&self) -> Option<U128> {
        if env::promise_results_count() != 1 {
            log!("Expected a result on the callback");
            return None;
        }
        log!("get_price_callback+++++++");

        // Get response, return false if failed
        let price: Option<U128> = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                near_sdk::serde_json::from_slice::<Option<U128>>(&value).unwrap()
            }
            _ => {
                log!("Getting info from Pool Party failed");
                return None;
            }
        };

        log!("RETURNING PRICE{:?}", price);
        price
    }

    pub fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Promise {
        log!("REQUESTER GET_PRICE");
        fpo::get_prices(
            pairs.clone(),
            providers.clone(),
            &self.oracle,
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::get_prices_callback(
            &env::current_account_id(),
            0,                 // yocto NEAR to attach to the callback
            5_000_000_000_000, // gas to attach to the callback
        ))
    }
    #[private]
    pub fn get_prices_callback(&self) -> Vec<Option<U128>> {
        if env::promise_results_count() != 1 {
            log!("Expected a result on the callback");
            return vec![None];
        }
        log!("get_price_callback+++++++");

        // Get response, return false if failed
        let prices: Vec<Option<U128>> = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                near_sdk::serde_json::from_slice::<Vec<Option<U128>>>(&value).unwrap()
            }
            _ => {
                log!("Getting info from Pool Party failed");
                return vec![None];
            }
        };

        log!("RETURNING PRICE{:?}", prices);
        prices
    }

    pub fn aggregate_avg(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> Promise {
        log!("REQUESTER GET_PRICE");
        fpo::aggregate_avg(
            pairs.clone(),
            providers.clone(),
            min_last_update.clone(),
            &self.oracle,
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::aggregate_avg_callback(
            &env::current_account_id(),
            0,                 // yocto NEAR to attach to the callback
            5_000_000_000_000, // gas to attach to the callback
        ))
    }
    #[private]
    pub fn aggregate_avg_callback(&self) -> Option<U128> {
        if env::promise_results_count() != 1 {
            log!("Expected a result on the callback");
            return None;
        }
        log!("get_price_callback+++++++");

        // Get response, return false if failed
        let avg: Option<U128> = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                near_sdk::serde_json::from_slice::<Option<U128>>(&value).unwrap()
            }
            _ => {
                log!("Getting info from Pool Party failed");
                return None;
            }
        };

        log!("RETURNING PRICE{:?}", avg);
        avg
    }

    pub fn aggregate_median(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
    ) -> Promise {
        log!("REQUESTER GET_PRICE");
        fpo::aggregate_median(
            pairs.clone(),
            providers.clone(),
            min_last_update.clone(),
            &self.oracle,
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::aggregate_median_callback(
            &env::current_account_id(),
            0,                 // yocto NEAR to attach to the callback
            5_000_000_000_000, // gas to attach to the callback
        ))
    }
    #[private]
    pub fn aggregate_median_callback(&self) -> Option<U128> {
        if env::promise_results_count() != 1 {
            log!("Expected a result on the callback");
            return None;
        }
        log!("get_price_callback+++++++");

        // Get response, return false if failed
        let avg: Option<U128> = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                near_sdk::serde_json::from_slice::<Option<U128>>(&value).unwrap()
            }
            _ => {
                log!("Getting info from Pool Party failed");
                return None;
            }
        };

        log!("RETURNING PRICE{:?}", avg);
        avg
    }
}

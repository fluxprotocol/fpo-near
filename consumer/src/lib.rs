// This contract demonstrates three ways to fetch/consume prices from the FPO contract:
//
// 1. Make a call originating on the FPO contract with any `..._call()`
//    function to forward prices to the `on_prices_received()` function
//    in this contract.

// 3. Make a call originating from this contract with `get_price()`, `get_prices()`,
//     to forward prices to the `price_callback()` or `prices_callback()` function in this contract.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Balance;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Gas, PanicOnDefault, Promise};

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);

#[ext_contract(fpo)]
trait FPO {
    fn get_price(&self, pair: String) -> Option<U128>;
    fn get_prices(&self, pairs: Vec<String>) -> Vec<Option<U128>>;
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub struct Registry {
    pub pairs: Vec<String>,
    pub results: Vec<Option<U128>>,
    pub sender_id: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Consumer {
    oracle: AccountId,
    pairs: LookupMap<String, PriceEntry>, // maps:  AccountId => Provider
    registries: LookupMap<AccountId, Registry>, // maps:  AccountId => Registry
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, Clone, Copy, PartialEq,
)]
pub enum PriceType {
    Single,
    Multiple,
}

#[near_bindgen]
impl Consumer {
    #[init]
    pub fn new(oracle: AccountId) -> Self {
        Self {
            oracle,
            pairs: LookupMap::new("p".as_bytes()),
            registries: LookupMap::new("r".as_bytes()),
        }
    }

    /// @dev Called by FPO contract after a `registry_aggregate_call()` to forward aggregated registry prices to the consumer.
    pub fn on_registry_prices_received(
        &mut self,
        sender_id: AccountId,
        pairs: Vec<String>,
        results: Vec<Option<U128>>,
        registry_owner: AccountId,
    ) {
        self.registries.insert(
            &registry_owner,
            &Registry {
                pairs,
                results,
                sender_id,
            },
        );
    }
    /// @dev Gets a cached registry prices from this contract.
    pub fn get_registry(&self, registry: AccountId) -> Registry {
        self.registries
            .get(&registry)
            .expect("no registry with this account id")
    }

    /// @dev Called by FPO contract after a `..._call()` call to forward a price to the consumer.
    pub fn on_price_received(
        &mut self,
        sender_id: AccountId,
        pairs: Vec<String>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    ) {
        for index in 0..pairs.len() {
            if price_type == PriceType::Single {
                match results[0] {
                    Some(result) => {
                        let entry: PriceEntry = PriceEntry {
                            price: result,
                            sender: sender_id.clone(),
                            price_type,
                        };
                        self.pairs.insert(&pairs[index], &entry.clone());
                    }
                    None => log!("Not found"),
                }
            } else {
                // Multiple
                match results[index] {
                    Some(result) => {
                        let entry: PriceEntry = PriceEntry {
                            price: result,
                            sender: sender_id.clone(),
                            price_type,
                        };
                        self.pairs.insert(&pairs[index], &entry.clone());
                    }
                    None => log!("Not found"),
                }
            }
        }
    }

    /// @dev Gets a cached price from this contract.
    pub fn get_pair(&self, pair: String) -> PriceEntry {
        self.pairs.get(&pair).expect("No pair found")
    }

    /// @dev Fetches a price from the FPO with the answer forwarded to `price_callback()`.
    pub fn get_price(&self, pair: String) -> Promise {
        fpo::get_price(
            pair,
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
    pub fn get_prices(&self, pairs: Vec<String>) -> Promise {
        fpo::get_prices(
            pairs,
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

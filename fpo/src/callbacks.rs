#![allow(clippy::too_many_arguments)]

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, log, Balance, Gas, Promise};
use std::convert::TryInto;

pub const GAS_TO_SEND_PRICE: Gas = Gas(5_000_000_000_000); // Todo: optimize
pub const ZERO_BALANCE: Balance = 0;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub enum PriceType {
    Single,
    Multiple,
}

/// Price consumer trait for consumer contract
#[ext_contract(ext_price_consumer)]
pub trait PriceConsumer {
    fn on_price_received(
        &self,
        sender_id: AccountId,
        pairs: Vec<String>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    );
    fn on_registry_prices_received(
        &self,
        sender_id: AccountId,
        pairs: Vec<String>,
        results: Vec<Option<U128>>,
        registry_owner: AccountId,
    );
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Forwards a price to the price consumer
    pub fn get_price_call(&self, pair: String, receiver_id: AccountId) -> Promise {
        let sender_id = env::predecessor_account_id();
        let price = self.get_price(pair.clone());
        ext_price_consumer::on_price_received(
            sender_id,
            vec![pair],
            PriceType::Single,
            vec![price],
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards prices to the price consumer
    pub fn get_prices_call(&self, pairs: Vec<String>, receiver_id: AccountId) -> Promise {
        let sender_id = env::predecessor_account_id();
        let entries = self.get_prices(pairs.clone());
        log!("entries: {:?}", entries);
        let num_pairs = pairs.len();
        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            PriceType::Multiple,
            entries,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE * num_pairs.try_into().unwrap(),
        )
    }

    /// Calls `registry_aggregate` and forwards the result to the price consumer
    pub fn registry_aggregate_call(
        &self,
        registry_owner: AccountId,
        receiver_id: AccountId,
    ) -> Promise {
        let registry = self
            .registries
            .get(&registry_owner)
            .expect("No registry found");
        let results = self.registry_aggregate_median(registry_owner.clone());
        ext_price_consumer::on_registry_prices_received(
            env::predecessor_account_id(),
            registry.pairs,
            results,
            registry_owner,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }
}

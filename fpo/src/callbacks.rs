#![allow(clippy::too_many_arguments)]

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Timestamp;
use near_sdk::{ext_contract, log, Balance, Gas, Promise};
use std::convert::TryInto;
// use near_account_id::AccountId;
const GAS_TO_SEND_PRICE: Gas = Gas(5_000_000_000_000); // Todo: calculate and optimize
const ZERO_BALANCE: Balance = 0;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub enum PriceType {
    Single,
    Multiple,
    Mean,
    Median,
    Collect, // same as multiple but with min_last_update
    MeanMany,
    MedianMany,
}

/// Price consumer trait for consumer contract
#[ext_contract(ext_price_consumer)]
pub trait PriceConsumer {
    fn on_price_received(
        &self,
        sender_id: AccountId,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    );
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Forwards a price to the price consumer
    pub fn get_price_call(
        &self,
        pair: String,
        provider: AccountId,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let price = self.get_price(pair.clone(), &provider);
        ext_price_consumer::on_price_received(
            sender_id,
            vec![pair],
            vec![provider],
            PriceType::Single,
            vec![price],
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards prices to the price consumer
    pub fn get_prices_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let entries = self.get_prices(pairs.clone(), providers.clone());
        log!("entries: {:?}", entries);
        let num_pairs = pairs.len();
        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            providers,
            PriceType::Multiple,
            entries,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE * num_pairs.try_into().unwrap(),
        )
    }

    /// Forwards the result of aggregate_avg() to the price consumer
    pub fn aggregate_avg_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let avg = self.aggregate_avg(pairs.clone(), providers.clone(), min_last_update);
        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            providers,
            PriceType::Mean,
            vec![avg],
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards the result of aggregate_median() to the price consumer
    pub fn aggregate_median_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let median = self.aggregate_median(pairs.clone(), providers.clone(), min_last_update);
        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            providers,
            PriceType::Median,
            vec![median],
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards the result of aggregate_collect() to the price consumer
    pub fn aggregate_collect_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: Timestamp,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let collect = self.aggregate_collect(pairs.clone(), providers.clone(), min_last_update);
        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            providers,
            PriceType::Collect,
            collect,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards the result of aggregate_avg_many() to the price consumer
    pub fn aggregate_avg_many_call(
        &self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<AccountId>>,
        min_last_update: Timestamp,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let avgs = self.aggregate_avg_many(pairs.clone(), providers.clone(), min_last_update);

        // get the first element of every subarray in `pairs`
        let pairs = pairs
            .iter()
            .map(|p| p.first().unwrap().clone())
            .collect::<Vec<String>>();

        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            vec![], // exclude providers
            PriceType::MeanMany,
            avgs,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }

    /// Forwards the result of aggregate_median_many() to the price consumer
    pub fn aggregate_median_many_call(
        &self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<AccountId>>,
        min_last_update: Timestamp,
        receiver_id: AccountId,
    ) -> Promise {
        let sender_id = env::predecessor_account_id();
        let medians = self.aggregate_avg_many(pairs.clone(), providers.clone(), min_last_update);

        // get the first element of every subarray in `pairs` to submit as associated pair name
        let pairs = pairs
            .iter()
            .map(|p| p.first().unwrap().clone())
            .collect::<Vec<String>>();

        ext_price_consumer::on_price_received(
            sender_id,
            pairs,
            vec![], // exclude providers
            PriceType::MedianMany,
            medians,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }
}

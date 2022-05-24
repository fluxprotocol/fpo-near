use crate::*;
use ed25519_dalek::Verifier;
use near_sdk::collections::LookupSet;
use near_sdk::json_types::U128;
use near_sdk::{
    serde::{Deserialize, Serialize},
    Timestamp,
};
use std::convert::TryFrom;

#[allow(dead_code)]
pub const STORAGE_COST: u128 = 5_700_000_000_000_000_000_000; // was 1_700_000_000_000_000_000_000

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
pub struct PriceEntry {
    pub price: U128,            // Last reported price
    pub decimals: u16,          // Amount of decimals (e.g. if 2, 100 = 1.00)
    pub last_update: Timestamp, // Time of report
    pub signers: Vec<PublicKey>,
    pub latest_round_id: u64,
}

impl PriceEntry {
    pub fn new(price: U128, decimals: u16, last_update: Timestamp) -> Self {
        Self {
            signers: Vec::new(),
            price,
            decimals,
            last_update,
            latest_round_id: 1,
        }
    }
}

/// Private contract methods
impl FPOContract {
    /// Returns all the data associated with a provider (non-serializable because LookupMap)
    pub fn get_entry_expect(&self, pair: &String) -> PriceEntry {
        self.pairs.get(pair).expect("no pair found")
    }

    /// Returns all the data associated with a provider wrapped in an Option
    pub fn get_entry_option(&self, pair: &String) -> Option<PriceEntry> {
        self.pairs.get(pair)
    }

    /// Sets the answer for a given price pair by a provider
    pub fn set_price(&mut self, pair: String, price: U128) {
        let mut entry = self.pairs.get(&pair).expect("pair does not exist");
        entry.last_update = env::block_timestamp();
        entry.latest_round_id = entry.latest_round_id + 1;
        entry.price = price;
        self.pairs.insert(&pair, &entry);
    }
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(
        &mut self,
        pair: String,
        decimals: u16,
        initial_price: U128,
        signers: Vec<PublicKey>,
    ) {
        self.assert_admin();

        let initial_storage_usage = env::storage_usage();

        // make sure the pair wasn't created before
        assert!(self.pairs.get(&pair).is_none(), "pair already exists");

        let mut price_entry = PriceEntry::new(initial_price, decimals, env::block_timestamp());
        price_entry.signers.clone_from(&signers);
        // price_entry.signers.extend(signers);

        self.pairs.insert(&pair, &price_entry);

        // check for storage deposit
        let storage_cost =
            env::storage_byte_cost() * u128::from(env::storage_usage() - initial_storage_usage);
        assert!(
            storage_cost <= env::attached_deposit(),
            "Insufficient storage, need {}",
            storage_cost
        );
    }

    #[payable]
    pub fn push_data_signed(
        &mut self,
        signatures: Vec<Vec<u8>>,
        signers_pks: Vec<PublicKey>,
        pair: String,
        prices: Vec<U128>,
        round_id: u64,
    ) {
        assert_eq!(signatures.len(), prices.len());
        assert_eq!(signatures.len(), signers_pks.len());

        let entry = self.pairs.get(&pair).expect("Pair doesn't exist");
        assert!(entry.latest_round_id == round_id, "Wrong round_id");

        // create a local set to check later for duplicate signature
        let mut signers_set: LookupSet<PublicKey> = LookupSet::new(b"m");

        // verify signatures
        for (index, signature) in signatures.iter().enumerate() {
            assert!(
                entry.signers.contains(&signers_pks[index]),
                "Signer doesn't exist"
            );

            let message = format!("{}:{}:{:?}", pair, round_id, U128::from(prices[index]));
            let data: &[u8] = message.as_bytes();
            let sig: ed25519_dalek::Signature =
                ed25519_dalek::Signature::try_from(signature.as_ref())
                    .expect("Signature should be a valid array of 64 bytes [13, 254, 123, ...]");
            let public_key: ed25519_dalek::PublicKey =
                ed25519_dalek::PublicKey::from_bytes(&signers_pks[index].as_bytes()[1..]).unwrap();
            assert!(
                public_key.verify(data, &sig).is_ok(),
                "Couldn't verify signature"
            );

            // assert answers are in ascending order
            if index < prices.len() - 1 {
                assert!(
                    u128::from(prices[index]) <= u128::from(prices[index + 1]),
                    "Prices not sorted"
                );
            }

            // assert unique signature
            assert!(
                !signers_set.contains(&signers_pks[index]),
                "Duplicate signature"
            );
            signers_set.insert(&signers_pks[index]);
        }

        // calculate median of answers
        let price;
        if prices.len() % 2 == 0 {
            price = (u128::from(prices[(prices.len() / 2) - 1])
                + u128::from(prices[prices.len() / 2]))
                / 2;
        } else {
            price = u128::from(prices[prices.len() / 2]);
        }
        self.set_price(pair, U128::from(price));
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: String) -> Option<PriceEntry> {
        let entry = self.get_entry_option(&pair);
        match entry {
            Some(_) => self.get_entry_option(&pair),
            None => None,
        }
    }

    /// Returns only the price of a price pair by a provider
    pub fn get_price(&self, pair: String) -> Option<U128> {
        self.get_entry_option(&pair).map(|entry| entry.price)
    }

    /// Returns all the data associated with multiple price pairs by associated providers
    pub fn get_prices(&self, pairs: Vec<String>) -> Vec<Option<U128>> {
        let mut result = vec![];
        for pair in pairs.iter() {
            result.push(self.get_entry_option(pair).map(|entry| entry.price));
        }
        result
    }

    /// Checks if a given price pair exists
    pub fn pair_exists(&self, pair: String) -> bool {
        self.get_entry_option(&pair).is_some()
    }
}

#[cfg(test)]
mod tests {

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use super::*;

    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn alicepk() -> PublicKey {
        PublicKey::from(
            "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
                .parse()
                .unwrap(),
        )
    }

    fn get_context(predecessor_account_id: AccountId, signer_pk: PublicKey) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor_account_id.clone())
            .signer_account_pk(signer_pk)
            .attached_deposit(STORAGE_COST);
        builder
    }

    #[test]
    fn measure_storage_cost() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new(alice());

        let storage_used_before = env::storage_usage();
        fpo_contract.create_pair(
            "123456789012".to_string(),
            u16::max_value(),
            U128(u128::max_value()),
            vec![alicepk()],
        );

        let storage_used_after = env::storage_usage();
        assert_eq!(storage_used_after - storage_used_before, 132); // was 170, 350, 124
    }

    #[test]
    fn create_pair() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new(alice());
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500), vec![alicepk()]);
        assert_eq!(true, fpo_contract.pair_exists("ETH/USD".to_string()));
    }

    #[test]
    fn create_diff_pairs() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new(alice());
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500), vec![alicepk()]);
        assert_eq!(true, fpo_contract.pair_exists("ETH/USD".to_string()));

        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(42000), vec![alicepk()]);
        assert_eq!(true, fpo_contract.pair_exists("BTC/USD".to_string()));

        assert_eq!(
            vec![U128(2500), U128(42000)],
            fpo_contract
                .get_prices(vec![
                    "ETH/USD".to_string().to_string(),
                    "BTC/USD".to_string()
                ],)
                .into_iter()
                .map(|entry| entry.unwrap())
                .collect::<Vec<U128>>()
        );
    }

    #[test]
    #[should_panic]
    fn create_same_pair() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new(alice());
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500), vec![alicepk()]);
        assert_eq!(true, fpo_contract.pair_exists("ETH/USD".to_string()));

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500), vec![alicepk()]);
    }
}

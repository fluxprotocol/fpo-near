use crate::*;
use near_sdk::{
    log,
    serde::{Deserialize, Serialize},
    Timestamp,
};
// use ed25519_dalek::{PublicKey, Signature, Verifier, Signer};
use ed25519_dalek::Verifier;

use std::convert::TryFrom;
// maximum cost of storing a new entry in create_pair() - 170 * yocto per byte (1e19 as of 2022-04-14)
// #[allow(dead_code)]
pub const STORAGE_COST: u128 = 5_700_000_000_000_000_000_000; // was 1_700_000_000_000_000_000_000

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub struct PriceEntry {
    pub price: U128,            // Last reported price
    pub decimals: u16,          // Amount of decimals (e.g. if 2, 100 = 1.00)
    pub last_update: Timestamp, // Time of report
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Creates a new price pair by a provider
    #[payable]
    pub fn create_pair(&mut self, pair: String, decimals: u16, initial_price: U128) {
        let initial_storage_usage = env::storage_usage();
        let mut provider = self
            .providers
            .get(&env::signer_account_pk())
            .unwrap_or_else(Provider::new);

        let pair_name = format!("{}:{:?}", pair, env::signer_account_pk());
        assert!(
            provider.pairs.get(&pair_name).is_none(),
            "pair already exists"
        );
        provider.pairs.insert(
            &pair_name,
            &PriceEntry {
                price: initial_price,
                decimals,
                last_update: env::block_timestamp(),
            },
        );

        self.providers.insert(&env::signer_account_pk(), &provider);

        // check for storage deposit
        let storage_cost =
            env::storage_byte_cost() * u128::from(env::storage_usage() - initial_storage_usage);
        assert!(
            storage_cost <= env::attached_deposit(),
            "Insufficient storage, need {}",
            storage_cost
        );
    }

    /// Sets the price for a given price pair by a provider
    #[payable]
    pub fn push_data(&mut self, pair: String, price: U128) {
        let mut provider = self.get_provider_expect(&env::signer_account_pk());
        let pair_name = format!("{}:{:?}", pair, env::signer_account_pk());
        provider.set_price(pair_name, price, env::block_timestamp());
        self.providers.insert(&env::signer_account_pk(), &provider);
    }

    #[payable]
    pub fn push_data_signed(
        &mut self,
        signature: Vec<u8>,
        signer_pk: PublicKey,
        pair: String,
        price: String,
    ) {
        let message = format!("{}:{}", pair, price);
        let data: &[u8] = message.as_bytes();
        log!("data {:?}", data);
        let sig: ed25519_dalek::Signature = ed25519_dalek::Signature::try_from(signature.as_ref())
            .expect("Signature should be a valid array of 64 bytes [13, 254, 123, ...]");

        log!(
            "signer_pk.as_bytes().len() {:?}",
            signer_pk.as_bytes()[1..].len()
        );

        let public_key: ed25519_dalek::PublicKey =
            ed25519_dalek::PublicKey::from_bytes(&signer_pk.as_bytes()[1..]).unwrap();

        if let Ok(_) = public_key.verify(data, &sig) {
            log!("VERIFIES*********");
            // Should find a way to make sure that the signer's accId is the pk owner
            let mut provider = self.get_provider_expect(&signer_pk);
            let pair_name = format!("{}:{:?}", pair, signer_pk);
            provider.set_price(
                pair_name,
                U128::from(price.parse::<u128>().unwrap()),
                env::block_timestamp(),
            );

            // self.providers
            //     .insert(&env::predecessor_account_id(), &provider);
        }
    }

    /// Returns all data associated with a price pair by a provider
    pub fn get_entry(&self, pair: String, provider: PublicKey) -> Option<PriceEntry> {
        let pair_name = format!("{}:{:?}", pair, provider);
        let provider = self.get_provider_option(&provider);
        match provider {
            Some(provider) => provider.get_entry_option(&pair_name),
            None => None,
        }
    }

    /// Returns only the price of a price pair by a provider
    pub fn get_price(&self, pair: String, provider: &PublicKey) -> Option<U128> {
        let pair_name = format!("{}:{:?}", pair, provider);
        let provider = self.get_provider_option(provider);
        match provider {
            Some(provider) => provider
                .get_entry_option(&pair_name)
                .map(|entry| entry.price),
            None => None,
        }
    }

    /// Returns all the data associated with multiple price pairs by associated providers
    pub fn get_prices(&self, pairs: Vec<String>, providers: Vec<PublicKey>) -> Vec<Option<U128>> {
        assert_eq!(
            pairs.len(),
            providers.len(),
            "pairs and provider should be of equal length"
        );

        let mut result = vec![];
        for (i, provider) in providers.iter().enumerate() {
            let pair_name = format!("{}:{:?}", pairs[i], provider);
            result.push(
                self.get_provider_expect(provider)
                    .get_entry_option(&pair_name)
                    .map(|entry| entry.price),
            );
        }
        result
    }

    /// Checks if a given price pair exists
    pub fn pair_exists(&self, pair: String, provider: PublicKey) -> bool {
        let pair_name = format!("{}:{:?}", pair, provider);
        self.get_provider_expect(&provider)
            .pairs
            .get(&pair_name)
            .is_some()
    }
}

/// Price pair tests
#[cfg(test)]
mod tests {

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use super::*;

    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn bob() -> AccountId {
        "bob.near".parse().unwrap()
    }
    fn bobpk() -> PublicKey {
        PublicKey::from(
            "Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy"
                .parse()
                .unwrap(),
        )
    }
    fn alicepk() -> PublicKey {
        PublicKey::from(
            "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
                .parse()
                .unwrap(),
        )
    }

    fn get_context(
        predecessor_account_id: AccountId,
        // current_account_id: AccountId,
        signer_pk: PublicKey,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            // .current_account_id(current_account_id.clone())
            // .signer_account_id("robert.testnet".parse().unwrap())
            .predecessor_account_id(predecessor_account_id.clone())
            .signer_account_pk(signer_pk)
            .attached_deposit(STORAGE_COST);
        builder
    }

    // DOESNT PANIC!!!
    // #[should_panic]
    // #[test]
    // fn pair_name_too_long() {
    //     let context = get_context(alice(), alicepk());
    //     testing_env!(context.build());
    //     let mut fpo_contract = FPOContract::new();
    //     fpo_contract.create_pair(
    //         "1234567890123".to_string(),
    //         u16::max_value(),
    //         U128(u128::max_value()),
    //     );
    // }

    #[test]
    fn measure_storage_cost() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();

        let storage_used_before = env::storage_usage();
        fpo_contract.create_pair(
            "123456789012".to_string(),
            u16::max_value(),
            U128(u128::max_value()),
        );

        let storage_used_after = env::storage_usage();
        assert_eq!(storage_used_after - storage_used_before, 350); // was 170
    }

    #[test]
    fn create_pair() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::signer_account_pk())
        );
    }

    #[test]
    fn create_diff_pairs() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::signer_account_pk())
        );

        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(42000));
        assert_eq!(
            true,
            fpo_contract.pair_exists("BTC/USD".to_string(), env::signer_account_pk())
        );

        assert_eq!(
            vec![U128(2500), U128(42000)],
            fpo_contract
                .get_prices(
                    vec!["ETH/USD".to_string().to_string(), "BTC/USD".to_string()],
                    vec![env::signer_account_pk(), env::signer_account_pk()]
                )
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
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            true,
            fpo_contract.pair_exists("ETH/USD".to_string(), env::signer_account_pk())
        );

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
    }

    #[test]
    fn push_data() {
        let context = get_context(alice(), alicepk());
        testing_env!(context.build());
        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::signer_account_pk())
                .unwrap()
                .price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::signer_account_pk())
                .unwrap()
                .price
        );
    }

    #[test]
    fn push_data_multiple_providers() {
        let mut context = get_context(alice(), alicepk());
        testing_env!(context.build());

        let mut fpo_contract = FPOContract::new();
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::signer_account_pk())
                .unwrap()
                .price
        );

        // switch to bob as signer
        context = get_context(bob(), bobpk());
        testing_env!(context.build());

        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2700));
        assert_eq!(
            U128(2700),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::signer_account_pk())
                .unwrap()
                .price
        );
        assert_eq!(
            U128(2500),
            fpo_contract
                .get_entry("ETH/USD".to_string(), alicepk())
                .unwrap()
                .price
        );

        fpo_contract.push_data("ETH/USD".to_string(), U128(3000));

        assert_eq!(
            U128(3000),
            fpo_contract
                .get_entry("ETH/USD".to_string(), env::signer_account_pk())
                .unwrap()
                .price
        );
    }
}

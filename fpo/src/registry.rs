use crate::*;
use callbacks::{ext_price_consumer, GAS_TO_SEND_PRICE, ZERO_BALANCE};
use near_sdk::{Promise, Timestamp};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Registry {
    pub pairs: Vec<Vec<String>>,
    pub providers: Vec<Vec<PublicKey>>,
    pub min_last_update: Timestamp,
}

/// Private contract methods
impl FPOContract {
    /// Returns all the data associated with a provider wrapped in an Option
    pub fn get_registry_option(&self, account_id: &AccountId) -> Option<Registry> {
        self.registries.get(account_id)
    }
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Create a new registry, charging for storage
    #[payable]
    pub fn create_registry(
        &mut self,
        pairs: Vec<Vec<String>>,
        providers: Vec<Vec<PublicKey>>,
        min_last_update: Timestamp,
    ) {
        let initial_storage_usage = env::storage_usage();

        // insert the new registry associated with tx predecessor_account_id
        self.registries.insert(
            &env::predecessor_account_id(),
            &Registry {
                pairs,
                providers,
                min_last_update,
            },
        );

        // check for storage deposit
        let storage_cost =
            env::storage_byte_cost() * u128::from(env::storage_usage() - initial_storage_usage);
        assert!(
            storage_cost <= env::attached_deposit(),
            "Insufficient storage, need {}",
            storage_cost
        );
    }

    /// Calls `aggregate_median_many` using specified registry
    pub fn registry_aggregate(&self, registry_owner: AccountId) -> Vec<Option<U128>> {
        let registry = self.get_registry_option(&registry_owner);

        match registry {
            Some(registry) => self.aggregate_median_many(
                registry.pairs,
                registry.providers,
                registry.min_last_update,
            ),
            None => vec![None; 0],
        }
    }

    /// Calls `registry_aggregate` and forwards the result to the price consumer
    pub fn registry_aggregate_call(
        &self,
        registry_owner: AccountId,
        receiver_id: AccountId,
    ) -> Promise {
        let registry = self
            .get_registry_option(&registry_owner)
            .unwrap_or_else(|| {
                panic!("Registry not found for {}", registry_owner);
            });

        // get the first element of every subarray in `pairs` to submit as associated pair name
        let pairs = registry
            .pairs
            .iter()
            .map(|p| p.first().unwrap().clone())
            .collect::<Vec<String>>();

        let results = self.aggregate_median_many(
            registry.pairs.clone(),
            registry.providers.clone(),
            registry.min_last_update,
        );

        ext_price_consumer::on_registry_prices_received(
            env::predecessor_account_id(),
            pairs,
            results,
            registry_owner,
            receiver_id,
            ZERO_BALANCE,
            GAS_TO_SEND_PRICE,
        )
    }
}

/// Registry tests
#[cfg(test)]
mod tests {

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use price_pair::STORAGE_COST;

    const REGISTRY_COST: u128 = 5_810_000_000_000_000_000_000; // was 1_810_000_000_000_000_000_000

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
        account_id: AccountId,
        signer_pk: PublicKey,
        attached_deposit: u128,
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account_id.clone())
            .signer_account_id("robert.testnet".parse().unwrap())
            .predecessor_account_id(account_id.clone())
            .signer_account_pk(signer_pk)
            .attached_deposit(attached_deposit.clone());
        builder
    }

    #[test]
    fn create_registry() {
        // use alice
        let context = get_context(alice(), alicepk(), STORAGE_COST);
        testing_env!(context.build());

        // create fpo contract
        let mut fpo_contract = FPOContract::new(alice());

        // alice creates feeds for ETH/USD and BTC/USD
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(40000));

        // use bob
        let context = get_context(bob(), bobpk(), STORAGE_COST);
        testing_env!(context.build());

        // bob creates feeds for ETH/USD, BTC/USD, and NEAR/USD
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(3000));
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(30000));
        fpo_contract.create_pair("NEAR/USD".to_string(), 8, U128(10));

        // bob creates a registry using his and alice's feeds
        let context = get_context(bob(), bobpk(), REGISTRY_COST);
        testing_env!(context.build());
        fpo_contract.create_registry(
            vec![
                vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
                vec!["BTC/USD".to_string(), "BTC/USD".to_string()],
            ],
            vec![vec![alicepk(), bobpk()], vec![alicepk(), bobpk()]],
            0, // min_last_update
        );

        assert_eq!(
            vec![Some(U128(2750)), Some(U128(35000))],
            fpo_contract.registry_aggregate(bob())
        );
        // bob reorders elements in his registry and adds NEAR/USD
        fpo_contract.create_registry(
            vec![
                vec!["BTC/USD".to_string(), "BTC/USD".to_string()],
                vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
                vec!["NEAR/USD".to_string()],
            ],
            vec![
                vec![alicepk(), bobpk()],
                vec![alicepk(), bobpk()],
                vec![bobpk()],
            ],
            0, // min_last_update
        );

        assert_eq!(
            vec![Some(U128(35000)), Some(U128(2750)), Some(U128(10))],
            fpo_contract.registry_aggregate(bob())
        );
    }
}

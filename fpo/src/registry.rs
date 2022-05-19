use crate::*;
use callbacks::{ext_price_consumer, GAS_TO_SEND_PRICE, ZERO_BALANCE};
use near_sdk::{Promise, Timestamp};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Registry {
    pub pairs: Vec<String>,
    pub min_last_update: Timestamp,
}

// /// Private contract methods
// impl FPOContract {
//     /// Returns all the data associated with a provider wrapped in an Option
//     pub fn get_registry_option(&self, account_id: &AccountId) -> Option<Registry> {
//         self.registries.get(account_id)
//     }
// }

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    /// Create a new registry, charging for storage
    #[payable]
    pub fn create_registry(
        &mut self,
        pairs: Vec<String>,
        min_last_update: Timestamp,
    ) {
        let initial_storage_usage = env::storage_usage();

        // insert the new registry associated with tx predecessor_account_id
        self.registries.insert(
            &env::predecessor_account_id(),
            &Registry {
                pairs,
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

    pub fn registry_aggregate_median(&self, registry_owner: AccountId) -> Vec<Option<U128>> {
        let reg = self.registries.get(&registry_owner).expect("Registry not found");
        self.get_prices(reg.pairs)
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
        fpo_contract.create_pair("ETH/USD".to_string(), 8, U128(2500), vec![alicepk(), bobpk()]);
        fpo_contract.create_pair("BTC/USD".to_string(), 8, U128(40000), vec![alicepk(), bobpk()]);
        fpo_contract.create_pair("NEAR/USD".to_string(), 8, U128(10), vec![alicepk(), bobpk()]);


        // use bob
        let context = get_context(bob(), bobpk(), STORAGE_COST);
        testing_env!(context.build());


        // bob creates a registry using his and alice's feeds
        let context = get_context(bob(), bobpk(), REGISTRY_COST);
        testing_env!(context.build());
        fpo_contract.create_registry(
            vec![ "ETH/USD".to_string(), "BTC/USD".to_string()],
            0, // min_last_update
        );

        assert_eq!(
            vec![Some(U128(2500)), Some(U128(40000))],
            fpo_contract.registry_aggregate_median(bob())
        );
        // bob reorders elements in his registry and adds NEAR/USD
        fpo_contract.create_registry(
            vec![ "ETH/USD".to_string(), "BTC/USD".to_string(), "NEAR/USD".to_string()],
            0, // min_last_update
        );

        assert_eq!(
            vec![Some(U128(2500)), Some(U128(40000)), Some(U128(10))],
            fpo_contract.registry_aggregate_median(bob())
        );
    }
}
use consumer::ConsumerContract;
use near_fpo::FPOContractContract;
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::PublicKey;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use serde_json::json;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
    CONSUMER_BYTES => "../res/consumer.wasm"
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_COST: u128 = 5_700_000_000_000_000_000_000; // was 1_700_000_000_000_000_000_000
const REGISTRY_COST: u128 = 2_810_000_000_000_000_000_000; // was 1_810_000_000_000_000_000_000

fn init() -> (
    UserAccount,
    ContractAccount<FPOContractContract>,
    ContractAccount<ConsumerContract>,
) {
    let root = init_simulator(None);
    // Deploy the compiled Wasm bytes
    let fpo: ContractAccount<FPOContractContract> = deploy! {
        contract: FPOContractContract,
        contract_id: "nearfpo".to_string(),
        bytes: &FPO_BYTES,
        signer_account: root
    };
    // Deploy the compiled Wasm bytes
    let consumer: ContractAccount<ConsumerContract> = deploy! {
        contract: ConsumerContract,
        contract_id: "consumer",
        bytes: &CONSUMER_BYTES,
        signer_account: root
    };

    (root, fpo, consumer)
}

#[test]
fn simulate_registry_agg_call() {
    let (root, fpo, consumer) = init();
    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new(root.account_id())).assert_success();
    call!(root, consumer.new(fpo.account_id())).assert_success();

    let provider0 = root.create_user("provider0".parse().unwrap(), to_yocto("1000000"));
    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let provider0_pk: PublicKey = provider0.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

    // let admin create a price pair with signers, check if it exists, and get the value
    let tx = root
        .call(
            fpo.account_id(),
            "create_pair",
            &json!([
                "ETH/USD".to_string(),
                8,
                U128(2000),
                vec![
                    provider0_pk.clone(),
                    provider1_pk.clone(),
                    provider2_pk.clone()
                ]
            ])
            .to_string()
            .into_bytes(),
            DEFAULT_GAS,
            STORAGE_COST, // attached deposit
        )
        .assert_success();
    let tx = root
        .call(
            fpo.account_id(),
            "create_pair",
            &json!([
                "BTC/USD".to_string(),
                8,
                U128(45000),
                vec![
                    provider0_pk.clone(),
                    provider1_pk.clone(),
                    provider2_pk.clone()
                ]
            ])
            .to_string()
            .into_bytes(),
            DEFAULT_GAS,
            STORAGE_COST, // attached deposit
        )
        .assert_success();

    // create a price pair, check if it exists, and get the value

    // create registry for user
    let tx = user.call(
        fpo.account_id(),
        "create_registry",
        &json!([vec!["ETH/USD".to_string(), "BTC/USD".to_string()], 0])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        REGISTRY_COST, // attached deposit
    );

    let aggregated = call!(user, fpo.registry_aggregate_median(user.account_id()));

    println!(
        "Returned aggregated values from registry: {:?}",
        &aggregated.unwrap_json_value().to_owned()
    );

    debug_assert_eq!(
        &aggregated.unwrap_json_value().to_owned(),
        &json!([&"2000".to_string(), &"45000".to_string()])
    );

    call!(
        user,
        fpo.registry_aggregate_call(user.account_id(), consumer.account_id())
    )
    .assert_success();

    let res = call!(user, consumer.get_registry(user.account_id()));

    println!("registry result: {:?}", &res.unwrap_json_value().to_owned());

    debug_assert_eq!(
        &res.unwrap_json_value()["pairs"].to_owned(),
        &json!([&"ETH/USD".to_string(), &"BTC/USD".to_string()])
    );

    debug_assert_eq!(
        &res.unwrap_json_value()["results"].to_owned(),
        &json!([&"2000".to_string(), &"45000".to_string()])
    );
}

#[test]
fn simulate_get_price() {
    let (root, fpo, consumer) = init();
    // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
    call!(provider1, fpo.new(root.account_id())).assert_success();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    // let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider1_pk]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider1, fpo.pair_exists("ETH/USD".to_string())).assert_success();
    let price_entry = call!(provider1, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(provider2, consumer.new(fpo.account_id())).assert_success();

    let outcome = call!(provider2, consumer.get_price("ETH/USD".to_string()));
    match &outcome.promise_results()[2] {
        Some(res) => {
            assert_eq!(res.unwrap_json_value(), "2000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_prices() {
    let (root, fpo, consumer) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new(root.account_id())).assert_success();
    call!(root, consumer.new(fpo.account_id())).assert_success();

    // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider1_pk]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider1, fpo.pair_exists("ETH/USD".to_string())).assert_success();
    let price_entry = call!(provider1, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC/USD".to_string(), 8, U128(45000), vec![provider2_pk]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider2, fpo.pair_exists("BTC/USD".to_string())).assert_success();
    let price_entry = call!(provider2, fpo.get_entry("BTC/USD".to_string()));
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"45000".to_string()
    );

    let outcome = call!(
        provider2,
        consumer.get_prices(vec!["ETH/USD".to_string(), "BTC/USD".to_string()])
    );
    match &outcome.promise_results()[2] {
        Some(res) => {
            assert_eq!(res.unwrap_json_value(), json!(vec!["2000", "45000"]));
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_price_call() {
    let (root, fpo, consumer) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new(root.account_id())).assert_success();
    call!(root, consumer.new(fpo.account_id())).assert_success();
    // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider1_pk]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider1, fpo.pair_exists("ETH/USD".to_string())).assert_success();
    let price_entry = call!(provider1, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(
        user,
        fpo.get_price_call("ETH/USD".to_string(), consumer.account_id())
    );

    let fetched_entry = call!(user, consumer.get_pair("ETH/USD".to_string()));

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            assert_eq!(res.unwrap_json_value()["price"], "2000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_prices_call() {
    let (root, fpo, consumer) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new(root.account_id())).assert_success();
    call!(root, consumer.new(fpo.account_id())).assert_success();

    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!([
            "ETH/USD".to_string(),
            8,
            U128(2000),
            vec![provider1_pk.clone()]
        ])
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider1, fpo.pair_exists("ETH/USD".to_string())).assert_success();
    let price_entry = call!(provider1, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC/USD".to_string(), 8, U128(45000), vec![provider1_pk]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(provider2, fpo.pair_exists("BTC/USD".to_string())).assert_success();
    let price_entry = call!(provider2, fpo.get_entry("BTC/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"45000".to_string()
    );

    call!(
        user,
        fpo.get_prices_call(
            vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
            consumer.account_id()
        )
    );

    let fetched_entry = call!(user, consumer.get_pair("ETH/USD".to_string()));

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "2000");
        }
        None => println!("Retrieved Nothing"),
    }

    let fetched_entry = call!(user, consumer.get_pair("BTC/USD".to_string()));

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            assert_eq!(res.unwrap_json_value()["price"], "45000");
        }
        None => println!("Retrieved Nothing"),
    }
}

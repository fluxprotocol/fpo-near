use near_fpo::FPOContractContract;
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_COST: u128 = 1_700_000_000_000_000_000_000;

fn init() -> (UserAccount, ContractAccount<FPOContractContract>) {
    let root = init_simulator(None);

    // Deploy the compiled Wasm bytes
    let fpo: ContractAccount<FPOContractContract> = deploy!(
        contract: FPOContractContract,
        contract_id: "nearfpo".to_string(),
        bytes: &FPO_BYTES,
        signer_account: root
    );

    (root, fpo)
}

#[test]
fn simulate_create_pair() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    )
    .assert_success();
    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );
}

#[test]
fn simulate_create_smae_pair() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    let err = call!(root, fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))).promise_errors();
    println!("ERROR: {:?}", err);
}

#[test]
fn simulate_push_data() {
    let (root, fpo) = init();

    call!(root, fpo.new()).assert_success();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    )
    .assert_success();
    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );

    // update the data
    call!(root, fpo.push_data("ETH/USD".to_string(), U128(4000))).assert_success();

    // get the updated data
    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );
}

#[test]
fn simulate_different_providers() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    )
    .assert_success();

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        bob,
        fpo.pair_exists("ETH/USD".to_string(), bob.account_id())
    )
    .assert_success();

    // output and check bob's data
    let price_entry = call!(bob, fpo.get_entry("ETH/USD".to_string(), bob.account_id()));
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    // output and check root's data
    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );
}

#[test]
fn simulate_different_pairs() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH / USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        bob,
        fpo.pair_exists("ETH / USD".to_string(), bob.account_id())
    )
    .assert_success();

    // create another price pair from bob
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC / USD".to_string(), 8, U128(45000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        bob,
        fpo.pair_exists("BTC / USD".to_string(), bob.account_id())
    )
    .assert_success();

    // output and check bob's data
    let price_entry = call!(
        bob,
        fpo.get_entry("ETH / USD".to_string(), bob.account_id())
    );
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    // output and check bob's data
    let price_entry = call!(
        bob,
        fpo.get_entry("BTC / USD".to_string(), bob.account_id())
    );
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"45000".to_string()
    );
}

#[test]
fn simulate_agg_avg() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from alice
    let alice = root.create_user("alice".parse().unwrap(), to_yocto("1000000"));
    alice.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(3000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from carol
    let carol = root.create_user("carol".parse().unwrap(), to_yocto("1000000"));
    carol.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(3000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // find the average of the four
    let pairs = vec![
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
    ];
    let avg = call!(
        bob,
        fpo.aggregate_avg(
            pairs,
            vec![
                root.account_id(),
                bob.account_id(),
                alice.account_id(),
                carol.account_id()
            ],
            0
        )
    );

    // output and check the data
    println!("Returned AVG: {:?}", &avg.unwrap_json_value());
    debug_assert_eq!(&avg.unwrap_json_value(), &"2500".to_string());
}

#[test]
fn simulate_agg_median() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from alice
    let alice = root.create_user("alice".parse().unwrap(), to_yocto("1000000"));
    alice.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from carol
    let carol = root.create_user("carol".parse().unwrap(), to_yocto("1000000"));
    carol.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // find the median of the four
    let pairs = vec![
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
    ];
    let median = call!(
        bob,
        fpo.aggregate_median(
            pairs,
            vec![
                root.account_id(),
                bob.account_id(),
                alice.account_id(),
                carol.account_id()
            ],
            0
        )
    );

    // output and check the data
    println!("Returned MEDIAN: {:?}", &median.unwrap_json_value());
    debug_assert_eq!(&median.unwrap_json_value(), &"3000".to_string());
}

#[test]
fn simulate_agg_median_diff_ids() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create a price pair from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH-USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH / USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from alice
    let alice = root.create_user("alice".parse().unwrap(), to_yocto("1000000"));
    alice.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(4000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // create a price pair from carol
    let carol = root.create_user("carol".parse().unwrap(), to_yocto("1000000"));
    carol.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // find the median of the four
    let pairs = vec![
        "ETH-USD".to_string(),
        "ETH / USD".to_string(),
        "ETH/USD".to_string(),
        "ETH/USD".to_string(),
    ];
    let median = call!(
        bob,
        fpo.aggregate_median(
            pairs,
            vec![
                root.account_id(),
                bob.account_id(),
                alice.account_id(),
                carol.account_id()
            ],
            0
        )
    );

    // output and check the data
    println!("Returned MEDIAN: {:?}", &median.unwrap_json_value());
    debug_assert_eq!(&median.unwrap_json_value(), &"3000".to_string());
}

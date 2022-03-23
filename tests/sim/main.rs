
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64};
use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{call, view, deploy, init_simulator, ContractAccount, UserAccount, to_yocto};
use near_fpo::FPOContractContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "target/wasm32-unknown-unknown/release/near_fpo.wasm",
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

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

    call!(
        root,
        fpo.new()
    ).assert_success();

    call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    ).assert_success();


    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    ).assert_success();


    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );

    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"]);
    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"2000".to_string());


}

#[test]
fn simulate_push_data() {
    let (root, fpo) = init();

    call!(
        root,
        fpo.new()
    ).assert_success();

    call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    ).assert_success();


    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    ).assert_success();


    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"].to_owned());

    call!(
        root,
        fpo.push_data("ETH/USD".to_string(),  U128(4000))
    ).assert_success();


    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"]);

    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"4000".to_string());

}


#[test]
fn simulate_different_providers() {
    let (root, fpo) = init();

    call!(
        root,
        fpo.new()
    ).assert_success();

    call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    ).assert_success();


    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    ).assert_success();


    println!("ROOT {:?}", root);


    let bob = root.create_user("bob".to_string(), to_yocto("1000000"));    
    println!("BOB {:?}", bob);

    call!(
        bob,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    ).assert_success();


    call!(
        bob,
        fpo.pair_exists("ETH/USD".to_string(), bob.account_id())
    ).assert_success();



    let price_entry = call!(
        bob,
        fpo.get_entry("ETH/USD".to_string(), bob.account_id())
    );
    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"].to_owned());
    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"4000".to_string());

    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"].to_owned());
    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"2000".to_string());


}


#[test]
fn simulate_agg_avg() {
    let (root, fpo) = init();

    call!(
        root,
        fpo.new()
    ).assert_success();

    call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    ).assert_success();


   

    let bob = root.create_user("bob".to_string(), to_yocto("1000000"));    
    println!("BOB {:?}", bob);

    call!(
        bob,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    ).assert_success();


    let pairs = vec!["ETH/USD".to_string(), "ETH/USD".to_string()];
    let avg = call!(
        bob,
        fpo.aggregate_avg(pairs, vec![root.account_id(), bob.account_id()], U64(0))
    );
    println!("Returned AVG: {:?}", &avg.unwrap_json_value());
    debug_assert_eq!(&avg.unwrap_json_value(), &"3000".to_string());


}


#[test]
fn simulate_agg_median() {
    let (root, fpo) = init();

    call!(
        root,
        fpo.new()
    ).assert_success();

    call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    ).assert_success();


   

    let bob = root.create_user("bob".to_string(), to_yocto("1000000"));    
    println!("BOB {:?}", bob);

    call!(
        bob,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    ).assert_success();


    let pairs = vec!["ETH/USD".to_string(), "ETH/USD".to_string()];
    let avg = call!(
        bob,
        fpo.aggregate_median(pairs, vec![root.account_id(), bob.account_id()], U64(0))
    );
    println!("Returned MEDIAN: {:?}", &avg.unwrap_json_value());
    debug_assert_eq!(&avg.unwrap_json_value(), &"3000".to_string());


}
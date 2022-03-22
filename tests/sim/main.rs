pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64};
use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{call, view, deploy, init_simulator, ContractAccount, UserAccount};
use near_fpo::FPOContractContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "/home/mnaga/flux/fpo-near/target/wasm32-unknown-unknown/release/near_fpo.wasm",
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
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2500))
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
    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"2500".to_string());


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
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2500))
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
        fpo.push_data("ETH/USD".to_string(),  U128(3000))
    ).assert_success();


    let price_entry = call!(
        root,
        fpo.get_entry("ETH/USD".to_string(), root.account_id())
    );
    println!("Returned Price: {:?}", &price_entry.unwrap_json_value()["price"]);

    debug_assert_eq!(&price_entry.unwrap_json_value()["price"].to_owned(), &"3000".to_string());

}
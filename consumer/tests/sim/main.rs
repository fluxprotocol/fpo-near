// use consumer::ConsumerContract;
// use near_fpo::FPOContractContract;
// pub use near_sdk::json_types::Base64VecU8;
// use near_sdk::json_types::U128;
// use near_sdk::PublicKey;
// use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
// use serde_json::json;

// near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
//     FPO_BYTES => "../res/near_fpo.wasm",
//     CONSUMER_BYTES => "../res/consumer.wasm"
// }

// pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
// pub const STORAGE_COST: u128 = 5_700_000_000_000_000_000_000; // was 1_700_000_000_000_000_000_000
//                                                               // const REGISTRY_COST: u128 = 2_810_000_000_000_000_000_000; // was 1_810_000_000_000_000_000_000

// fn init() -> (
//     UserAccount,
//     ContractAccount<FPOContractContract>,
//     ContractAccount<ConsumerContract>,
// ) {
//     let root = init_simulator(None);
//     // Deploy the compiled Wasm bytes
//     let fpo: ContractAccount<FPOContractContract> = deploy! {
//         contract: FPOContractContract,
//         contract_id: "nearfpo".to_string(),
//         bytes: &FPO_BYTES,
//         signer_account: root
//     };
//     // Deploy the compiled Wasm bytes
//     let consumer: ContractAccount<ConsumerContract> = deploy! {
//         contract: ConsumerContract,
//         contract_id: "consumer",
//         bytes: &CONSUMER_BYTES,
//         signer_account: root
//     };

//     (root, fpo, consumer)
// }

// #[test]
// fn simulate_get_price() {
//     let (root, fpo, consumer) = init();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
//     call!(provider1, fpo.new("admin".parse().unwrap())).assert_success();
//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     // let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     call!(provider2, consumer.new(fpo.account_id())).assert_success();

//     let outcome = call!(
//         provider2,
//         consumer.get_price("ETH/USD".to_string(), provider1_pk.clone())
//     );
//     match &outcome.promise_results()[2] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value(), "2000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_get_prices() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();

//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();
//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH/USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH/USD".to_string(), provider2_pk.clone())
//     );
//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     let outcome = call!(
//         provider2,
//         consumer.get_prices(
//             vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()]
//         )
//     );
//     match &outcome.promise_results()[2] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value(), json!(vec!["2000", "4000"]));
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_agg_avg() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();

//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();
//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH/USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH/USD".to_string(), provider2_pk.clone())
//     );
//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     let outcome = call!(
//         provider2,
//         consumer.aggregate_avg(
//             vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             0
//         )
//     );
//     match &outcome.promise_results()[2] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value(), "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_agg_median() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();

//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH/USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH/USD".to_string(), provider2_pk.clone())
//     );
//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     let outcome = call!(
//         provider2,
//         consumer.aggregate_median(
//             vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             0
//         )
//     );
//     match &outcome.promise_results()[2] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value(), "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_get_price_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     call!(
//         user,
//         fpo.get_price_call(
//             "ETH/USD".to_string(),
//             provider1_pk.clone(),
//             consumer.account_id()
//         )
//     );

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider1_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "2000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_get_prices_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     // create a price pair, check if it exists, and get the value
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["BTC/USD".to_string(), 8, U128(45000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("BTC/USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("BTC/USD".to_string(), provider2_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"45000".to_string()
//     );

//     call!(
//         user,
//         fpo.get_prices_call(
//             vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             consumer.account_id()
//         )
//     );

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider1_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             // println!("Retrieved Value: {:?}", res.unwrap_json_value());
//             assert_eq!(res.unwrap_json_value()["price"], "2000");
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider2_pk.clone(), "BTC/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "45000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_get_prices_call2() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     // create a price pair, check if it exists, and get the value
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH/USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH/USD".to_string(), provider2_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     call!(
//         user,
//         fpo.get_prices_call(
//             vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             consumer.account_id()
//         )
//     );

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider1_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "2000"); // why wrong??
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider2_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "4000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_aggregate_avg_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     // create a price pair, check if it exists, and get the value
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH / USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH / USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH / USD".to_string(), provider2_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     call!(
//         user,
//         fpo.aggregate_avg_call(
//             vec!["ETH/USD".to_string(), "ETH / USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             0,
//             consumer.account_id()
//         )
//     );

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider1_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider2_pk.clone(), "ETH / USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_aggregate_median_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create a price pair, check if it exists, and get the value
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider1,
//         fpo.pair_exists("ETH/USD".to_string(), provider1_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider1,
//         fpo.get_entry("ETH/USD".to_string(), provider1_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"2000".to_string()
//     );

//     // create a price pair, check if it exists, and get the value
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH / USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     call!(
//         provider2,
//         fpo.pair_exists("ETH / USD".to_string(), provider2_pk.clone())
//     )
//     .assert_success();
//     let price_entry = call!(
//         provider2,
//         fpo.get_entry("ETH / USD".to_string(), provider2_pk.clone())
//     );

//     debug_assert_eq!(
//         &price_entry.unwrap_json_value()["price"].to_owned(),
//         &"4000".to_string()
//     );

//     call!(
//         user,
//         fpo.aggregate_median_call(
//             vec!["ETH/USD".to_string(), "ETH / USD".to_string()],
//             vec![provider1_pk.clone(), provider2_pk.clone()],
//             0,
//             consumer.account_id()
//         )
//     );

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider1_pk.clone(), "ETH/USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     let fetched_entry = call!(
//         user,
//         consumer.get_pair(provider2_pk.clone(), "ETH / USD".to_string())
//     );

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(res.unwrap_json_value()["price"], "3000");
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

// #[test]
// fn simulate_aggregate_median_many_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create eth/usd and btc/usd from provider1
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["BTC/USD".to_string(), 8, U128(30000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );

//     // create eth/usd and btc/usd from provider2
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["BTC/USD".to_string(), 8, U128(40000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );

//     let pairs_eth = vec!["ETH/USD".to_string(), "ETH/USD".to_string()];
//     let pairs_btc = vec!["BTC/USD".to_string(), "BTC/USD".to_string()];
//     let providers = vec![provider1_pk.clone(), provider2_pk.clone()];

//     call!(
//         user,
//         fpo.aggregate_median_many_call(
//             vec![pairs_eth, pairs_btc],
//             vec![providers.clone(), providers],
//             0,
//             consumer.account_id()
//         )
//     );
// }

// #[test]
// fn simulate_registry_aggregate_call() {
//     let (root, fpo, consumer) = init();

//     let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
//     let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

//     let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

//     call!(root, fpo.new("admin".parse().unwrap())).assert_success();
//     call!(root, consumer.new(fpo.account_id())).assert_success();
//     // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

//     let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
//     let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

//     // create eth/usd and btc/usd from provider1
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(2000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     provider1.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["BTC/USD".to_string(), 8, U128(30000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );

//     // create eth/usd and btc/usd from provider2
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["ETH/USD".to_string(), 8, U128(4000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );
//     provider2.call(
//         fpo.account_id(),
//         "create_pair",
//         &json!(["BTC/USD".to_string(), 8, U128(40000)])
//             .to_string()
//             .into_bytes(),
//         DEFAULT_GAS,
//         STORAGE_COST, // attached deposit
//     );

//     let pairs_eth = vec!["ETH/USD".to_string(), "ETH/USD".to_string()];
//     let pairs_btc = vec!["BTC/USD".to_string(), "BTC/USD".to_string()];
//     let providers = vec![provider1_pk.clone(), provider2_pk.clone()];

//     // create a registry for provider1
//     provider1
//         .call(
//             fpo.account_id(),
//             "create_registry",
//             &json!([
//                 vec![pairs_eth.clone(), pairs_btc.clone(),],
//                 vec![providers.clone(), providers.clone()],
//                 0
//             ])
//             .to_string()
//             .into_bytes(),
//             DEFAULT_GAS,
//             STORAGE_COST, // attached deposit
//         )
//         .assert_success();

//     // aggregate provider1 registry prices and forward to consumer contract
//     let tx = call!(
//         user,
//         fpo.registry_aggregate_call(provider1.account_id(), consumer.account_id())
//     );
//     println!("{:?}", tx);

//     // fetch provider1 registry prices from consumer
//     let fetched_entry = call!(user, consumer.get_registry(provider1.account_id()));
//     println!("{:?}", fetched_entry);

//     // check forwarded prices
//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(
//                 res.unwrap_json_value()["results"],
//                 json!([&"3000".to_string(), &"35000".to_string()])
//             )
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     // update provider2 ETH/USDfeed
//     call!(provider2, fpo.push_data("ETH/USD".to_string(), U128(6000))).assert_success();

//     // aggregate and push answers to consumer again
//     let tx = call!(
//         user,
//         fpo.registry_aggregate_call(provider1.account_id(), consumer.account_id())
//     );
//     println!("{:?}", tx);

//     // fetch provider1 registry prices from consumer
//     let fetched_entry = call!(user, consumer.get_registry(provider1.account_id()));
//     println!("{:?}", fetched_entry);

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(
//                 res.unwrap_json_value()["results"],
//                 json!([&"4000".to_string(), &"35000".to_string()])
//             )
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     // create a registry for provider2
//     provider2
//         .call(
//             fpo.account_id(),
//             "create_registry",
//             &json!([
//                 vec![pairs_eth, pairs_btc,],
//                 vec![providers.clone(), providers.clone()],
//                 0
//             ])
//             .to_string()
//             .into_bytes(),
//             DEFAULT_GAS,
//             STORAGE_COST, // attached deposit
//         )
//         .assert_success();

//     // aggregate prov2 registry prices and forward to consumer contract
//     let tx = call!(
//         user,
//         fpo.registry_aggregate_call(provider2.account_id(), consumer.account_id())
//     );
//     println!("{:?}", tx);

//     // fetch provider2 registry prices from consumer
//     let fetched_entry = call!(user, consumer.get_registry(provider2.account_id()));
//     println!("{:?}", fetched_entry);

//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(
//                 res.unwrap_json_value()["results"],
//                 json!([&"4000".to_string(), &"35000".to_string()])
//             )
//         }
//         None => println!("Retrieved Nothing"),
//     }

//     // update provider2 ETH/USDfeed
//     call!(provider2, fpo.push_data("ETH/USD".to_string(), U128(7000))).assert_success();

//     // aggregate and push answers to consumer again
//     let tx = call!(
//         user,
//         fpo.registry_aggregate_call(provider2.account_id(), consumer.account_id())
//     );
//     println!("{:?}", tx);

//     let fetched_entry = call!(user, consumer.get_registry(provider2.account_id()));
//     println!("{:?}", fetched_entry);
//     // check forwarded prices
//     match &fetched_entry.promise_results()[1] {
//         Some(res) => {
//             assert_eq!(
//                 res.unwrap_json_value()["results"],
//                 json!([&"4500".to_string(), &"35000".to_string()])
//             )
//         }
//         None => println!("Retrieved Nothing"),
//     }
// }

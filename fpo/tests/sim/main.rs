use near_fpo::FPOContractContract;
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{log, PublicKey};
use near_sdk_sim::borsh::BorshSerialize;
use near_sdk_sim::near_crypto::Signer;
use near_sdk_sim::to_yocto;
use near_sdk_sim::{call, deploy, init_simulator, ContractAccount, UserAccount};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_COST: u128 = 5_700_000_000_000_000_000_000; // was 1_700_000_000_000_000_000_000
const REGISTRY_COST: u128 = 2_810_000_000_000_000_000_000; // was 1_810_000_000_000_000_000_000

fn init() -> (UserAccount, ContractAccount<FPOContractContract>) {
    let root: UserAccount = init_simulator(None);
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
fn simulate_creating_registries() {
    let (root, fpo) = init();

    call!(root, fpo.new(root.account_id())).assert_success();

    let provider0 = root.create_user("provider0".parse().unwrap(), to_yocto("1000000"));
    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let provider0_pk: PublicKey = provider0.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();

   

    // let admin create a price pair with signers, check if it exists, and get the value
    let tx = root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    ).assert_success();
    let tx = root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC/USD".to_string(), 8, U128(45000), vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    ).assert_success();

    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64().expect("Couldn't fetch round_id");

    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(2000));
    let data: &[u8] = message.as_bytes();
    let p1_sig = provider1.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(3000));
    let data: &[u8] = message.as_bytes();
    let p2_sig = provider2.signer.sign(data);

  

    let p0_sig_vec = p0_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p1_sig_vec = p1_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p2_sig_vec = p2_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");



    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));


    // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
    // let bob update root's feed
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    ).assert_success();


    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );
    // create registry for bob
    let tx = bob.call(
        fpo.account_id(),
        "create_registry",
        &json!([
                vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
                0
        ])
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        REGISTRY_COST, // attached deposit
    );

    let aggregated = call!(
        bob,
        fpo.registry_aggregate_median(
            bob.account_id()
        )
    );

    println!(
        "Returned aggregated values from registry: {:?}",
        &aggregated.unwrap_json_value().to_owned()
    );

    debug_assert_eq!(
        &aggregated.unwrap_json_value().to_owned(),
        &json!([&"2000".to_string(), &"45000".to_string()])
    );


}

#[test]
fn simulate_add_rm_signer() {
    let (root, fpo) = init();

    call!(root, fpo.new(root.account_id())).assert_success();

    let provider0 = root.create_user("provider0".parse().unwrap(), to_yocto("1000000"));
    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
    let provider3 = root.create_user("provider3".parse().unwrap(), to_yocto("1000000"));

    let provider0_pk: PublicKey = provider0.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();
    let provider3_pk: PublicKey = provider3.signer.public_key.to_string().parse().unwrap();



    // let admin create a price pair with signers, check if it exists, and get the value
    let tx = root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    ).assert_success();
    log!("**{:?}", tx);
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string())
    )
    .assert_success();

    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64().expect("Couldn't fetch round_id");

    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(2000));
    let data: &[u8] = message.as_bytes();
    let p1_sig = provider1.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(3000));
    let data: &[u8] = message.as_bytes();
    let p2_sig = provider2.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(4000));
    let data: &[u8] = message.as_bytes();
    let p3_sig = provider3.signer.sign(data);


    let p0_sig_vec = p0_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p1_sig_vec = p1_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p2_sig_vec = p2_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p3_sig_vec = p3_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");


    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    // try pushing data with invalid signer provider3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec(), p3_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone(), provider3_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );
    let tx = call!(
        root,
        fpo.add_signers(
            vec![provider3_pk.clone()], "ETH/USD".to_string()
        )
    ).assert_success();

    println!("----tx {:?}", tx);
    // push data after adding provider3 as signer 
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec(), p3_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone(), provider3_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id
        )
    ).assert_success();
    println!("----tx {:?}", tx);


    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2500".to_string()
    );


    let tx = call!(
        root,
        fpo.rm_signers(
            vec![provider3_pk.clone()], "ETH/USD".to_string()
        )
    ).assert_success();

    println!("----tx {:?}", tx);

    // try pushing data with invalid signer provider3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec(), p3_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone(), provider3_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id + 1
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );


}

#[test]
fn simulate_push_data_signed() {
    let (root, fpo) = init();

    call!(root, fpo.new(root.account_id())).assert_success();

    let provider0 = root.create_user("provider0".parse().unwrap(), to_yocto("1000000"));
    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
    let provider3 = root.create_user("provider3".parse().unwrap(), to_yocto("1000000"));

    let provider0_pk: PublicKey = provider0.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();
    let provider2_pk: PublicKey = provider2.signer.public_key.to_string().parse().unwrap();
    let provider3_pk: PublicKey = provider3.signer.public_key.to_string().parse().unwrap();

    // let non-admin creat pair
    let tx = provider1.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider1_pk.clone(), provider2_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
   // assert error
   assert!(!tx.is_ok());
   println!("----tx {:?}", tx.status());

    // let admin create a price pair with signers, check if it exists, and get the value
    let tx = root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    ).assert_success();
    log!("**{:?}", tx);
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string())
    )
    .assert_success();

    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );

    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"].to_owned()
    );
    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64()
    );
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64().expect("Couldn't fetch round_id");
    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id - 1, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_invalid_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(2000));
    let data: &[u8] = message.as_bytes();
    let p1_sig = provider1.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(3000));
    let data: &[u8] = message.as_bytes();
    let p2_sig = provider2.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(4000));
    let data: &[u8] = message.as_bytes();
    let p3_sig = provider3.signer.sign(data);


    let p0_sig_vec = p0_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p0_invalid_sig_vec = p0_invalid_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p1_sig_vec = p1_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p2_sig_vec = p2_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p3_sig_vec = p3_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");


    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    // try pushing data with invalid signer provider3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec(), p3_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone(), provider3_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id

        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );


    // try pushing data with invalid signature (Wrong round_id for provider0)
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_invalid_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );

    // try pushing data with duplicate signature
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );

    // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
    // let bob update root's feed
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );

    println!("----tx {:?}", tx);

    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64()
    );

    // try pushing data with wrong round_id
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(), p2_sig_vec[1..].to_vec()],
            vec![provider0_pk.clone(), provider1_pk.clone(), provider2_pk.clone()],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id // 1
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status() );
}

#[test]
fn simulate_create_pair() {
    let (root, fpo) = init();
    call!(root, fpo.new(root.account_id())).assert_success();
    let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    let tx = root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![root_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    println!("----tx {:?}", tx);

    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string())
    )
    .assert_success();
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

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
fn simulate_create_same_pair() {
    let (root, fpo) = init();
    call!(root, fpo.new(root.account_id())).assert_success();
    let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    // create a price pair
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![root_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    let tx = call!(root, fpo.create_pair("ETH/USD".to_string(), 8, U128(2000), vec![root_pk.clone()]));
    println!("tx: {:?}", tx);
    assert!(!tx.is_ok())
}

#[test]
fn simulate_push_data() {
    let (root, fpo) = init();

    call!(root, fpo.new(root.account_id())).assert_success();
    let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    // create a price pair, check if it exists, and get the value
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![root_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string())
    )
    .assert_success();
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64()
    );
    let round = price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64().expect("Couldn't fetch round_id");

    // update the data
    call!(root, fpo.push_data("ETH/USD".to_string(), U128(4000), round)).assert_success();

    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

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
    call!(root, fpo.new(root.account_id())).assert_success();



    let provider0 = root.create_user("provider0".parse().unwrap(), to_yocto("1000000"));
    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));

    let provider0_pk: PublicKey = provider0.signer.public_key.to_string().parse().unwrap();
    let provider1_pk: PublicKey = provider1.signer.public_key.to_string().parse().unwrap();

    // create a price pair from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2000), vec![provider0_pk.clone(), provider1_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        root,
        fpo.pair_exists("ETH/USD".to_string())
    )
    .assert_success();
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    let round = price_entry.unwrap_json_value()["latest_round_id"].to_owned().as_u64().expect("Couldn't fetch round_id");


    // update the data
    call!(provider0, fpo.push_data("ETH/USD".to_string(), U128(4000), round)).assert_success();

    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );


    // update the data
    call!(provider1, fpo.push_data("ETH/USD".to_string(), U128(5000), round + 1)).assert_success();

    // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"5000".to_string()
    );
  
}

#[test]
fn simulate_different_pairs() {
    let (root, fpo) = init();
    call!(root, fpo.new(root.account_id())).assert_success();
    // let root_pk: PublicKey = root.signer.public_key.to_string().parse().unwrap();

    // create a price pair from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    let bob_pk: PublicKey = bob.signer.public_key.to_string().parse().unwrap();

    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH / USD".to_string(), 8, U128(4000), vec![bob_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        bob,
        fpo.pair_exists("ETH / USD".to_string())
    )
    .assert_success();

    // create another price pair from bob
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC / USD".to_string(), 8, U128(45000), vec![bob_pk.clone()]])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    call!(
        bob,
        fpo.pair_exists("BTC / USD".to_string())
    )
    .assert_success();

    // output and check bob's data
    let price_entry = call!(bob, fpo.get_entry("ETH / USD".to_string()));
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    // output and check bob's data
    let price_entry = call!(bob, fpo.get_entry("BTC / USD".to_string()));
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"].to_owned()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"45000".to_string()
    );
}


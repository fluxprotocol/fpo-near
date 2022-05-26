use std::collections::HashSet;

use near_fpo::FPOContractContract;
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{env, log, AccountId, PublicKey};
use near_sdk_sim::borsh::BorshSerialize;
use near_sdk_sim::lazy_static_include::syn::Signature;
use near_sdk_sim::near_crypto::Signer;
use near_sdk_sim::to_yocto;
use near_sdk_sim::{call, deploy, init_simulator, ContractAccount, UserAccount};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_COST: u128 = 8_700_000_000_000_000_000_000; // was 1/5_700_000_000_000_000_000_000
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
fn simulate_duplicate_sig() {
    let (root, fpo) = init();

    call!(root, fpo.new(root.account_id())).assert_success();

    let mut providers: Vec<UserAccount> = Vec::new();
    let mut providers_pks: Vec<PublicKey> = Vec::new();
    let providers_count = 20;
    for i in 0..providers_count {
        let prov = format!("provider{:?}", i);
        let acc = prov.as_str().parse().unwrap();
        let provider = root.create_user(acc, to_yocto("1000000"));
        let pk: PublicKey = provider.signer.public_key.to_string().parse().unwrap();
        providers.push(provider);
        providers_pks.push(pk);
    }

    // let admin create a price pair with signers, check if it exists, and get the value
    let tx = root
        .call(
            fpo.account_id(),
            "create_pair",
            &json!(["ETH/USD".to_string(), 8, U128(2000), providers_pks])
                .to_string()
                .into_bytes(),
            DEFAULT_GAS,
            STORAGE_COST, // attached deposit
        )
        .assert_success();
    log!("**{:?}", tx);
    call!(root, fpo.pair_exists("ETH/USD".to_string())).assert_success();

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
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");

    let mut p_sigs = Vec::new();
    let mut p_sigs_vecs: Vec<Vec<u8>> = Vec::new();
    for i in 0..providers_count {
        let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(2000));
        let data: &[u8] = message.as_bytes();
        let p_sig = providers[i].signer.sign(data);
        let p_sig_vec = p_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
        p_sigs.push(p_sig);
        p_sigs_vecs.push(p_sig_vec[1..].to_vec());
    }

    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    // try pushing data with duplicate sig
    let gas1 = env::used_gas();
    let tx = call!(
        bob,
        fpo.push_data_signed(
            p_sigs_vecs,
            providers_pks,
            "ETH/USD".to_string(),
            vec![U128(2000); providers_count],
            round_id
        )
    );

    let gas2 = env::used_gas();
    log!("GAS USED = {:?}", gas2 - gas1);
    log!("Gas BURNT =  {:?}", tx.gas_burnt()); // 202441142077742, 202432674595684
                                               // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());
}

#[test]
fn simulate_setting_min_signers() {
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
    );
    log!("**GAS BURNT IN CREATING PAIR {:?}", tx.gas_burnt()); //hash: 3503211876329, vec: 3470139615479
    call!(root, fpo.pair_exists("ETH/USD".to_string())).assert_success();

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
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");
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

    // let bob update root's feed with less than 2 signatures
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(),],
            vec![provider0_pk.clone(),],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000)],
            round_id
        )
    );

    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

    // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
    // let bob update root's feed with enough signatures (2 by default)
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(),],
            vec![provider0_pk.clone(), provider1_pk.clone(),],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000)],
            round_id
        )
    );

    println!("----GAS BURNT IN PUSH DATA SIGNED {:?}", tx.gas_burnt()); // hash: 62933528439786, vec: 62844473241129

    // // get the updated data
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()["price"]
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"1500".to_string()
    );

    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );

    let tx = call!(root, fpo.set_min_signers(3, "ETH/USD".to_string()));
    log!("++++++BURNT GAS IN set_min_signers: {:? }", tx.gas_burnt()); //hash: 3372792631547, vec: 3315259409444
                                                                       // // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
                                                                       // let bob update root's feed with 2 signatures after setting it to 3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![p0_sig_vec[1..].to_vec(), p1_sig_vec[1..].to_vec(),],
            vec![provider0_pk.clone(), provider1_pk.clone(),],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000)],
            round_id + 1
        )
    );
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id + 1, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id + 1, U128(2000));
    let data: &[u8] = message.as_bytes();
    let p1_sig = provider1.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id + 1, U128(3000));
    let data: &[u8] = message.as_bytes();
    let p2_sig = provider2.signer.sign(data);

    let p0_sig_vec = p0_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p1_sig_vec = p1_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p2_sig_vec = p2_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    // let bob update feed with 3 signatures
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec(),
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone(),
            ],
            "ETH/USD".to_string(),
            vec![U128(1000), U128(2000), U128(3000)],
            round_id + 1
        )
    )
    .assert_success();
    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );
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

    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");

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
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    )
    .assert_success();

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
        &json!([vec!["ETH/USD".to_string(), "BTC/USD".to_string()], 0])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        REGISTRY_COST, // attached deposit
    );

    let aggregated = call!(bob, fpo.registry_aggregate_median(bob.account_id()));

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
    log!("**{:?}", tx);
    call!(root, fpo.pair_exists("ETH/USD".to_string())).assert_success();

    let price_entry = call!(root, fpo.get_entry("ETH/USD".to_string()));
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");

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
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec(),
                p3_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone(),
                provider3_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());
    let tx = call!(
        root,
        fpo.add_signers(vec![provider3_pk.clone()], "ETH/USD".to_string())
    )
    .assert_success();

    println!("----tx {:?}", tx);
    // push data after adding provider3 as signer
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec(),
                p3_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone(),
                provider3_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id
        )
    )
    .assert_success();
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
        fpo.rm_signers(vec![provider3_pk.clone()], "ETH/USD".to_string())
    );

    println!("----GAS BURNT IN rm_signers: {:?}", tx.gas_burnt()); //hash: 3529919471594, vec: 3458769403490

    // try pushing data with invalid signer provider3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec(),
                p3_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone(),
                provider3_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id + 1
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());
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
        &json!([
            "ETH/USD".to_string(),
            8,
            U128(2000),
            vec![provider1_pk.clone(), provider2_pk.clone()]
        ])
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

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
    log!("**{:?}", tx);
    call!(root, fpo.pair_exists("ETH/USD".to_string())).assert_success();

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
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );
    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");
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
    let p0_invalid_sig_vec = p0_invalid_sig
        .try_to_vec()
        .expect("CANT CONVERT SIG TO VEC");

    let p1_sig_vec = p1_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    let p2_sig_vec = p2_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let p3_sig_vec = p3_sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");

    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    // try pushing data with invalid signer provider3
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec(),
                p3_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone(),
                provider3_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000, 4000],
            vec![U128(1000), U128(2000), U128(3000), U128(4000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

    // try pushing data with invalid signature (Wrong round_id for provider0)
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_invalid_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

    // try pushing data with duplicate signature
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----TXXXXXXXXX {:?}", tx.status());

    // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
    // let bob update root's feed
    let storage_used_before = env::used_gas();
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2000), U128(3000)],
            round_id
        )
    );
    let storage_used_after = env::used_gas();
    log!("GAS USED =  {:?}", storage_used_after - storage_used_before);

    println!("----tx {:?}", tx.gas_burnt());

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
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );

    // try pushing data with wrong round_id
    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            vec![U128(1000), U128(2000), U128(3000)],
            round_id // 1
        )
    );
    // assert error
    assert!(!tx.is_ok());
    println!("----tx {:?}", tx.status());

    let round_id: u64 = price_entry.unwrap_json_value()["latest_round_id"]
        .to_owned()
        .as_u64()
        .expect("Couldn't fetch round_id");
    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(1000));
    let data: &[u8] = message.as_bytes();
    let p0_sig = provider0.signer.sign(data);

    let message = format!("{}:{}:{:?}", "ETH/USD", round_id, U128(2500));
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
    let storage_used_before = env::storage_usage();

    let tx = call!(
        bob,
        fpo.push_data_signed(
            vec![
                p0_sig_vec[1..].to_vec(),
                p1_sig_vec[1..].to_vec(),
                p2_sig_vec[1..].to_vec()
            ],
            vec![
                provider0_pk.clone(),
                provider1_pk.clone(),
                provider2_pk.clone()
            ],
            "ETH/USD".to_string(),
            // vec![1000, 2000, 3000],
            vec![U128(1000), U128(2500), U128(3000)],
            round_id // 1
        )
    );
    let storage_used_after = env::storage_usage();
    log!(
        "STORAGE USED =  {:?}",
        storage_used_after - storage_used_before
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
        &"2500".to_string()
    );

    println!(
        "Returned round_id: {:?}",
        &price_entry.unwrap_json_value()["latest_round_id"]
            .to_owned()
            .as_u64()
    );
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

    call!(root, fpo.pair_exists("ETH/USD".to_string())).assert_success();
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

    let tx = call!(
        root,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000), vec![root_pk.clone()])
    );
    println!("tx: {:?}", tx);
    assert!(!tx.is_ok())
}

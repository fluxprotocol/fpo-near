// use std::io::Bytes;

// use std::str::Bytes;
// use ::byte_strings::concat_bytes;

use std::convert::TryFrom;
use std::str::FromStr;

use near_account_id::AccountId;
use near_fpo::FPOContractContract;
use near_sdk::{PublicKey, log};
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::{U128, Base58PublicKey};
use near_sdk::serde_json::json;
use near_sdk_sim::near_crypto::{InMemorySigner, KeyType, EmptySigner};
use near_sdk_sim::to_yocto;
use near_sdk_sim::{call, deploy, init_simulator, ContractAccount, UserAccount, borsh::BorshSerialize, near_crypto::{ED25519PublicKey, Signer}, near_crypto::SecretKey};
// extern crate ed25519_dalek;
// extern crate rand;


// use rand::rngs::{OsRng};
// use rand_core::{RngCore, OsRng};

// use ed25519_dalek::Keypair;
// use ed25519_dalek::Signature;
// use ed25519_dalek::{Signature, Keypair};
// use near_sdk_sim::near_crypto::Signer;
// use near_sdk_sim::near_crypto::Signer;


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;
pub const STORAGE_COST: u128 = 1_700_000_000_000_000_000_000;

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
fn simulate_push_data_signed() {
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

    let root_signer = InMemorySigner::from_seed(&root.account_id.as_str(), KeyType::ED25519, &root.account_id.as_str());

    println!("x = {:?}", root_signer.public_key);
    let message = format!("{}:{}", "ETH/USD", "4000"); 
    let data: &[u8]  = message.as_bytes();

    let sig = root_signer.sign(data);
    let sig_vec = sig.try_to_vec().expect("CANT CONVERT SIG TO VEC");
    println!("----sig_vec {:?}", sig_vec.len());

    let verif1 = root_signer.verify(data, &sig);
    let verif2 = sig.verify(data, &root_signer.public_key);
    println!("----VERIFIED {:?}", verif1);
    println!("----VERIFIED {:?}", verif2);
    let signer_pk_vec = root_signer.public_key.try_to_vec().expect("CANT CONVERT PK TO VEC");

    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));

    // For some reason near_crypto's signature is converted to a 65 bytes vec, removing the first byte verifies using ed25519_dalek tho
    // let bob update root's feed
    let tx = call!(bob, fpo.push_data_signed(sig_vec[1..].to_vec(), root.account_id(), signer_pk_vec[1..].to_vec() ,"ETH/USD".to_string(), "4000".to_string()));

    println!("----tx {:?}", tx);

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

// #[test]
// fn simulate_push_data_signed() {
    // let (root, fpo) = init();

    // call!(root, fpo.new()).assert_success();
    // // create a price pair, check if it exists, and get the value
    // root.call(
    //     fpo.account_id(),
    //     "create_pair",
    //     &json!(["ETH/USD".to_string(), 8, U128(2000)])
    //         .to_string()
    //         .into_bytes(),
    //     DEFAULT_GAS,
    //     STORAGE_COST, // attached deposit
    // );
    // call!(
    //     root,
    //     fpo.pair_exists("ETH/USD".to_string(), root.account_id())
    // )
    // .assert_success();
    // let price_entry = call!(
    //     root,
    //     fpo.get_entry("ETH/USD".to_string(), root.account_id())
    // );
    // println!(
    //     "Returned Price: {:?}",
    //     &price_entry.unwrap_json_value()["price"].to_owned()
    // );

    // let account_id = "alice".to_string();
    // // Creates a signer which contains a public key.
    // let signer = InMemorySigner::from_seed(&account_id, KeyType::ED25519, &account_id);
    // let x = InMemorySigner::from(root.signer);
    // // update the data
    // call!(root, fpo.push_data("ETH/USD".to_string(), U128(4000))).assert_success();

    // // get the updated data
    // let price_entry = call!(
    //     root,
    //     fpo.get_entry("ETH/USD".to_string(), root.account_id())
    // );

    // // output and check the data
    // println!(
    //     "Returned Price: {:?}",
    //     &price_entry.unwrap_json_value()["price"]
    // );
    // debug_assert_eq!(
    //     &price_entry.unwrap_json_value()["price"].to_owned(),
    //     &"4000".to_string()
    // );

    // let data1: &[u8] = "ETH/USD".as_bytes();
    // let data2: &[u8] = "2000".as_bytes();
    // let data: &[u8]= concat_bytes!(b"ETH/USD", b"2000");
    // let message = format!("{}:{}", "ETH/USD", "4000"); 
    // let data: &[u8]  = message.as_bytes();

    // let mut csprng = OsRng{};
    // let keypair: Keypair = Keypair::generate(&mut csprng);
    // let sk = root.signer.secret_key.unwrap_as_ed25519();
    // let pk: &ED25519PublicKey = root.signer.public_key.unwrap_as_ed25519();
    
    // let keypair: Keypair = Keypair::from_bytes(sk.to_owned().as_bytes()).unwrap();

    // println!("----keypair {:?}", keypair);

    // let message: &[u8] = b"This is a test of the tsunami alert system.";
    // let signature: Signature = keypair.sign(data);

    // let signature = root.signer.sign(data);
    // println!("----SIGNATURE TYPEEE {:?}", signature.key_type());

    // println!("----SIGNATURE {:?}", signature.to_bytes().to_vec());
    // let sig = signature.try_to_vec().expect("CANT CONVERT TO VEC");
    // println!("----SIGNATURE {:?}", sig);

    // let verif = keypair.verify(data, &signature);
    // let verif = signature.verify(data, &root.signer.public_key);
    // println!("----VERIFIED {:?}", verif);
    // println!("----root.account() {:?}", root);

    // let root_pk = root.signer.public_key.try_to_vec().expect("NOO");
    // let root_pk = keypair.public.to_bytes().to_vec();
    // let root_pk = root.signer.public_key().unwrap_as_ed25519().as_ref().to_vec();
    // let pk58 = root.signer.public_key.clone();

    // println!("----root_pk_str {:?}", pk58);
    // println!("----root_pk_ {:?}", root.signer.public_key().unwrap_as_ed25519());

    // println!("-----accId", nearAPI.utils.PublicKey.fromString(pk58).data.hexSlice());

    // println!("----root_pk {:?}", root_pk);
    // let sig = signature.try_to_vec().unwrap();
    // println!("----sig {:?}", sig);

    // update the data
    // let tx = call!(root, fpo.push_data_signed(sig, root, pk58t.account_id(), root_pk,"ETH/USD".to_string(), "4000".to_string()));
    // println!("----TXXX {:?}", tx);

    // let pk58 = root.signer.public_key.to_string();
    // println!("----pk58 {:?}", pk58);

    // let bob_acc = near_sdk::AccountId::from_str(&pk58[..]).unwrap();
    // let bob_acc = near_sdk::AccountId::from(pk58);
    // let bob = root.create_user_from(&root, "bob".parse().unwrap(), to_yocto("1000000"));
    // println!("----bob_acc {:?}", bob.signer.public_key());
    // println!("----bob_acc {:?}", root.account_id);


    // let bob = root.create_user(bob_acc, to_yocto("1000000"));

    //   // get the updated data
    // let price_entry = call!(
    //     root,
    //     fpo.get_entry("ETH/USD".to_string(), root.account_id())
    // );

    // // output and check the data
    // println!(
    //     "Returned Price: {:?}",
    //     &price_entry.unwrap_json_value()["price"]
    // );
    // debug_assert_eq!(
    //     &price_entry.unwrap_json_value()["price"].to_owned(),
    //     &"4000".to_string()
    // );


// }


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




#[test]
fn simulate_creating_registeries() {
    let (root, fpo) = init();
    call!(root, fpo.new()).assert_success();

    // create pricepairs from root
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(2500)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    root.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC/USD".to_string(), 8, U128(40000)])
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
    call!(
        root,
        fpo.pair_exists("BTC/USD".to_string(), root.account_id())
    )
    .assert_success();

    // create pricepairs from bob
    let bob = root.create_user("bob".parse().unwrap(), to_yocto("1000000"));
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["ETH/USD".to_string(), 8, U128(3000)])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );
    bob.call(
        fpo.account_id(),
        "create_pair",
        &json!(["BTC/USD".to_string(), 8, U128(30000)])
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
    call!(
        bob,
        fpo.pair_exists("BTC/USD".to_string(), bob.account_id())
    )
    .assert_success();

     // create a registery for root
     root.call(
        fpo.account_id(),
        "create_registry",
        &json!([ 
            vec![
                vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
                vec!["BTC/USD".to_string(), "BTC/USD".to_string()],
            ], 
            vec![vec![root.account_id(), bob.account_id()], vec![root.account_id(), bob.account_id()]], 
            0])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );


    // create a registery for bob
    bob.call(
        fpo.account_id(),
        "create_registry",
        &json!([ 
            vec![
                vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
                vec!["BTC/USD".to_string(), "BTC/USD".to_string()],
            ], 
            vec![vec![root.account_id(), bob.account_id()], vec![root.account_id(), bob.account_id()]], 
            0])
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        STORAGE_COST, // attached deposit
    );

    // aggregate values from root's registery
    let aggregated = call!(
        root,
        fpo.registry_aggregate(root.account_id())
    );
    println!(
        "Returned aggregated values from root's registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"2750".to_string(), &"35000".to_string()])
        
    );
   
  
    // aggregate values from bob's registery
    let aggregated = call!(
        bob,
        fpo.registry_aggregate(bob.account_id())
    );
    println!(
        "Returned aggregated values from bob's registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"2750".to_string(), &"35000".to_string()])
        
    );

    // update root's ETH/USD pricefeed
    call!(root, fpo.push_data("ETH/USD".to_string(), U128(4000))).assert_success();

    // aggregate values from root's registery after updating
    let aggregated = call!(
        root,
        fpo.registry_aggregate(root.account_id())
    );
    println!(
        "Returned aggregated values from root's  registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"3500".to_string(), &"35000".to_string()])
        
    );


    // aggregate values from bob's registery after updating
    let aggregated = call!(
        bob,
        fpo.registry_aggregate(bob.account_id())
    );
    println!(
        "Returned aggregated values from bob's registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"3500".to_string(), &"35000".to_string()])
        
    );






    // update bob's BTC/USD pricefeed
    call!(bob, fpo.push_data("BTC/USD".to_string(), U128(50000))).assert_success();

    // aggregate values from root's registery after updating
    let aggregated = call!(
        root,
        fpo.registry_aggregate(root.account_id())
    );
    println!(
        "Returned aggregated values from root's  registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"3500".to_string(), &"45000".to_string()])
        
    );


    // aggregate values from bob's registery after updating
    let aggregated = call!(
        bob,
        fpo.registry_aggregate(bob.account_id())
    );
    println!(
        "Returned aggregated values from bob's registery: {:?}",
        &aggregated.unwrap_json_value()["result"].to_owned()
    );
    
    debug_assert_eq!(
        &aggregated.unwrap_json_value()["result"].to_owned(),
        &json!([&"3500".to_string(), &"45000".to_string()])
        
    );

}




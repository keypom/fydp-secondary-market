use std::collections::HashMap;
use std::env;


extern crate anyhow;

use access_key_marketplace::{DropId, AddedDropDetails, ExtKeyData};
use near_sdk::{serde_json::json, PublicKey, AccountId};
use near_workspaces::{self, types::{NearToken, KeyType}, types::{Gas, SecretKey}};
use tokio;
//use anyhow::{self, anyhow};
use near_units;

const KEYPOM_WASM_PATH: &str = "./__tests__/ext_wasm/keypom.wasm";
const MARKETPLACE_WASM_PATH: &str = "./out/access_key_marketplace.wasm";
const LINKDROP_WASM_PATH: &str = "./__tests__/ext_wasm/linkdrop.wasm";



// #[tokio::test]
// async fn initial_sale_test() -> anyhow::Result<()> {
//     // Setup and init both contracts
//     let worker = near_workspaces::sandbox().await?;
//     let linkdrop_wasm = std::fs::read(LINKDROP_WASM_PATH)?;
//     let root = worker.dev_deploy(&linkdrop_wasm).await?;
//     let keypom_wasm = std::fs::read(KEYPOM_WASM_PATH)?;
//     let keypom = worker.dev_deploy(&keypom_wasm).await?;
//     let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_PATH)?;
//     let marketplace = worker.dev_deploy(&marketplace_wasm).await?;
//     let ali = worker.dev_create_account().await?;
//     let bob = worker.dev_create_account().await?;
//     env::set_var("RUST_BACKTRACE", "1");

//     // Init
//     let mut init = keypom.call("new").args_json(json!({
//         "root_account": root.id(),
//         "owner_id": keypom.id(),
//         "contract_metadata": {
//             "version": "3.0.0",
//             "link": "foo"
//         }
//     }))
//     .transact()
//     .await?;

//     assert!(init.is_success());

//     init = marketplace.call("new").args_json(json!({
//         "keypom_contract": keypom.id(),
//         "contract_owner": marketplace.id()
//     }))
//     .transact()
//     .await?;
    
//     assert!(init.is_success());

    


//     let deposit = NearToken::from_near(1);

//     let deposit_result = marketplace.as_account().call(keypom.id(), "add_to_balance").args_json(json!({
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;

//     assert!(deposit_result.is_success());
    
//     // List Event
//     let mut outcome = ali.call(marketplace.id(), "list_event").args_json(json!({
//         "event_id": "moon-party",
//         "max_markup": 2
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     // Creating drop
//     outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
//         "drop_id": "drop-id-premium",
//         "asset_data": [{
//             "assets": [null],
//             "uses": 2,
//         }],
//         "key_data": []
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
//         "drop_id": "drop-id-normal",
//         "asset_data": [{
//             "assets": [null],
//             "uses": 2,
//         }],
//         "key_data": []
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
//     .args_json(json!({
//         "event_id": "moon-party"
//     }))
//     .await?.json()?;

//     println!("--------------\n{}", result);
//     println!("hello");

//     // Add drops to event
//     let mut added_drops: HashMap<DropId, AddedDropDetails> = HashMap::new();
//     added_drops.insert("drop-id-normal".to_string(), AddedDropDetails { max_tickets: Some(50), price_by_drop_id: Some(near_units::parse_near!("1")) });
//     added_drops.insert("drop-id-premium".to_string(), AddedDropDetails { max_tickets: Some(5), price_by_drop_id: Some(near_units::parse_near!("3")) });
//     outcome = ali.call(marketplace.id(), "add_drop_to_event").args_json(json!({
//         "event_id": "moon-party",
//         "added_drops": added_drops
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;

//     let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
//     .args_json(json!({
//         "event_id": "moon-party"
//     }))
//     .await?.json()?;

//     println!("--------------\n{}", result);


//     // Attempting to buy without allowlist
//     let key: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap();
//     let new_key_info = ExtKeyData {
//         public_key: key,
//         key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
//         password_by_use: None,
//         metadata: None

//     };
//     outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//         "event_id": "moon-party",
//         "new_key_info": new_key_info,
//         "ticket_tier": 2
//     }))
//     .deposit(deposit.saturating_mul(2))
//     .transact()
//     .await?;
//     // Should not have enough money, allowlist
//     assert!(outcome.is_failure());

//     outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//         "event_id": "moon-party",
//         "new_key_info": new_key_info,
//         "ticket_tier": 1
//     }))
//     .deposit(deposit.saturating_mul(2))
//     .transact()
//     .await?;
//     // should not work, allowlist
//     assert!(outcome.is_failure());

//     // Allow marketplace to add tickets
//     outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
//         "drop_id": "drop-id-normal",
//         "account_ids": [marketplace.id().to_string()]
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     let drop_result: serde_json::Value = worker.view(keypom.id(), "get_drop_information")
//     .args_json(json!({
//         "drop_id": "drop-id-normal"
//     }))
//     .await?.json()?;

//     // 0.05N per key

//     println!("--------------\n{}", drop_result);

//     outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
//         "drop_id": "drop-id-premium",
//         "account_ids": [marketplace.id().to_string()]
//     }))
//     .deposit(deposit)
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     let drop_result: serde_json::Value = worker.view(keypom.id(), "get_drop_information")
//     .args_json(json!({
//         "drop_id": "drop-id-premium"
//     }))
//     .await?.json()?;

//     println!("--------------\n{}", drop_result);
    

//     // Should now fail based on balance, first should pass, second should not
//     outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//         "event_id": "moon-party",
//         "new_key_info": new_key_info,
//         "ticket_tier": 2
//     }))
//     .deposit(deposit.saturating_mul(2))
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
//     
//     println!("LOGS: {:?}", outcome.logs());
//     assert!(outcome.is_failure());

//     outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//         "event_id": "moon-party",
//         "new_key_info": new_key_info,
//         "ticket_tier": 1
//     }))
//     .deposit(deposit.saturating_mul(2))
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
//     
//     println!("LOGS: {:?}", outcome.logs());
//     assert!(outcome.is_success());

//     let key2: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtM".parse().unwrap();
//     // TODO: MULTIPLE FIRST SALES
//     let new_key_info2 = ExtKeyData {
//         public_key: key2,
//         key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
//         password_by_use: None,
//         metadata: None
//     };

//     outcome = bob.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//         "event_id": "moon-party",
//         "new_key_info": new_key_info2,
//         "ticket_tier": 1
//     }))
//     .deposit(deposit.saturating_mul(2))
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
//     
//     println!("LOGS: {:?}", outcome.logs());
//     assert!(outcome.is_success());

//     Ok(())
    
// }

// resale of owned key
#[tokio::test]
async fn owner_resale_test() -> anyhow::Result<()> {
     // Setup and init both contracts
     let worker = near_workspaces::sandbox().await?;
     let linkdrop_wasm = std::fs::read(LINKDROP_WASM_PATH)?;
     let root = worker.dev_deploy(&linkdrop_wasm).await?;
     let keypom_wasm = std::fs::read(KEYPOM_WASM_PATH)?;
     let keypom = worker.dev_deploy(&keypom_wasm).await?;
     let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_PATH)?;
     let marketplace = worker.dev_deploy(&marketplace_wasm).await?;
     let ali = worker.dev_create_account().await?;
     let bob = worker.dev_create_account().await?;
     env::set_var("RUST_BACKTRACE", "1");
 
     // Init
     let mut init = keypom.call("new").args_json(json!({
         "root_account": root.id(),
         "owner_id": keypom.id(),
         "contract_metadata": {
             "version": "3.0.0",
             "link": "foo"
         }
     }))
     .transact()
     .await?;
 
     assert!(init.is_success());
 
     init = marketplace.call("new").args_json(json!({
         "keypom_contract": keypom.id(),
         "contract_owner": marketplace.id()
     }))
     .transact()
     .await?;
     
     assert!(init.is_success());
 
 
     let deposit = NearToken::from_near(1);
 
     let deposit_result = marketplace.as_account().call(keypom.id(), "add_to_balance").args_json(json!({
     }))
     .deposit(deposit)
     .transact()
     .await?;
 
     assert!(deposit_result.is_success());
     
     // List Event
     let mut outcome = ali.call(marketplace.id(), "list_event").args_json(json!({
         "event_id": "moon-party",
         "max_markup": 2
     }))
     .deposit(deposit)
     .transact()
     .await?;
     assert!(outcome.is_success());
 
     // Creating drop
     outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
         "drop_id": "drop-id-normal",
         "asset_data": [{
             "assets": [null],
             "uses": 2,
         }],
         "key_data": []
     }))
     .deposit(deposit)
     .transact()
     .await?;
     assert!(outcome.is_success());
 
 
     let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
     .args_json(json!({
         "event_id": "moon-party"
     }))
     .await?.json()?;
 
     println!("--------------\n{}", result);
     println!("hello");
 
     // Add drops to event
     let mut added_drops: HashMap<DropId, AddedDropDetails> = HashMap::new();
     added_drops.insert("drop-id-normal".to_string(), AddedDropDetails { max_tickets: Some(50), price_by_drop_id: Some(near_units::parse_near!("1")) });
     outcome = ali.call(marketplace.id(), "add_drop_to_event").args_json(json!({
         "event_id": "moon-party",
         "added_drops": added_drops
     }))
     .deposit(deposit)
     .transact()
     .await?;
    assert!(outcome.is_success());
 
     let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
     .args_json(json!({
         "event_id": "moon-party"
     }))
     .await?.json()?;
 
     println!("--------------\n{}", result);
 
 
     // Buying initial sale
     let key_secret: String = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".to_string();
     let key: PublicKey = key_secret.parse().unwrap();
     let key_to_be_sold = ExtKeyData {
         public_key: key,
         key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
         password_by_use: None,
         metadata: None
     };

     // Allow marketplace to add tickets
     outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
         "drop_id": "drop-id-normal",
         "account_ids": [marketplace.id().to_string()]
     }))
     .deposit(deposit)
     .transact()
     .await?;
     assert!(outcome.is_success());
 
     let drop_result: serde_json::Value = worker.view(keypom.id(), "get_drop_information")
     .args_json(json!({
         "drop_id": "drop-id-normal"
     }))
     .await?.json()?;
 
     println!("--------------\n{}", drop_result);
 
     outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
         "event_id": "moon-party",
         "new_key_info": key_to_be_sold,
         "ticket_tier": 1
     }))
     .deposit(deposit.saturating_mul(2))
     .gas(Gas::from_tgas(150))
     .transact()
     .await?;
     
     println!("LOGS: {:?}", outcome.logs());
     assert!(outcome.is_success());

    // Resale process - try without approval, and then with 
    let key2_secret: SecretKey = SecretKey::from_random(KeyType::ED25519);
    let key2_public_string = key2_secret.public_key().to_string();
    println!("key2 public string: {}", key2_public_string);
    let key2 = key2_public_string.parse().unwrap();    
    let new_key_info_resale = ExtKeyData {
        public_key: key2,
        key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
        password_by_use: None,
        metadata: None
    };
    // Should fail with wrong account ID
    outcome = bob.call(marketplace.id(), "list_ticket").args_json(json!({
        "key": key_to_be_sold,
        "price": near_units::parse_near!("1.5"),
        "approval_id": 1
    }))
    .gas(Gas::from_tgas(150))
    .transact()
    .await?;
    assert!(outcome.is_failure());

    // Should now work with right account
    outcome = ali.call(marketplace.id(), "list_ticket").args_json(json!({
        "key": key_to_be_sold,
        "price": near_units::parse_near!("1.5"),
        "approval_id": 0
    }))
    .deposit(deposit)
    .gas(Gas::from_tgas(150))
    .transact()
    .await?;
    assert!(outcome.is_success());


    let key_info: serde_json::Value = worker.view(keypom.id(), "get_key_information").args_json(json!({
        "key": key_to_be_sold.public_key
    })).await?.json()?;
    println!("key info: {}", key_info);

    let token_id = key_info["token_id"].as_str().unwrap().to_string();

    // Add marketplace to allowlist
    outcome = ali.call(keypom.id(), "nft_approve").args_json(json!({
        "account_id": marketplace.id(),
        "token_id": token_id,
    }))
    .deposit(deposit)
    .gas(Gas::from_tgas(150))
    .transact()
    .await?;
println!("nft_approve outcome {:?}", outcome);
println!("nft_approve LOGS: {:?}", outcome.logs());
    assert!(outcome.is_success());

     // TODO: Test what happens if resale new key is the same, what state gets rolled back?
     let new_pub_key: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtM".parse().unwrap();
     let sale_outcome = bob.call(marketplace.id(), "buy_resale").args_json(json!({
        "public_key": key_to_be_sold.public_key,
        "new_owner_id": Some(bob.id()),
        "new_public_key": new_pub_key,
     }))
     .deposit(deposit.saturating_mul(2))
     .gas(Gas::from_tgas(150))
     .transact()
     .await?;
    println!("buy_resale outcome {:?}", sale_outcome);
        println!("buy_resale LOGS: {:?}", sale_outcome.logs());
    assert!(sale_outcome.is_success());


    // TODO: Figure out why nft_transfer is not working as expected
    let result: serde_json::Value = worker.view(keypom.id(), "get_key_information")
    .args_json(json!({
        "key": new_pub_key
    }))
    .await?.json()?;
    println!("result: {}", result);
    println!("bob: {}", bob.id().to_string());
    assert!(result["owner_id"] == bob.id().to_string());
 
     Ok(())
    
}

// [tokio::test]
// async fn owner_resale_test() -> anyhow::Result<()> {
//      // Setup and init both contracts
//      let worker = near_workspaces::sandbox().await?;
//      let linkdrop_wasm = std::fs::read(LINKDROP_WASM_PATH)?;
//      let root = worker.dev_deploy(&linkdrop_wasm).await?;
//      let keypom_wasm = std::fs::read(KEYPOM_WASM_PATH)?;
//      let keypom = worker.dev_deploy(&keypom_wasm).await?;
//      let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_PATH)?;
//      let marketplace = worker.dev_deploy(&marketplace_wasm).await?;
//      let mut ali = worker.dev_create_account().await?;
//      let bob = worker.dev_create_account().await?;
//      env::set_var("RUST_BACKTRACE", "1");
 
//      // Init
//      let mut init = keypom.call("new").args_json(json!({
//          "root_account": root.id(),
//          "owner_id": keypom.id(),
//          "contract_metadata": {
//              "version": "3.0.0",
//              "link": "foo"
//          }
//      }))
//      .transact()
//      .await?;
 
//      assert!(init.is_success());
 
//      init = marketplace.call("new").args_json(json!({
//          "keypom_contract": keypom.id(),
//          "contract_owner": marketplace.id()
//      }))
//      .transact()
//      .await?;
     
//      assert!(init.is_success());
 
 
//      let deposit = NearToken::from_near(1);
 
//      let deposit_result = marketplace.as_account().call(keypom.id(), "add_to_balance").args_json(json!({
//      }))
//      .deposit(deposit)
//      .transact()
//      .await?;
 
//      assert!(deposit_result.is_success());
     
//      // List Event
//      let mut outcome = ali.call(marketplace.id(), "list_event").args_json(json!({
//          "event_id": "moon-party",
//          "max_markup": 2
//      }))
//      .deposit(deposit)
//      .transact()
//      .await?;
//      assert!(outcome.is_success());
 
//      // Creating drop
//      outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
//          "drop_id": "drop-id-normal",
//          "asset_data": [{
//              "assets": [null],
//              "uses": 2,
//          }],
//          "key_data": []
//      }))
//      .deposit(deposit)
//      .transact()
//      .await?;
//      assert!(outcome.is_success());
 
 
//      let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
//      .args_json(json!({
//          "event_id": "moon-party"
//      }))
//      .await?.json()?;
 
//      println!("--------------\n{}", result);
//      println!("hello");
 
//      // Add drops to event
//      let mut added_drops: HashMap<DropId, AddedDropDetails> = HashMap::new();
//      added_drops.insert("drop-id-normal".to_string(), AddedDropDetails { max_tickets: Some(50), price_by_drop_id: Some(near_units::parse_near!("1")) });
//      outcome = ali.call(marketplace.id(), "add_drop_to_event").args_json(json!({
//          "event_id": "moon-party",
//          "added_drops": added_drops
//      }))
//      .deposit(deposit)
//      .transact()
//      .await?;
//     assert!(outcome.is_success());
 
//      let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
//      .args_json(json!({
//          "event_id": "moon-party"
//      }))
//      .await?.json()?;
 
//      println!("--------------\n{}", result);
 
 
//      // Buying initial sale
//      let key_secret: String = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".to_string();
//      let key: PublicKey = key_secret.parse().unwrap();
//      let new_key_info = ExtKeyData {
//          public_key: key,
//          key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
//          password_by_use: None,
//          metadata: None
//      };

//      // Allow marketplace to add tickets
//      outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
//          "drop_id": "drop-id-normal",
//          "account_ids": [marketplace.id().to_string()]
//      }))
//      .deposit(deposit)
//      .transact()
//      .await?;
//      assert!(outcome.is_success());
 
//      let drop_result: serde_json::Value = worker.view(keypom.id(), "get_drop_information")
//      .args_json(json!({
//          "drop_id": "drop-id-normal"
//      }))
//      .await?.json()?;
 
//      println!("--------------\n{}", drop_result);
 
//      outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
//          "event_id": "moon-party",
//          "new_key_info": new_key_info,
//          "ticket_tier": 1
//      }))
//      .deposit(deposit.saturating_mul(2))
//      .gas(Gas::from_tgas(150))
//      .transact()
//      .await?;
     
//      println!("LOGS: {:?}", outcome.logs());
//      assert!(outcome.is_success());

//     // Resale process - try without approval, and then with 
//     let key2_secret: SecretKey = SecretKey::from_random(KeyType::ED25519);
//     let key2_public_string = key2_secret.public_key().to_string();
//     println!("key2 public string: {}", key2_public_string);
//     let key2 = key2_public_string.parse().unwrap();    
//     let new_key_info_resale = ExtKeyData {
//         public_key: key2,
//         key_owner: Some(AccountId::try_from(ali.id().to_string()).unwrap()),
//         password_by_use: None,
//         metadata: None
//     };
//     // Should fail with wrong account ID
//     outcome = bob.call(marketplace.id(), "list_ticket").args_json(json!({
//         "key": new_key_info,
//         "price": near_units::parse_near!("1.5"),
//         "approval_id": 1
//     }))
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
//     assert!(outcome.is_failure());

//     let key_info: serde_json::Value = worker.view(keypom.id(), "get_key_information").args_json(json!({
//         "key": new_key_info.public_key
//     })).await?.json()?;
//     println!("key info: {}", key_info);
//     let token_id = key_info["token_id"].as_str().unwrap().to_string();

//     // Add marketplace to allowlist
//     outcome = ali.call(keypom.id(), "nft_approve").args_json(json!({
//         "account_id": marketplace.id(),
//         "token_id": token_id,
//     }))
//     .deposit(deposit)
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
//     assert!(outcome.is_success());

//     println!("before");

//     // Should now work with approval ID done
//     outcome = ali.call(marketplace.id(), "list_ticket").args_json(json!({
//         "key": new_key_info_resale,
//         "price": near_units::parse_near!("1.5"),
//         "approval_id": 1
//     }))
//     .gas(Gas::from_tgas(150))
//     .transact()
//     .await?;
// println!("OUTCOME: {:?}", outcome);
//     println!("LOGS: {:?}", outcome.logs());
//     assert!(outcome.is_success());


//      // TODO: Test what happens if resale new key is the same, what state gets rolled back?
//      let new_pub_key: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtM".parse().unwrap();
//      let mut sale_outcome = bob.call(marketplace.id(), "buy_resale").args_json(json!({
//         "public_key": new_key_info.public_key,
//         "new_owner_id": Some(bob.id()),
//         "new_public_key": new_pub_key,
//      }))
//      .deposit(deposit.saturating_mul(2))
//      .gas(Gas::from_tgas(150))
//      .transact()
//      .await?;
//     assert!(sale_outcome.is_success());

//     let result: serde_json::Value = worker.view(keypom.id(), "get_key_information")
//     .args_json(json!({
//         "public_key": new_key_info.public_key
//     }))
//     .await?.json()?;
//     assert!(result["owner_id"] == bob.id().to_string());
    
 
//      Ok(())
    
// }
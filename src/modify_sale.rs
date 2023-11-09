use std::ops::Index;

use crate::*;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    #[payable]
    pub fn modify_event_details(
        &mut self,
        event_id: EventID,
        new_name: Option<String>,
        new_host: Option<AccountId>,
        new_description: Option<String>,

    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == Some(env::predecessor_account_id()), "Must be event host to modify event details!");
        let mut event_details = self.event_by_id.get(&event_id).expect("No Event Found");
        event_details.name = new_name;
        if new_host.is_some(){
            event_details.host = new_host;
        }
        event_details.description = new_description;
        self.event_by_id.insert(&event_id, &event_details);

        let final_storage = env::storage_usage();
        if final_storage > initial_storage {
            let storage_used = final_storage - initial_storage;
            self.charge_deposit(near_sdk::json_types::U128((storage_used as u128 * env::storage_byte_cost()) as u128));
        }
        else if final_storage < initial_storage {
            let storage_freed = initial_storage - final_storage;
            Promise::new(env::predecessor_account_id()).transfer(storage_freed as u128 * env::storage_byte_cost() as u128).as_return();   
        }
        else{
            Promise::new(env::predecessor_account_id()).transfer(0).as_return();   
        }

    }

    #[payable]
    // IGNORING STORAGE
    pub fn modify_sale_prices(
        &mut self,
        event_id: EventID,
        new_price_by_drop_id: Option<HashMap<DropId, Option<U128>>>,
    ){
        self.assert_no_global_freeze();

        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == Some(env::predecessor_account_id()), "Must be event host to modify event details!");

        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.price_by_drop_id = new_price_by_drop_id.unwrap();
        self.event_by_id.insert(&event_id, &event);

    }
    
    /// Modify a Drop's Resale Conditions
    #[payable]
    pub fn modify_drop_resale_markup(
        &mut self,
        event_id: EventID,
        new_markup: u64
    ){
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.max_markup = new_markup;
        self.event_by_id.insert(&event_id, &event);
    }
    
    // Modify a Key's Resale Conditions
    
    // Rovoke a Resale
    pub fn revoke_resale(
        &mut self,
        public_key: PublicKey,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Make sure Key exists on contract
        require!(self.resale_per_pk.get(&public_key).is_some(), "Key Resale does not exist!");

        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .get_key_information(String::try_from(&public_key).unwrap())
                       .then(
                            Self::ext(env::current_account_id())
                            .internal_revoke_ticket(public_key, initial_storage, env::predecessor_account_id())
                        );
    }

    #[private]
    pub fn internal_revoke_ticket(
        &mut self,
        public_key: PublicKey,
        initial_storage: u64,
        predecessor: AccountId
    ){
         // Parse Response and Check if Fractal is in owned tokens
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            // expected result: Result<ExtKeyInfo, String>
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<Result<ExtKeyInfo, String>>(&val) {
                // Predecessor must either own the key, or sign the txn using the key!
                require!(predecessor == key_info.as_ref().unwrap().owner_id.clone()
                || env::signer_account_pk() == public_key, "Must own or use the access key being de-list!");
                let drop_id = &key_info.unwrap().drop_id;
                
                // Remove from approval_id_by_pk, resale_per_pk, 
                // listed_keys_per_drop, max_price_per_dropless_key, 
                // resales_for_event

                self.resale_per_pk.remove(&public_key);
                self.approval_id_by_pk.remove(&public_key);

                let listed_keys: Vec<PublicKey> = self.listed_keys_per_drop.get(&drop_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                let new_listed_keys: Vec<PublicKey> = listed_keys.iter().filter(|&x| x != &public_key).cloned().collect();
                self.listed_keys_per_drop.insert(&drop_id, &Some(new_listed_keys));
                
                self.max_price_per_dropless_key.remove(&public_key);

                if self.event_by_drop_id.contains_key(&drop_id){
                    let event_id = self.event_by_drop_id.get(&drop_id).unwrap();
                    let listed_keys_per_event: Vec<PublicKey> = self.resales_for_event.get(&event_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                    let new_listed_event_keys: Vec<PublicKey> = listed_keys_per_event.iter().filter(|&x| x != &public_key).cloned().collect();
                    self.resales_for_event.insert(&drop_id, &Some(new_listed_event_keys));
                }

                let final_storage = env::storage_usage();
                let storage_freed = final_storage - initial_storage;
                let refund_amount = storage_freed as u128 * env::storage_byte_cost();

                Promise::new(predecessor).transfer(refund_amount).as_return();

            } else {
             env::panic_str("ERR_WRONG_VAL_RECEIVED");
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
    }
}
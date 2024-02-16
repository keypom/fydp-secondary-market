use std::ops::Index;

use borsh::de;

use crate::*;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {
    
    // Modify a Key's Resale Price
    pub fn change_resale_price(&mut self, public_key: PublicKey, new_resale_price: U128){
        // TODO: IMPLEMENT
    }

    // Rovoke a Resale - only key owner can do this
    pub fn revoke_resale(
        &mut self,
        public_key: PublicKey,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Make sure ticket resale exists on contract
        require!(self.resale_info_per_pk.get(&public_key).is_some(), "Key Resale does not exist!");

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
                // REVOKE RESALE METHOD MUST BE ALLOWED ON EACH KEY
                require!(predecessor == key_info.as_ref().unwrap().owner_id.clone()
                || env::signer_account_pk() == public_key, "Must own or use the access key being de-list!");
                let drop_id = &key_info.unwrap().drop_id;
                
                // Remove from approval_id_by_pk, resale_per_pk, 
                // resales_per_drop, 
                // resales_per_event

                self.resale_info_per_pk.remove(&public_key);

                let listed_resales: Vec<StoredResaleInformation> = self.resales_per_drop.get(&drop_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                let new_listed_resales: Vec<StoredResaleInformation> = listed_resales.iter().filter(|&x| x.public_key != public_key).cloned().collect();
                self.resales_per_drop.insert(&drop_id, &Some(new_listed_resales));

                if self.event_by_drop_id.contains_key(&drop_id){
                    let event_id = self.event_by_drop_id.get(&drop_id).unwrap();
                    let listed_keys_per_event: Vec<StoredResaleInformation> = self.resales_per_event.get(&event_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                    let new_listed_event_keys: Vec<StoredResaleInformation> = listed_keys_per_event.iter().filter(|&x| x.public_key != public_key).cloned().collect();
                    self.resales_per_event.insert(&drop_id, &Some(new_listed_event_keys));
                }

                let final_storage = env::storage_usage();
                self.charge_storage(initial_storage, final_storage);
            } else {
             env::panic_str("Could not parse Key Information from Keypom Contract!");
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
    }
}
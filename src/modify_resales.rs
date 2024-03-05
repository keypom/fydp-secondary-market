use borsh::de;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {
    
    // Modify a Key's Resale Price
    pub fn change_resale_price(&mut self, public_key: PublicKey, new_resale_price: U128){
        self.assert_no_global_freeze();
        self.assert_resales_active(&public_key);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Make sure ticket resale exists on contract
        require!(self.resale_info_per_pk.get(&public_key).is_some(), "Key Resale does not exist!");

        // Predecessor must own the key
        require!(self.owned_tickets_per_account.get(&env::predecessor_account_id()).unwrap().unwrap().iter().map(|ticket| ticket.public_key.clone()).collect::<Vec<PublicKey>>().contains(&public_key), "Must own the access key being modified!");

        // Get resale, then modify price
        let mut resale = self.resale_info_per_pk.get(&public_key).unwrap();
        let final_price = self.clamp_price(new_resale_price, resale.drop_id.clone());
        resale.price = final_price;
        self.resale_info_per_pk.insert(&public_key, &resale);
    }

    // Rovoke a Resale - only key owner can do this
    // NON-OWNED KEYS CANNOT SIGN THIS TXN
    pub fn revoke_resale(
        &mut self,
        public_key: PublicKey,
    ){
        self.assert_no_global_freeze();
        self.assert_resales_active(&public_key);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Make sure ticket resale exists on contract
        require!(self.resale_info_per_pk.get(&public_key).is_some(), "Key Resale does not exist!");

        // Signer must own the key
        require!(self.owned_tickets_per_account.get(&env::signer_account_id()).unwrap().unwrap().iter().map(|ticket| ticket.public_key.clone()).collect::<Vec<PublicKey>>().contains(&public_key), "Must own the access key being de-list!");

        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                .get_key_information(String::try_from(&public_key).unwrap())
                .then(
                    Self::ext(env::current_account_id())
                    .internal_revoke_ticket(public_key, initial_storage)
                );
    }

    #[private]
    pub fn internal_revoke_ticket(
        &mut self,
        public_key: PublicKey,
        initial_storage: u64,
    ){
         // Parse Response and Check if Fractal is in owned tokens
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            // expected result: Result<ExtKeyInfo, String>
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<Result<ExtKeyInfo, String>>(&val) {
                // Predecessor must either own the key on Keypom contract as well, effectively a double check since drop ID needs be retrieved anyways
                require!(env::signer_account_id() == key_info.as_ref().unwrap().owner_id.clone(), "Must own the access key being de-list!");
                let drop_id = &key_info.unwrap().drop_id;

                self.resale_info_per_pk.remove(&public_key);

                let listed_resales: Vec<StoredResaleInformation> = self.resales_per_drop.get(&drop_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                let new_listed_resales: Vec<StoredResaleInformation> = listed_resales.iter().filter(|&x| x.public_key != public_key).cloned().collect();
                self.resales_per_drop.insert(&drop_id, &Some(new_listed_resales));

                let final_storage = env::storage_usage();
                self.charge_storage(initial_storage, final_storage, 0);
            } else {
             env::panic_str("Could not parse Key Information from Keypom Contract!");
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
    }
}
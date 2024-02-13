use std::string;

use near_sdk::store::key;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    // Frontend must sort drop ID prices for tiers, same with contract side

    // Buy Initial Sale Ticket (add_key)
    #[payable]
    pub fn buy_initial_sale(
        &mut self,
        event_id: EventID,
        new_key_info: ExtKeyData,
        // By default, ticket tier is sorted low to high. Tier 1 is lowest, tier 6 is higher etc.
        ticket_tier: u64 
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Verify Sale - price wise, was attached deposit enough?
        let received_deposit = env::attached_deposit();
        let tiered_drops = self.get_tiered_drop_list_for_event(event_id.clone(), None);
        let tier: usize = ticket_tier as usize - 1 as usize;
        require!(tier < tiered_drops.len(), "Desired Tier not in valid");
        
        let desired_drop = tiered_drops.get(tier).unwrap();
        let binding = self.event_by_id.get(&event_id);
        let price = binding.as_ref().unwrap().price_by_drop_id.get(&desired_drop.to_string()).unwrap();
        require!(received_deposit >= u128::from(price.clone()), "Not enough attached deposit to fund purchase at specified ticket tier!");

        require!(self.approved_drops.contains(&desired_drop.to_string()), "No drop found");

        near_sdk::log!("Trying to purchase key on drop ID {} at price of {}", desired_drop, u128::from(price.clone()));
        near_sdk::log!("Received paymnet: {}", received_deposit);
        let mut keys_vec = Vec::new();
        let public_key = new_key_info.public_key.clone();
        keys_vec.push(new_key_info);
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .add_keys(desired_drop.to_string(), keys_vec, None)
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_initial_sale_callback(initial_storage, env::predecessor_account_id(), public_key)
                        );
        
    }

    #[private]
    pub fn buy_initial_sale_callback(
        &mut self,
        initial_storage: u64, 
        predecessor: AccountId,
        public_key: PublicKey) -> bool {

             // Parse Response and Check if Fractal is in owned tokens
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            // expected result: Result<ExtKeyInfo, String>
            
            if let Ok(result) = near_sdk::serde_json::from_slice::<bool>(&val) {
                // Add key to owned keys
                if self.owned_keys_per_account.contains_key(&predecessor){
                    if self.owned_keys_per_account.get(&predecessor).is_none(){
                        // No existing vector
                        let mut keys_vec: Vec<PublicKey> = Vec::new();
                        keys_vec.push(public_key.clone());
                        self.owned_keys_per_account.insert(&predecessor, &Some(keys_vec));
                    }else{
                        self.owned_keys_per_account.get(&predecessor).unwrap().unwrap().push(public_key.clone());
                    }
                }else{
                   // Create new drop <-> vector pairing
                   let mut keys_vec: Vec<PublicKey> = Vec::new();
                   keys_vec.push(public_key.clone());
                   self.owned_keys_per_account.insert(&predecessor, &Some(keys_vec));
                }
                
                let final_storage = env::storage_usage();
                let storage_freed = final_storage - initial_storage;
                let refund_amount = storage_freed as u128 * env::storage_byte_cost();

                Promise::new(predecessor).transfer(refund_amount).as_return();
                return result

            } else {

             env::panic_str("ERR_WRONG_VAL_RECEIVED");
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
    }
    
    // Buy Resale
    #[payable]
    pub fn buy_resale(
        &mut self,
        public_key: PublicKey,
        new_owner_id: Option<AccountId>,
        new_public_key: PublicKey,
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Verify Sale - price wise, was attached deposit enough?
        let received_deposit = env::attached_deposit();
        let resale_info =  self.resale_info_per_pk.get(&public_key).expect("No resale for found this private key");
        let price = resale_info.price;
        require!(received_deposit >= u128::from(price), "Not enough attached deposit to fund purchase at specified ticket tier!");
        require!(new_public_key != public_key, "New and old key cannot be the same");

        let approval_id = resale_info.approval_id;

        let pk_string = String::from(&public_key);
        near_sdk::log!("getting key information with {:?}", pk_string);
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .get_key_information(pk_string)
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_resale_middle_callback(public_key, initial_storage, env::predecessor_account_id(), new_owner_id, new_public_key, approval_id)
                        );
        
    }

    #[private]
    pub fn buy_resale_middle_callback(
        &mut self,
        public_key: PublicKey,
        initial_storage: u64,
        predecessor: AccountId,
        new_owner_id: Option<AccountId>,
        new_public_key: PublicKey,
        approval_id: Option<u64>
    ){
         // Parse Response and Check if Fractal is in owned tokens
         if let PromiseResult::Successful(val) = env::promise_result(0) {
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<ExtKeyInfo>(&val) {
                let token_id = &key_info.token_id;
                let drop_id = &key_info.drop_id;
                let old_owner = &key_info.owner_id;

                // if key is not owned by contract, approval is required
                if key_info.owner_id != env::current_account_id(){
                    require!(approval_id.is_some(), "Approval ID is required for resale of non-owned keys");
                }

                ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .nft_transfer(Some(token_id.clone()), new_owner_id, approval_id, new_public_key)
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_resale_callback(initial_storage, predecessor, public_key, drop_id.to_string(), old_owner.clone())
                        );

            } else {
             env::panic_str("ERR_WRONG_VAL_RECEIVED");
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
    }

    #[private]
    pub fn buy_resale_callback(
        &mut self,
        initial_storage: u64, 
        predecessor: AccountId,
        public_key: PublicKey,
        drop_id: DropId,
        old_owner: AccountId
    ) {

             // Parse Response and Check if Fractal is in owned tokens
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            near_sdk::log!("made it into resale callback");
            self.resale_info_per_pk.remove(&public_key);
            let listed_keys: Vec<PublicKey> = self.listed_keys_per_drop.get(&drop_id).as_ref().unwrap().as_ref().unwrap().to_vec();
            let new_listed_keys: Vec<PublicKey> = listed_keys.iter().filter(|&x| x != &public_key).cloned().collect();
            self.listed_keys_per_drop.insert(&drop_id, &Some(new_listed_keys));

            // self.owned_keys_per_owner swap
            if old_owner != AccountId::try_from(self.keypom_contract.to_string()).unwrap() {
                let new_key_list: Vec<PublicKey> = self.owned_keys_per_account.get(&old_owner).unwrap().unwrap().iter().filter(|&x| x != &public_key).cloned().collect();
                self.owned_keys_per_account.insert(&old_owner, &Some(new_key_list));
            }
            
            
            if self.event_by_drop_id.contains_key(&drop_id){
                let event_id = self.event_by_drop_id.get(&drop_id).unwrap();
                let listed_keys_per_event: Vec<StoredResaleInformation> = self.resales_for_event.get(&event_id).as_ref().unwrap().as_ref().unwrap().to_vec();
                let new_listed_event_keys: Vec<StoredResaleInformation> = listed_keys_per_event.iter().filter(|&x| x.public_key != public_key).cloned().collect();
                self.resales_for_event.insert(&drop_id, &Some(new_listed_event_keys));
            }
            let final_storage = env::storage_usage();
            let storage_freed = final_storage - initial_storage;
            let refund_amount = storage_freed as u128 * env::storage_byte_cost();
            Promise::new(predecessor).transfer(refund_amount).as_return();   
        }
        else{
            env::panic_str("NFT Transfer Failed!")
        }  
    }



}
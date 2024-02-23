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
        ticket_tier: u64,
        new_owner: Option<AccountId>
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure valid new key owner
        if new_owner.is_some(){
            require!(new_owner.clone().unwrap() != self.keypom_contract, "New owner cannot be Keypom");
            require!(new_owner.clone().unwrap() != env::current_account_id(), "New owner cannot be marketplace");
        }

        // Get ticket tier
        let received_deposit = env::attached_deposit();
        let tiered_drops = self.get_tiered_drop_list_for_event(event_id.clone(), None);
        let tier: usize = ticket_tier as usize - 1 as usize;
        require!(tier < tiered_drops.len(), "Desired Tier not in valid");
        
        // Get ticket price, and evaluate if attached deposit is sufficient
        let desired_drop = tiered_drops.get(tier).unwrap();
        let binding = self.event_by_id.get(&event_id);
        let price = binding.as_ref().unwrap().price_by_drop_id.get(&desired_drop.to_string()).unwrap();
        require!(received_deposit.gt(&u128::from(price.clone())), "Not enough attached deposit to purchase ticket!");

        require!(self.approved_drops.contains(&desired_drop.to_string()), "No drop found");
        near_sdk::log!("Trying to purchase key on drop ID {} at price of {}", desired_drop, u128::from(price.clone()));
        near_sdk::log!("Received paymnet: {}", received_deposit);
        
        // Add key to keypom contract
        let mut keys_vec: Vec<ExtKeyData> = Vec::new();
        let public_key = new_key_info.public_key.clone();
        // TODO: ADD PASSWORD LOGIC HERE
        keys_vec.push(ExtKeyData{public_key: public_key.clone(), key_owner: new_owner.clone(), password_by_use: None, metadata: None});
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

             env::panic_str("Could not parse add key bool response from Keypom contract");
            }      
        }
        else{
            env::panic_str("Add Key Failed!")
        }  
    }
    
    // Buy Resale
    #[payable]
    pub fn buy_resale(
        &mut self,
        nft_transfer_memo: String,
        new_owner: Option<AccountId>,
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Parse msg to get transfer information
        let memo: NftTransferMemo = near_sdk::serde_json::from_str(&nft_transfer_memo).expect("Could not parse nft_transfer_memo to get nft transfer memo"); 
        let public_key = memo.linkdrop_pk.clone();
        let new_public_key = memo.new_public_key.clone();

        // Verify Sale - price wise, was attached deposit enough?
        let received_deposit = env::attached_deposit();
        let resale_info =  self.resale_info_per_pk.get(&public_key).expect("No resale for found this private key");
        let price = resale_info.price;
        require!(received_deposit.gt(&u128::from(price.clone())), "Not enough attached deposit to resale ticket!");
        require!(new_public_key != public_key, "New and old key cannot be the same");

        let approval_id = resale_info.approval_id;

        let pk_string = String::from(&public_key);
        near_sdk::log!("getting key information with {:?}", pk_string);
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .get_key_information(pk_string)
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_resale_middle_callback(public_key, initial_storage, env::predecessor_account_id(), new_owner, approval_id, memo)
                        );
        
    }

    #[private]
    pub fn buy_resale_middle_callback(
        &mut self,
        public_key: PublicKey,
        initial_storage: u64,
        predecessor: AccountId,
        new_owner: Option<AccountId>,
        approval_id: Option<u64>,
        memo: NftTransferMemo
    ){
         // Parse Response and Check if Fractal is in owned tokens
         if let PromiseResult::Successful(val) = env::promise_result(0) {
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<ExtKeyInfo>(&val) {
                let drop_id = &key_info.drop_id;
                let old_owner = &key_info.owner_id;

                // if key is not owned by contract, approval is required
                if key_info.owner_id != env::current_account_id(){
                    require!(approval_id.is_some(), "Approval ID is required for resale of non-owned keys");
                }

                ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .nft_transfer(new_owner.clone(), approval_id, serde_json::to_string(&memo).unwrap())
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_resale_callback(initial_storage, predecessor, public_key, drop_id.to_string(), old_owner.clone(), new_owner)
                        );

            } else {
             env::panic_str("Could not parse Key Information from Keypom Contract");
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
        old_owner: AccountId,
        new_owner: Option<AccountId>
    ) {

        // Parse Response and Update Contract Data Structures
        // TODO: MAKE SURE ALL DATA STRUCTURES ARE UPDATED HERE
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            near_sdk::log!("made it into resale callback");
            
            // remove ticket from resell market
            self.resale_info_per_pk.remove(&public_key);

            // Remove ticket from event resales
            let event_id = self.event_by_drop_id.get(&drop_id).unwrap();
            let listed_resales_per_event: Vec<StoredResaleInformation> = self.resales_per_event.get(&event_id).as_ref().unwrap().as_ref().unwrap().to_vec();
            let new_listed_event_resales: Vec<StoredResaleInformation> = listed_resales_per_event.iter().filter(|&x| x.public_key != public_key).cloned().collect();
            self.resales_per_event.insert(&drop_id, &Some(new_listed_event_resales));
            
            // Remove ticket from drop resales
            let listed_resales: Vec<StoredResaleInformation> = self.resales_per_drop.get(&drop_id).as_ref().unwrap().as_ref().unwrap().to_vec();
            let new_listed_resales: Vec<StoredResaleInformation> = listed_resales.iter().filter(|&x| x.public_key != public_key).cloned().collect();
            self.resales_per_drop.insert(&drop_id, &Some(new_listed_resales));

            // Remove key from old owner's owned keys if previous owner is not Keypom
            if old_owner != AccountId::try_from(self.keypom_contract.to_string()).unwrap() {
                let new_key_list: Vec<PublicKey> = self.owned_keys_per_account.get(&old_owner).unwrap().unwrap().iter().filter(|&x| x != &public_key).cloned().collect();
                self.owned_keys_per_account.insert(&old_owner, &Some(new_key_list));
            }

            // Add key to new owner's owned keys
            if new_owner.is_some(){
                let unwrapped_new_owner = new_owner.unwrap();
                if self.owned_keys_per_account.contains_key(&unwrapped_new_owner){
                    if self.owned_keys_per_account.get(&unwrapped_new_owner).is_none(){
                        // No existing vector
                        let mut keys_vec: Vec<PublicKey> = Vec::new();
                        keys_vec.push(public_key.clone());
                        self.owned_keys_per_account.insert(&unwrapped_new_owner, &Some(keys_vec));
                    }else{
                        self.owned_keys_per_account.get(&unwrapped_new_owner).unwrap().unwrap().push(public_key.clone());
                    }
                }
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
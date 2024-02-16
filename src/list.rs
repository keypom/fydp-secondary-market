use near_units::near;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    // *********** ASSUMING ALL NEW DROPS WITH NO KEYS ***********

    /// List an event
    #[payable]
    pub fn list_event(
        &mut self,
        // Unique event identifier 
        event_id: EventID,
        // Event Information
        event_name: Option<String>,
        description: Option<String>,
        date: Option<String>,
        host: Option<AccountId>,
        // Resale Markup
        max_markup: u64,
        // Associated drops, prices, and max tickets for each. If None, assume unlimited tickets for that drop 
        drop_ids: Option<Vec<DropId>>,
        max_tickets: Option<HashMap<DropId, Option<u64>>>,
        price_by_drop_id: Option<HashMap<DropId, U128>>,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure event with this ID does not already exist
        require!(self.event_by_id.get(&event_id).is_none(), "Event ID already exists!");


        // If drop_ids provided, prices must be provided as well
        if drop_ids.is_some(){
            let received_drop_ids = drop_ids.as_ref().unwrap();
            let received_price_by_drop_id = price_by_drop_id.as_ref().unwrap();
            for drop_id in received_drop_ids{
                require!(received_price_by_drop_id.contains_key(drop_id), "Price not provided for all drops!");
                // Ensure drops do not exist on the contract already
                require!(self.approved_drops.get(drop_id).is_none(), "Drop ID already exists on the contract!");
                require!(self.event_by_drop_id.get(drop_id).is_none(), "Drop ID already associated with an event!");
            }
        }

        let final_event_details = self.create_event_details(
            event_id, 
            event_name, 
            description, 
            date, 
            host, 
            max_markup, 
            max_tickets, 
            drop_ids, 
            price_by_drop_id);

        // Insert by event ID stuff first
        self.event_by_id.insert(&final_event_details.event_id, &final_event_details);
        self.resales_per_event.insert(&final_event_details.event_id, &None);
 
        // By Drop ID data structures
        let stored_drop_ids = final_event_details.drop_ids;
        for drop_id in stored_drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &final_event_details.event_id);
            self.resales_per_drop.insert(&drop_id, &None);
        }

        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(near_sdk::json_types::U128(storage_cost));
    }
    
    // List a ticket for sale on secondary market
    #[payable]
    pub fn list_ticket(
        &mut self,
        key: ExtKeyData,
        price: U128,
        approval_id: u64,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        near_sdk::log!("listing key {:?}", serde_json::to_string(&key.public_key));
        near_sdk::log!("Signer PK {:?}", serde_json::to_string(&env::signer_account_pk()));
        
        // Predecessor must own the key, or the marketplace must call list_ticket
        require!(env::predecessor_account_id() == key.key_owner.clone().unwrap_or(env::current_account_id()), "Must own or use the access key being listed!");

        near_sdk::log!("attached deposit: {}", env::attached_deposit());

        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
            .get_key_information(String::try_from(&key.public_key).unwrap())
            .then(
                 Self::ext(env::current_account_id())
                 .with_attached_deposit(env::attached_deposit())
                 .internal_list_ticket(key, price, approval_id, initial_storage)
             );
    }

    #[private] #[payable]
    pub fn internal_list_ticket(
        &mut self,
        key: ExtKeyData,
        price: U128,
        approval_id: u64,
        initial_storage: u64
    ){
        
        // Parse Response to ensure key exists on Keypom first
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            // expected result: Result<ExtKeyInfo, String>
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<ExtKeyInfo>(&val) {
                // Data structures to update: event_by_id, resales_per_event, resales_per_drop, approval_id_by_pk, resale_per_pk
                let drop_id = key_info.drop_id;

                // Require the key to be associated with an event                
                let event_id = self.event_by_drop_id.get(&drop_id).expect("Key not associated with any event, cannot list!");
                let event = self.event_by_id.get(&event_id).expect("No event found for Event ID");
                
                // Clamp price using max_markup
                let mut final_price = price;
                let base_price = event.price_by_drop_id.get(&drop_id).expect("No base price found for drop, cannot set max price");
                let max_price = u128::from(base_price.clone()) * event.max_markup as u128;
                if u128::from(price).gt(&max_price){
                    final_price = U128::from(max_price);
                }

                // resale info object
                let resale_info: StoredResaleInformation = StoredResaleInformation{
                    price: final_price,
                    public_key: key.public_key.clone(),
                    approval_id: Some(approval_id),
                    event_id: event_id.clone(),
                };



                // Resale per PK
                self.resale_info_per_pk.insert(&key.public_key, &resale_info);

                // updated listed resales per drop, if drop has no resales, add drop to this resaleInfo
                if self.resales_per_drop.contains_key(&drop_id){
                    if self.resales_per_drop.get(&drop_id).as_ref().unwrap().is_none(){
                        // No existing vector
                        let mut resale_vec: Vec<StoredResaleInformation> = Vec::new();
                        resale_vec.push(resale_info.clone());
                        self.resales_per_drop.insert(&event_id, &Some(resale_vec));
                    }else{
                        self.resales_per_drop.get(&drop_id).unwrap().unwrap().push(resale_info.clone());
                    }
                }else{
                    // Create new drop <-> vector pairing
                    let mut resale_vec: Vec<StoredResaleInformation> = Vec::new();
                    resale_vec.push(resale_info.clone());
                    self.resales_per_drop.insert(&drop_id, &Some(resale_vec));
                }

                // updated resales for event, if drop has no resales, add event to this resaleInfo
                if self.resales_per_event.contains_key(&event_id){
                    if self.resales_per_event.get(&event_id).as_ref().unwrap().is_none(){
                        // No existing vector
                        let mut resale_vec: Vec<StoredResaleInformation> = Vec::new();
                        resale_vec.push(resale_info.clone());
                        self.resales_per_event.insert(&event_id, &Some(resale_vec));
                    }else{
                        // Existing vector
                        self.resales_per_event.get(&event_id).unwrap().unwrap().push(resale_info);
                    }
                }else{
                   // Create new drop <-> vector pairing
                   let mut resale_vec: Vec<StoredResaleInformation> = Vec::new();
                   resale_vec.push(resale_info.clone());
                   self.resales_per_event.insert(&event_id, &Some(resale_vec));
                }
            } else {
             env::panic_str("Could Not Parse KeyInfo")
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
        
        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();
        near_sdk::log!("storage cost {}", storage_cost);
        near_sdk::log!("attached deposit: {}", env::attached_deposit());

        self.charge_deposit(near_sdk::json_types::U128::from(storage_cost));
    }


    // TODO: VERIFY IF ALL NECESSARY DATA STRUCTURES ARE UPDATED HERE
    // Add drop to an existing event
    #[payable]
    pub fn add_drop_to_event(
        &mut self, 
        event_id: EventID,
        added_drops: HashMap<DropId, AddedDropDetails>,
    ){
        // Data Structures to update: event_by_id (EventDetails), approved_drops, event_by_drop_id, resales_per_drop
        // EventDetails fields to update: max_tickets, drop_ids, price_by_drop_ids

        // Ensure no global freeze and event exists
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        require!(self.event_by_id.get(&event_id).is_some(), "Event not found!");
        
        // Ensure that all drops being added are not already associated with an event
        for drop in added_drops.keys(){
            require!(self.event_by_drop_id.get(drop).is_none(), "Drop already associated with an event!");
        }

        // Update event details
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        let added_drops_vec = added_drops.iter();
        let mut drop_ids: Vec<String> = Vec::new();

        for (key, val) in added_drops_vec{
            event.drop_ids.push(key.to_string());
            event.max_tickets.insert(key.to_string(), val.max_tickets);
            event.price_by_drop_id.insert(key.to_string(), val.price_by_drop_id);
            drop_ids.push(key.to_string());
        }

        self.event_by_id.insert(&event_id, &event);

        for drop_id in drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &event_id);
            // if let Some(pub_key) = &existing_keys.as_ref().unwrap().get(&drop_id){
            //     self.keys_by_drop_id.insert(&drop_id, &Some(pub_key.to_vec()));
            // }
            self.resales_per_drop.insert(&drop_id, &None);
        }
        
        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(near_sdk::json_types::U128(storage_cost));
    }
}
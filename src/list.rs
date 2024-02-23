use near_sdk::store::vec;
use near_units::near;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    /// List an event, expected call after drop creation succeeds, assuming no keys in those drops
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
        // Associated drops, prices, and max tickets for each. If None, assume unlimited tickets for that drop 
        max_tickets: HashMap<DropId, Option<u64>>,
        price_by_drop_id: HashMap<DropId, U128>,
    ) -> EventID {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure event with this ID does not already exist
        require!(self.event_by_id.get(&event_id).is_none(), "Event ID already exists!");

        let mut drop_ids: Vec<DropId> = vec![];

        // Ensure drop IDs in max tickets and price_by_drop_id match
        require!(max_tickets.len() > 0 && price_by_drop_id.len() > 0, "No drops provided!");
        require!(max_tickets.len() == price_by_drop_id.len(), "Max Tickets and Prices must have same number of drops!");
        let received_max_tickets = max_tickets.clone();
        let received_price_by_drop_id = price_by_drop_id.clone();
        for drop_id in received_max_tickets.keys(){
            require!(received_price_by_drop_id.contains_key(drop_id), "Max Tickets and Prices must have the same drops!");
            drop_ids.push(drop_id.clone());
        }

        let final_event_details = self.create_event_details(
            event_id.clone(), 
            event_name, 
            description, 
            date, 
            host, 
            drop_ids,
            max_tickets, 
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

        event_id
    }

    // NFT Approve Callback, list the ticket
    // TODO: CURRENTLY EATS UP ALL STORAGE, NEED TO RECONSIDER
    pub fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        require!(env::predecessor_account_id() == self.keypom_contract, "nft_on_approve be called by Keypom contract using nft_approve!");
        require!(env::signer_account_id() == owner_id || env::signer_account_id() == self.keypom_contract, "Must be owner or Keypom contract to approve NFT");

        // Parse msg to get price
        let received_resale_info: ReceivedResaleInfo = near_sdk::serde_json::from_str(&msg).expect("Could not parse msg to get resale information");    

        let drop_id = Marketplace::drop_id_from_token_id(&token_id);
        let price = received_resale_info.price;
        let key = received_resale_info.public_key;

        // Require the key to be associated with an event                
        let event_id = self.event_by_drop_id.get(&drop_id).expect("Key not associated with any event, cannot list!");
        let event = self.event_by_id.get(&event_id).expect("No event found for Event ID");

        // ~~~~~~~~~~~~~~ BEGIN LISTING PROCESS ~~~~~~~~~~~~~~
        // Clamp price using max_markup
        let mut final_price = price;
        let base_price = event.price_by_drop_id.get(&drop_id).expect("No base price found for drop, cannot set max price");
        let max_price = u128::from(base_price.clone()) * self.max_markup as u128;
        if u128::from(price).gt(&max_price){
            final_price = U128::from(max_price);
        }
        
        // resale info object
        let resale_info: StoredResaleInformation = StoredResaleInformation{
            price: final_price,
            public_key: key.clone(),
            approval_id: Some(approval_id),
            event_id: event_id.clone(),
        };

        // Resale per PK
        self.resale_info_per_pk.insert(&key, &resale_info);
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

        
        // Calculate used storage and charge the user
        // let net_storage = env::storage_usage() - initial_storage;
        // let storage_cost = net_storage as Balance * env::storage_byte_cost();
        // near_sdk::log!("storage cost {}", storage_cost);
        // near_sdk::log!("attached deposit: {}", env::attached_deposit());

        //self.charge_deposit(near_sdk::json_types::U128::from(storage_cost));
    }
}
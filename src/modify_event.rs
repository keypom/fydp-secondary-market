use std::ops::Index;

use borsh::de;

use crate::*;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    //TODO: IMPLEMENT STATUS CHECKS ON ALL SALES AND LISTINGS
    pub fn deactivate_event(event_id: EventId){
        self.assert_no_global_freeze();
        self.assert_event_active(event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.status = Status::Inactive;
        self.event_by_id.insert(&event_id, &event);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

    pub fn reactivate_event(event_id: EventId){
        self.assert_no_global_freeze();
        require!(self.event_by_id.get(&event_id).expect("No Event Found").status == Status::Inactive, "Event is not inactive, cannot reactivate");
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.status = Status::Active;
        self.event_by_id.insert(&event_id, &event);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

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

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");
        
        // if element is provided, then update it
        let mut event_details = self.event_by_id.get(&event_id).expect("No Event Found");
        event_details.host = new_host.unwrap_or(event_details.host);
        if let Some(name) = new_name { event_details.name = Some(name); }
        if let Some(description) = new_description { event_details.description = Some(description); }

        // reinsert event details
        self.event_by_id.insert(&event_id, &event_details);

        // charge storage
        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

    // Must update prices for all drops together, free drops should have price set to 0
    // DOES NOT MODIFY PRICES OF EXISTING RESALES
    #[payable]
    pub fn modify_sale_prices(
        &mut self,
        event_id: EventID,
        new_price_by_drop_id: HashMap<DropId, U128>,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");
        
        // update prices, make sure new price map covers all drops in event
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        require!(new_price_by_drop_id.len() == event.drop_ids.len(), "New Price Map must contain same number of drops!");
        for drop_id in event.drop_ids.iter(){
            require!(new_price_by_drop_id.contains_key(drop_id), "New Price Map must cover all drops in event!");
        }
        event.price_by_drop_id = new_price_by_drop_id;
        self.event_by_id.insert(&event_id, &event);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

    // Modify the maximum number of tickets that can be sold for each drop
    pub fn modify_max_tickets(
        &mut self,
        event_id: EventId,
        new_max_tickets_by_drop_id: HashMap<DropId, Option<u64>>
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");
        
        // update max tckets, make sure new ticket number map covers all drops in event
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        require!(new_max_tickets_by_drop_id.len() == event.drop_ids.len(), "New Price Map must contain same number of drops!");
        for drop_id in event.drop_ids.iter(){
            require!(new_max_tickets_by_drop_id.contains_key(drop_id), "New Price Map must cover all drops in event!");
        }

        event.max_tickets_by_drop_id = new_max_tickets_by_drop_id;
        self.event_by_id.insert(&event_id, &event);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
        
    }
}
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
        self.assert_event_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.contains_key(&event_id), "No Event Found"); 
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get_mut(&event_id).map(|event| {
            event.status = Status::Inactive;
        });

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

        self.event_by_id.get_mut(&event_id).map(|event| {
            event.status = Status::Active;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

    pub fn deactivate_resales(event_id: EventId){
        self.assert_no_global_freeze();
        self.assert_event_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.contains_key(&event_id), "No Event Found"); 
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get_mut(&event_id).map(|event| {
            event.resale_status = ResaleStatus::Inactive;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }

    pub fn reactivate_resales(event_id: EventID){
        self.assert_no_global_freeze();
        require!(self.event_by_id.get(&event_id).expect("No Event Found").resale_status == ResaleStatus::Inactive, "Event resale market is not inactive, cannot reactivate");
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get_mut(&event_id).map(|event| {
            event.resale_status = ResaleStatus::Active;
        });

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
        self.assert_event_active(&event_id);

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
        self.assert_event_active(&event_id);

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

    // Delete an Event and all associated resales
    pub fn delete_event(
        &mut self,
        event_id: EventID
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        self.assert_event_active(&event_id);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        // delete from all by drop data structures
        let drops = self.event_by_id.get(&event_id).unwrap().drop_ids;
        for drop in drops{
            self.approved_drops.remove(&drop);
            self.event_by_drop_id.remove(&drop);
            self.resales_per_drop.remove(&drop);
        }

        // remove ticket from owned tickets and resales by pk if it is for the desired event
        for account in self.owned_tickets_per_account.keys(){
            if let Some(owned_tickets) = self.owned_tickets_per_account.get(&account){
                if let Some(tickets) = owned_tickets{
                    for ticket in tickets{
                        if ticket.event_id == event_id{
                            self.owned_tickets_per_account.get_mut(&account).map(|tickets| {
                                tickets.remove(&ticket);
                            });
                            if self.resale_info_per_pk.get(&ticket.public_key).is_some(){
                                self.resale_info_per_pk.remove(&ticket.public_key);
                            }
                        }
                    }
                }
            }
        }


        // delete event
        self.event_by_id.remove(&event_id);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage);
    }
}
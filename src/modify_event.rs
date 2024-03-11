use near_sdk::store::vec;
use near_units::near;

use crate::*;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    //TODO: IMPLEMENT STATUS CHECKS ON ALL SALES AND LISTINGS
    pub fn deactivate_event(&mut self, event_id: EventID){
        self.assert_no_global_freeze();
        self.assert_event_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found"); 
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get(&event_id).as_mut().map(|event| {
            event.status = Status::Inactive;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }

    pub fn reactivate_event(&mut self, event_id: EventID){
        self.assert_no_global_freeze();
        require!(self.event_by_id.get(&event_id).expect("No Event Found").status == Status::Inactive, "Event is not inactive, cannot reactivate");
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get(&event_id).as_mut().map(|event| {
            event.status = Status::Active;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }

    pub fn deactivate_resales(&mut self, event_id: EventID){
        self.assert_no_global_freeze();
        self.assert_event_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found"); 
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get(&event_id).as_mut().map(|event| {
            event.status = Status::NoResales;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }

    pub fn reactivate_resales(&mut self, event_id: EventID){
        self.assert_no_global_freeze();
        self.assert_resales_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        self.event_by_id.get(&event_id).as_mut().map(|event| {
            event.status = Status::Active;
        });

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }

    // Must update prices for all drops together, free drops should have price set to 0
    // DOES NOT MODIFY PRICES OF EXISTING RESALES
    #[payable]
    pub fn modify_ticket_info(
        &mut self,
        event_id: EventID,
        new_ticket_info: HashMap<DropId, TicketInfo>,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        self.assert_event_active(&event_id);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");
        require!(new_ticket_info.len() > 0, "No drops provided to modify in event!");

        // update prices, make sure new price map covers all drops in event
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        let mut ticket_info: UnorderedMap<DropId, TicketInfo> = UnorderedMap::new(StorageKeys::TicketInfoPerEvent);
        for ticket_infos in new_ticket_info{
            ticket_info.insert(&ticket_infos.0, &ticket_infos.1);
        }
        event.ticket_info = ticket_info;
        self.event_by_id.insert(&event_id, &event);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
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
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        // delete from all by drop data structures
        let event = self.event_by_id.get(&event_id).unwrap();
        let drops = event.ticket_info.keys();
        for drop in drops{
            self.event_by_drop_id.remove(&drop);
            self.resales.remove(&drop);
        }

        // delete event
        self.event_by_id.remove(&event_id);

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }
}

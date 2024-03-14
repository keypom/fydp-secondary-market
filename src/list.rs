use near_sdk::store::vec;
use near_units::near;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    /// Create an event, expected call after drop creation succeeds, assuming no keys in those drops
    #[payable]
    pub fn create_event(
        &mut self,
        // Unique event identifier 
        event_id: EventID,
        // Host
        funder_id: AccountId,
        // Host Strip ID
        stripe_id: Option<String>,
        // Associated drops, prices, and max tickets for each. If None, assume unlimited tickets for that drop 
        ticket_information: HashMap<DropId, TicketInfo>
    ) -> EventID {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure event with this ID does not already exist
        require!(self.event_by_id.get(&event_id).is_none(), "Event ID already exists!");

        if self.marketplace_balance.get(&funder_id).is_none(){
            self.marketplace_balance.insert(&funder_id, &0);
        }

        // Ensure all prices are greater than base cost per key
        for ticket_info in ticket_information.values(){
            // only check if not free
            if ticket_info.price.0 > u128::from(0 as u64) {
                require!(u128::from(ticket_info.price) > (self.base_key_storage_size as u128 * env::storage_byte_cost()), "Price for a drop is less than the cost of a key!");
            }
        }

        // Ensure current balance + attached deposit can cover the cost of all the keys on Keypom contract
        let total_credit: Balance = self.marketplace_balance.get(&funder_id).unwrap() + env::attached_deposit();
        let total_tickets = ticket_information.iter().map(|x| x.1.max_tickets.unwrap_or(0)).sum::<u64>();
        let base_total_key_bytes = self.base_key_storage_size as u128 * total_tickets as u128;
        let base_total_key_cost = base_total_key_bytes * env::storage_byte_cost();

        near_sdk::log!("User Balance + Attached Deposit: {}, Total Key Storage Cost: {}", total_credit, base_total_key_cost);
        require!(total_credit > base_total_key_cost, "Attached Deposit and User Balance do not cover cost to create event!");

        // Ensure drop IDs in max tickets and price_by_drop_id match
        require!(ticket_information.len() > 0);

        // Insert new stripe ID, or ensure current one is valid
        if stripe_id.is_some(){
            if self.stripe_id_per_account.contains_key(&env::predecessor_account_id()){
                require!(self.stripe_id_per_account.get(&env::predecessor_account_id()).unwrap() == stripe_id.unwrap(), "Stripe ID does not match existing Stripe ID for this account!");
            }else{
                self.stripe_id_per_account.insert(&env::predecessor_account_id(), &stripe_id.unwrap());
            }
        }

        let final_event_details = self.create_event_details(
            event_id.clone(), 
            funder_id.clone(),
            ticket_information,
        );

        // Insert by event ID stuff first
        self.event_by_id.insert(&final_event_details.event_id, &final_event_details);
 
        // By Drop ID data structures
        for drop_id in final_event_details.ticket_info.keys() {
            self.event_by_drop_id.insert(&drop_id, &final_event_details.event_id);
            let identifier_hash = self.hash_string(&drop_id);
            self.resales.insert(&drop_id, &UnorderedMap::new(StorageKeys::ResalesPerDropInner{ identifier_hash }));
        }

        // Calculate used storage and charge the user
        self.charge_storage(initial_storage, env::storage_usage(), env::attached_deposit(), funder_id);

        event_id
    }

    // TODO: Review
    #[payable]
    pub fn add_drops_to_event(
        &mut self,
        event_id: EventID,
        ticket_information: HashMap<DropId, TicketInfo>
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        self.assert_event_active(&event_id);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().funder_id == env::predecessor_account_id(), "Must be event host to modify event details!");

        require!(ticket_information.len() > 0, "No drops provided to add to event!");

        // Ensure all drops are not already in event
        let event = self.event_by_id.get(&event_id).expect("No Event Found");

        for drop_id in ticket_information.keys(){
            require!(!event.ticket_info.keys().collect::<Vec<DropId>>().contains(&drop_id), "Drop already in event!");
        }

        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        // Update event details
        for ticket_tier_info in ticket_information.iter(){
            event.ticket_info.insert(&ticket_tier_info.0, &ticket_tier_info.1);
        }
        self.event_by_id.insert(&event_id, &event);

        // Update by drop ID data structures
        for drop_id in ticket_information.keys() {
            self.event_by_drop_id.insert(&drop_id, &event_id);
            let identifier_hash = self.hash_string(&drop_id);
            self.resales.insert(&drop_id, &UnorderedMap::new(StorageKeys::ResalesPerDropInner { identifier_hash }));
        }

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0, env::predecessor_account_id());
    }

    // Listing ticket through NFT Approve
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

        // Parse msg to get price and public key
        let received_resale_info: ReceivedResaleInfo = near_sdk::serde_json::from_str(&msg).expect("Could not parse msg to get resale information");    
        let price = received_resale_info.price;
        let key = received_resale_info.public_key;
        
        // Require the key to be associated with an event                
        let drop_id = self.drop_id_from_token_id(&token_id);
        let event_id = self.event_by_drop_id.get(&drop_id).expect("Key not associated with any event, cannot list!");
        self.assert_resales_active(&event_id);

        // ~~~~~~~~~~~~~~ BEGIN LISTING PROCESS ~~~~~~~~~~~~~~
        // Clamp price and create resale info object
        self.price_check(price, drop_id.clone());
        let resale_info: ResaleInfo = ResaleInfo{
            price,
            public_key: key.clone(),
            seller_id: owner_id,
            approval_id: Some(approval_id),
            event_id: event_id.clone(),
            drop_id: drop_id.clone(),
        };

        near_sdk::log!("Resale Info: {:?}", resale_info);  
        let mut sale_binding = self.resales.get(&drop_id); 
        near_sdk::log!("a");
        let sale = sale_binding.as_mut().unwrap();
        near_sdk::log!("b");
        sale.insert(&key, &resale_info);
        near_sdk::log!("c");
        self.resales.insert(&drop_id, &sale);
    }

    // Add stripe ID to marketplace
    #[payable]
    pub fn register_stripe_id(&mut self, stripe_id: String){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        require!(!self.stripe_id_per_account.contains_key(&env::predecessor_account_id()), "Stripe ID already registered for this account!");
        self.stripe_id_per_account.insert(&env::predecessor_account_id(), &stripe_id);
        self.charge_storage(initial_storage, env::storage_usage(), 0, env::predecessor_account_id());
    }
}
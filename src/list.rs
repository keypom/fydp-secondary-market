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
        // Host Strip ID
        stripe_id: Option<String>,
        // Event Information
        event_name: Option<String>,
        metadata: Option<String>,
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
        for drop_id in max_tickets.clone().keys(){
            require!(price_by_drop_id.contains_key(drop_id), "Max Tickets and Prices must have the same drops!");
            drop_ids.push(drop_id.clone());
        }

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
            event_name,
            metadata,
            drop_ids,
            max_tickets, 
            price_by_drop_id);

        // Insert by event ID stuff first
        self.event_by_id.insert(&final_event_details.event_id, &final_event_details);
 
        // By Drop ID data structures
        let stored_drop_ids = final_event_details.drop_ids;
        for drop_id in stored_drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &final_event_details.event_id);
            self.resales_per_drop.insert(&drop_id, &None);
        }

        // Calculate used storage and charge the user
        self.charge_storage(initial_storage, env::storage_usage(), 0);

        event_id
    }

    // TODO: Review
    #[payable]
    pub fn add_drops_to_event(
        &mut self,
        event_id: EventID,
        drop_ids: Vec<DropId>,
        max_tickets: HashMap<DropId, Option<u64>>,
        price_by_drop_id: HashMap<DropId, U128>,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        self.assert_event_active(&event_id);

        // Ensure correct perms
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().host == env::predecessor_account_id(), "Must be event host to modify event details!");

        // Ensure drop IDs in max tickets and price_by_drop_id match
        require!(max_tickets.len() == price_by_drop_id.len() && price_by_drop_id.len() == drop_ids.len(), "Drops, Max Tickets and Prices must have same number of drops!");
        require!(max_tickets.len() > 0, "No drops provided!");
        for drop_id in drop_ids.clone(){
            require!(price_by_drop_id.contains_key(&drop_id), "Prices and Drops must have the same drops!");
            require!(max_tickets.contains_key(&drop_id), "Max Tickets and Drops must have the same drops!");
        }

        // Ensure all drops are approved
        for drop_id in drop_ids.iter(){
            require!(self.approved_drops.contains(drop_id), "Drop not approved for use in marketplace!");
        }

        // Ensure all drops are not already in event
        let event = self.event_by_id.get(&event_id).expect("No Event Found");
        for drop_id in drop_ids.iter(){
            require!(!event.drop_ids.contains(drop_id), "Drop already in event!");
        }

        // Update event details
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.drop_ids.extend(drop_ids.clone());
        for drop_id in drop_ids.iter(){
            event.max_tickets.insert(drop_id.clone(), max_tickets.get(drop_id).unwrap().clone());
            event.price_by_drop_id.insert(drop_id.clone(), price_by_drop_id.get(drop_id).unwrap().clone());
        }
        self.event_by_id.insert(&event_id, &event);

        // Update by drop ID data structures
        for drop_id in drop_ids {
            self.event_by_drop_id.insert(&drop_id, &event_id);
            self.resales_per_drop.insert(&drop_id, &None);
        }

        let final_storage = env::storage_usage();
        self.charge_storage(initial_storage, final_storage, 0);
    }

    // Listing ticket through NFT Approve
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

        // Parse msg to get price and public key
        let received_resale_info: ReceivedResaleInfo = near_sdk::serde_json::from_str(&msg).expect("Could not parse msg to get resale information");    
        let price = received_resale_info.price;
        let key = received_resale_info.public_key;
        self.assert_resales_active(&key);
        
        // Require the key to be associated with an event                
        let drop_id = self.drop_id_from_token_id(&token_id);
        let event_id = self.event_by_drop_id.get(&drop_id).expect("Key not associated with any event, cannot list!");

        // ~~~~~~~~~~~~~~ BEGIN LISTING PROCESS ~~~~~~~~~~~~~~
        // Clamp price and create resale info object
        let final_price = self.clamp_price(price, drop_id.clone());
        let resale_info: StoredResaleInformation = StoredResaleInformation{
            price: final_price,
            public_key: key.clone(),
            approval_id: Some(approval_id),
            event_id: event_id.clone(),
            drop_id: drop_id.clone(),
        };

        // Resale per PK
        self.resale_info_per_pk.insert(&key, &resale_info);

        // Update resales per drop
        let mut resale_from_drop = self.resales_per_drop.get(&drop_id.clone());
        let resale_from_drop_vec = resale_from_drop.get_or_insert_with(|| Some(Vec::new()));
        resale_from_drop_vec.as_mut().unwrap().push(resale_info.clone());

        // ~~~~~~~~~~~~~~~~~~` STORAGE STUFF ~~~~~~~~~~~~~~~~~~`
        // Calculate used storage and charge the user
        // let net_storage = env::storage_usage() - initial_storage;
        // let storage_cost = net_storage as Balance * env::storage_byte_cost();
        // near_sdk::log!("storage cost {}", storage_cost);
        // near_sdk::log!("attached deposit: {}", env::attached_deposit());

        //self.charge_deposit(near_sdk::json_types::U128::from(storage_cost));
    }

    // Add stripe ID to marketplace
    #[payable]
    pub fn register_stripe_id(&mut self, stripe_id: String){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        require!(!self.stripe_id_per_account.contains_key(&env::predecessor_account_id()), "Stripe ID already registered for this account!");
        self.stripe_id_per_account.insert(&env::predecessor_account_id(), &stripe_id);
        self.charge_storage(initial_storage, env::storage_usage(), 0);
    }
}
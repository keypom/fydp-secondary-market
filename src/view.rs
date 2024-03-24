use crate::*;

#[near_bindgen]
impl Marketplace{

    // Return marketplace maximum markup
    pub fn get_max_markup(&self) -> u64 {
        self.max_markup
    }

    pub fn get_max_resale_for_drop(&self, drop_id: DropId) -> U128 {
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop");
        let base_price = self.event_by_id.get(&event_id).expect("No event found for event").ticket_info.get(&drop_id).expect("No ticket info found for drop").price;
        let max_markup = self.max_markup;
        let max_price = (u128::from(base_price.clone()) * u128::from(max_markup))/(100 as u128);
        U128(max_price)
    }
    
    // View calls -> all events/drops, filter by funder, get event info, get owner, keypom constract, resale price per pk, resales per event, etc.

    pub fn get_events_per_funder(&self, account_id: AccountId, limit: Option<u64>, from_index: Option<u64>) -> Vec<ExtEventDetails>{
        let funder_events: Vec<EventDetails> = self.event_by_id.iter().filter(|x| x.1.funder_id == account_id.clone()).map(|x| x.1).collect();
        let start = u128::from(from_index.unwrap_or(0));
         // Iterate through each event using an iterator
         funder_events.into_iter()
         // Skip to the index we specified in the start variable
         .skip(start as usize) 
         // Take the first "limit" elements in the vector. If we didn't specify a limit, use 50
         .take(limit.unwrap_or(50) as usize) 
         // Convert each to a External Event
         .map(|event| event.to_external_event())
         // Since we turned the keys into an iterator, we need to turn it back into a vector to return
         .collect()
    }

    pub fn get_event_supply_for_funder(&self, account_id: AccountId) -> u64 {
        self.event_by_id.iter().filter(|x| x.1.funder_id == account_id).count() as u64
    }

    pub fn get_event_supply(&self) -> u64 {
        self.event_by_id.len() as u64
    }  

    pub fn get_event_information(&self, event_id: EventID) -> ExtEventDetails {
        self.event_by_id.get(&event_id).expect("No Event Found").to_external_event()
    }

    // Get drop's stripe information, if it exists. Allows frontend to expose stripe payment method
    pub fn event_stripe_status(&self, event_id: EventID) -> (String, String){
        let funder = self.event_by_id.get(&event_id).expect("No Event Found").funder_id;
        if self.stripe_id_per_account.contains_key(&funder){
            let stripe_id = self.stripe_id_per_account.get(&funder).unwrap();
            (stripe_id.clone(), funder.to_string())
        }else{
            // return blank tuple
            ("".to_string(), "".to_string())
        }
    }

    

    pub fn get_stripe_enabled_events(&self) -> Vec<EventID> {
        self.event_by_id.iter().filter(|x| self.stripe_id_per_account.contains_key(&x.1.funder_id)).map(|x| x.1.event_id).collect()
    }

    pub fn get_max_tickets_for_drop(&self, drop_id: DropId) -> u64 {
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop");
        self.event_by_id.get(&event_id).expect("No event found for event").ticket_info.get(&drop_id).expect("No ticket info found for drop").max_tickets.unwrap_or(u64::MAX)
    }

    pub fn get_resales_per_drop(&self, drop_id: DropId) -> Vec<ResaleInfo> {
        let identifier_hash = self.hash_string(&drop_id);
        self.resales.get(&drop_id).unwrap_or(UnorderedMap::new(StorageKeys::ResalesPerDropInner { identifier_hash })).iter().map(|x| x.1).collect()
    }

    // get all resales (ticket, price, approval ID) for an event, can be empty
    pub fn get_resales_per_event(&self, event_id: EventID) -> Option<HashMap<DropId, Vec<ResaleInfo>>> {
        let event = self.event_by_id.get(&event_id).expect("No Event Found for Event ID");
        let drops = event.ticket_info.keys();
        let mut all_resales: HashMap<DropId, Vec<ResaleInfo>> = HashMap::new();
        for drop_id in drops{
            let drop_resales = self.get_resales_per_drop(drop_id.clone());
            all_resales.insert(drop_id.clone(), drop_resales.clone());
        }
        Some(all_resales)
    }

    // All resales on the contract, sorted by event
    pub fn get_all_resales(&self) -> HashMap<EventID, HashMap<DropId, Vec<ResaleInfo>>> {
        let all_event_id = self.get_event_ids();
        let mut all_resales: HashMap<EventID, HashMap<DropId, Vec<ResaleInfo>>> = HashMap::new();
        for event_id in all_event_id {
            let resales = self.get_resales_per_event(event_id.clone()).expect("get_resales_per_event returning None somehow");
            all_resales.insert(event_id.clone(), resales);
        }
        all_resales
    }

    // get ticket price
    pub fn get_ticket_price(&self, drop_id: DropId) -> U128 {
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop");
        self.event_by_id.get(&event_id).expect("No event found for event").ticket_info.get(&drop_id).expect("No price found for drop").price.clone()
    }

    // get all event IDs
    pub fn get_event_ids(&self) -> Vec<EventID> {
        self.event_by_id.iter().map(|x| x.1.event_id).collect()
    }

    // get stripe ID for an account
    pub fn get_stripe_id_for_account(&self, account_id: AccountId) -> Option<String> {
        self.stripe_id_per_account.get(&account_id)
    }

    pub fn get_user_balance(&self, account_id: AccountId) -> U128 {
        near_sdk::json_types::U128(self.marketplace_balance.get(&account_id).unwrap_or(0))
    }

    // get all event details
    pub fn get_events(&self, limit: Option<u64>, from_index: Option<u64>) -> Vec<ExtEventDetails> {
        let start = u128::from(from_index.unwrap_or(0));
         // Iterate through each token using an iterator
         self.event_by_id.iter()
         // Skip to the index we specified in the start variable
         .skip(start as usize) 
         // Take the first "limit" elements in the vector. If we didn't specify a limit, use 50
         .take(limit.unwrap_or(50) as usize) 
         // Get only the event details
         .map(|id_and_event| id_and_event.1)
         // Convert each to a External Event
         .map(|event| event.to_external_event())
         // Since we turned the keys into an iterator, we need to turn it back into a vector to return
         .collect()
    }
}
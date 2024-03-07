use crate::*;

#[near_bindgen]
impl Marketplace{

    // Return marketplace maximum markup
    pub fn get_max_markup(&self) -> u64 {
        self.max_markup
    }
    
    // View calls -> all events/drops, filter by funder, get event info, get owner, keypom constract, resale price per pk, resales per event, etc.

    pub fn get_events_per_funder(&self, funder: AccountId, limit: Option<u64>, from_index: Option<u64>) -> Vec<ExtEventDetails>{
        let funder_events: Vec<EventDetails> = self.event_by_id.iter().filter(|x| x.1.funder_id == funder.clone()).map(|x| x.1).collect();
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

    // get all resales (ticket, price, approval ID) for an event, can be empty
    pub fn get_resales_per_event(&self, event_id: EventID) -> Option<Vec<ResaleInfo>> {
        let event = self.event_by_id.get(&event_id).expect("No Event Found for Event ID");
        let drops = event.ticket_info.keys();
        let mut all_resales: Vec<ResaleInfo> = Vec::new();
        for drop in drops{
            let resales = self.resales.get(&drop).unwrap_or(UnorderedMap::new(StorageKeys::ResaleByPK));
            for resale in resales.iter() {
                all_resales.push(resale.1);
            }
        }
        Some(all_resales)
    }

    // All resales on the contract, sorted by event
    pub fn get_all_resales(&self) -> Vec<Vec<ResaleInfo>> {
        let all_event_id = self.get_event_ids();
        let mut all_resales: Vec<Vec<ResaleInfo>> = Vec::new();
        near_sdk::log!("all event id {:?}", all_event_id);
        for event_id in all_event_id {
            let resales = self.get_resales_per_event(event_id).expect("Get resales per event returning None");
            all_resales.push(resales);
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
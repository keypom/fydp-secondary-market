use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct ResaleInformation {
    pub price: U128,
    pub public_key: PublicKey,
    pub approval_id: Option<u64>,
    // Public Facing event name
    pub event_name: Option<String>,
    // Event hosts, not necessarily the same as all the drop funders
    pub host: AccountId,
    // Event ID, in case on needing to abstract on contract to multiple drops per event
    // For now, event ID is drop ID
    pub event_id: String,
    pub description: Option<String>,
    // Date
    pub date: Option<String>,
   
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct StoredResaleInformation {
    pub price: U128,
    pub public_key: PublicKey,
    pub approval_id: Option<u64>,
    pub event_id: EventID
}

#[near_bindgen]
impl Marketplace{
    
    // View calls -> all events/drops, filter by funder, get event info, get owner, keypom constract, resale price per pk, resales per event, etc.
    pub fn get_events_per_funder(&self, funder: AccountId, limit: Option<u64>, from_index: Option<u64>) -> Vec<EventDetails>{
        let funder_events: Vec<EventDetails> = self.event_by_id.iter().filter(|x| x.1.host == funder.clone()).map(|x| x.1).collect();
        let start = u128::from(from_index.unwrap_or(0));
         // Iterate through each token using an iterator
         funder_events.into_iter()
         // Skip to the index we specified in the start variable
         .skip(start as usize) 
         // Take the first "limit" elements in the vector. If we didn't specify a limit, use 50
         .take(limit.unwrap_or(50) as usize) 
         // Since we turned the keys into an iterator, we need to turn it back into a vector to return
         .collect()
    }

    // Probably not needed
    pub fn get_num_tiers_per_event(&self, event_id: EventID) -> u64 {
        self.event_by_id.get(&event_id).unwrap().drop_ids.len() as u64
    }

    // return sorted list of drop IDs based on price, default high to low pricing
    pub fn get_tiered_drop_list_for_event(&self, event_id: EventID, high_to_low: Option<bool>) -> Vec<DropId> {
        let mut drops: Vec<DropId> = self.event_by_id.get(&event_id).unwrap().drop_ids;

        drops.sort_by_key(|drop_id| {
            self.event_by_id.get(&event_id).as_ref().unwrap().price_by_drop_id.get(drop_id).unwrap().clone()
        });

        // sort high to low if specified, otherwise, keep it low to high
        if high_to_low.unwrap_or(false){
            drops.reverse();
        }
        
        drops
    }

    pub fn get_event_information(&self, event_id: EventID) -> EventDetails {
        self.event_by_id.get(&event_id).expect("No Event Found")
    }

    // Return ticket resll and event info
    pub fn get_full_resale_info_per_pk(&self, public_key: PublicKey) -> ResaleInformation {
        let simple_info = self.resale_info_per_pk.get(&public_key).expect("No resale for Public Key");
        let event = self.event_by_id.get(&simple_info.event_id).expect("No event found");
        ResaleInformation{
            price: simple_info.price,
            public_key: simple_info.public_key,
            approval_id: simple_info.approval_id,
            event_name: event.name,
            host: event.host,
            description: event.description,
            date: event.date,
            event_id: simple_info.event_id
        }
    }

    // get all resales (ticket, price, approval ID) for an event, can be empty
    pub fn get_resales_per_event(&self, event_id: EventID) -> Option<Vec<StoredResaleInformation>> {
        self.resales_per_event.get(&event_id).expect("No Resales for Event")
    }

    // All resales on the contract, sorted by event
    pub fn get_all_resales(&self) -> Vec<Vec<ResaleInformation>> {
        let all_event_id = self.get_event_ids();
        let all_events_copy = all_event_id.clone();
        let mut event_name;
        let mut host; 
        let mut description; 
        let mut date;
        let mut all_resales: Vec<Vec<ResaleInformation>> = Vec::new();
        let mut index = 0;
        near_sdk::log!("all event id {:?}", all_event_id);
        for event_id in all_event_id {
            // Same for all keys in event
            event_name = self.event_by_id.get(&event_id).unwrap().name.clone();
            host = self.event_by_id.get(&event_id).unwrap().host.clone();
            description = self.event_by_id.get(&event_id).unwrap().description.clone();
            date = self.event_by_id.get(&event_id).unwrap().date.clone();

            let resales = self.get_resales_per_event(event_id);
            let mut event_resales = Vec::new();
            for resale in resales.unwrap_or(Vec::new()) {
                let resale_info = ResaleInformation{
                    price: resale.price,
                    public_key: resale.public_key,
                    approval_id: resale.approval_id,
                    event_id: all_events_copy.get(index).unwrap().clone(),
                    event_name: event_name.clone(),
                    host: host.clone(),
                    description: description.clone(),
                    date: date.clone()
                };
                event_resales.push(resale_info);
            }
            all_resales.push(event_resales);
            index += 1;
        }
        all_resales
    }

    // Al drops that marketplace can add keys to
    pub fn get_drops_on_contract(&self) -> Vec<DropId> {
        self.approved_drops.iter().cloned().collect()
    }

    // get all event IDs
    pub fn get_event_ids(&self) -> Vec<EventID> {
        self.event_by_id.iter().map(|x| x.1.event_id).collect()
    }

    // get all tickets for a certain owner
    pub fn get_keys_for_owner(&self, owner_id: AccountId) -> Vec<PublicKey> {
        self.owned_keys_per_account.get(&owner_id).unwrap().unwrap()
    }

    // get all event details
    pub fn get_events(&self, limit: Option<u64>, from_index: Option<u64>) -> Vec<EventDetails> {
        let start = u128::from(from_index.unwrap_or(0));
         // Iterate through each token using an iterator
         self.event_by_id.iter()
         // Skip to the index we specified in the start variable
         .skip(start as usize) 
         // Take the first "limit" elements in the vector. If we didn't specify a limit, use 50
         .take(limit.unwrap_or(50) as usize) 
         // Get only the event details
         .map(|x| x.1)
         // Since we turned the keys into an iterator, we need to turn it back into a vector to return
         .collect()
    }
}
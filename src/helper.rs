use std::path::Prefix;

use crate::*;

#[near_bindgen]
impl Marketplace{
    pub(crate) fn create_event_details(
        &mut self,
        event_id: EventID,
        funder_id: AccountId,
        ticket_information: HashMap<DropId, TicketInfo>,
    ) -> EventDetails{

        let identifier_hash = self.hash_string(&event_id);
        let mut ticket_info: UnorderedMap<DropId, TicketInfo> = UnorderedMap::new(StorageKeys::TicketInfoPerEventInner { identifier_hash });
        
        for ticket_infos in ticket_information{
            near_sdk::log!("Inside loop Ticket Info: {:?}", ticket_infos);
            ticket_info.insert(&ticket_infos.0, &ticket_infos.1);
        }
        
        let event_details = EventDetails{
            funder_id,
            event_id,
            status: Status::Active,
            // unorderedmap from hashmap
            ticket_info,
        };

        event_details
    }

    pub(crate) fn assert_event_active(&self, event_id: &EventID){
        require!(self.event_by_id.get(event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(event_id).unwrap().status != Status::Inactive, "Event is not active");
    }

    pub(crate) fn assert_resales_active(&self, event_id: &EventID){
        let status = self.event_by_id.get(event_id).expect("No Event Found").status;
        require!(status != Status::NoResales && status != Status::Inactive, "Event resale market is not active");
    }

    pub(crate) fn hash_string(&self, string: &String) -> CryptoHash {
        env::sha256_array(string.as_bytes())
    }

    pub(crate) fn drop_id_from_token_id(&self, token_id: &TokenId) -> DropId{
        let delimiter = ":";
        let split: Vec<&str> = token_id.split(delimiter).collect();
        let drop_id = split[0];
        drop_id.to_string()
    }

    pub(crate) fn price_check(&self, current_price: U128, drop_id: DropId){
        // Get event and base price
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop, cannot set max price");
        let event = self.event_by_id.get(&event_id).expect("No event found for event ID, cannot set max price");
        let base_price = event.ticket_info.get(&drop_id).expect("No base price found for drop, cannot set max price").price;
        
        // divide by 100 to get percentage
        let max_price = (u128::from(base_price.clone()) * u128::from(self.max_markup))/(100 as u128);

        // Evaluate
        near_sdk::log!("Received Price: {}, Max Price: {}", u128::from(current_price), max_price);
        if u128::from(current_price).gt(&max_price){
            // price is too high, clamp it to max
            env::panic_str("Price is too high")
        }
    }
}
use crate::*;

#[near_bindgen]
impl Marketplace{
    pub(crate) fn create_event_details(
        &mut self,
        event_id: EventID,
        event_name: Option<String>,
        description: Option<String>,
        date: Option<String>,
        host: Option<AccountId>,
        drop_ids: Vec<DropId>,
        max_tickets: HashMap<DropId, Option<u64>>,
        price_by_drop_id: HashMap<DropId, U128>,
    ) -> EventDetails{

        require!(self.event_by_id.get(&event_id).is_none(), "Event ID already exists!");

        let event_details = EventDetails{
            name: event_name,
            host: host.unwrap_or(env::predecessor_account_id()),
            event_id,
            status: Status::Active,
            description,
            date,
            max_tickets,
            drop_ids,
            price_by_drop_id
        };
        event_details
    }

    pub(crate) fn assert_event_active(&self, event_id: EventID){
        require!(self.event_by_id.get(&event_id).is_some(), "No Event Found");
        require!(self.event_by_id.get(&event_id).unwrap().status == Status::Active, "Event is not active");
    }

    pub(crate) fn drop_id_from_token_id(&self, token_id: &TokenId) -> DropId{
        let delimiter = ":";
        let split: Vec<&str> = token_id.split(delimiter).collect();
        let drop_id = split[0];
        drop_id.to_string()
    }

    pub(crate) fn clamp_price(&self, current_price: U128, drop_id: DropId, event: EventDetails) -> U128{
        let final_price = current_price;
        let base_price = event.price_by_drop_id.get(&drop_id).expect("No base price found for drop, cannot set max price");
        let max_price = u128::from(base_price.clone()) * self.max_markup as u128;
        if u128::from(current_price).gt(&max_price){
            // price is too high, clamp it to max
            U128::from(max_price)
        }else{
            // price is fine
            final_price
        }
    }

}
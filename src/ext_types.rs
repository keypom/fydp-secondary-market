use crate::*;

impl EventDetails{
    pub fn to_external_event(&self) -> ExtEventDetails{
        let mut ticket_info: HashMap<DropId, TicketInfo> = HashMap::new();
        
        for (k, v) in self.ticket_info.iter(){
            ticket_info.insert(k.clone(), v.clone());
        }

        ExtEventDetails{
            funder_id: self.funder_id.clone(),
            event_id: self.event_id.clone(),
            status: self.status.clone(),
            ticket_info,
        }
    }
}
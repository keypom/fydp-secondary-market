use borsh::de;

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {
    
    // Modify a Key's Resale Price --> assume a dropId is given too
    pub fn change_resale_price(&mut self, public_key: PublicKey, new_resale_price: U128, drop_id: DropId){
        self.assert_no_global_freeze();
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop, cannot revoke resale");
        self.assert_resales_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        if let Some(mut resale) = self.resales.get(&drop_id).expect("No resales for Drop found").get(&public_key) {
            require!(resale.seller_id == env::predecessor_account_id(), "Must own the access key being modified!");
            // Get resale, then modify price
            let final_price = self.clamp_price(new_resale_price, resale.drop_id.clone());
            resale.price = final_price;
            self.resales.get(&drop_id).as_mut().expect("No resales for Drop found").insert(&public_key, &resale);
        } else {
            env::panic_str("Key Resale does not exist!");
        }
    }

    // Rovoke a Resale - only key owner can do this
    // Assume drop ID is given too
    // NON-OWNED KEYS CANNOT SIGN THIS TXN
    pub fn revoke_resale(
        &mut self,
        public_key: PublicKey,
        drop_id: DropId
    ){
        self.assert_no_global_freeze();
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop, cannot revoke resale");
        self.assert_resales_active(&event_id);
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        if let Some(resale) = self.resales.get(&drop_id).expect("No resales for Drop found").get(&public_key) {
            require!(resale.seller_id == env::predecessor_account_id(), "Must own the access key to de-list!");
            self.resales.get(&drop_id).as_mut().unwrap().remove(&public_key);

            let final_storage = env::storage_usage();
            self.charge_storage(initial_storage, final_storage, 0, resale.seller_id);
        } else {
            env::panic_str("Key Resale does not exist!");
        }
    }
}
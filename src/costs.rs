use crate::*;

#[near_bindgen]
impl Marketplace {
    pub(crate) fn charge_storage(&mut self, initial_storage: u64, final_storage: u64, credit: u128) -> Promise{
        if final_storage > initial_storage {
            let storage_used = final_storage - initial_storage;
            let cost = (storage_used as u128 * env::storage_byte_cost()) as u128;
            near_sdk::log!("Required Storage Cost: {}, Attached Deposit: {}", cost, credit);
            if cost > credit as u128 {
                
                env::panic_str("Insufficient Attached Deposit")
            }
            else{
                Promise::new(env::predecessor_account_id()).transfer(credit - cost).as_return()
            }
        }
        else if final_storage < initial_storage {
            let storage_freed = initial_storage - final_storage;
            let storage_freed_cost = storage_freed as u128 * env::storage_byte_cost() as u128;
            Promise::new(env::predecessor_account_id()).transfer(storage_freed_cost + credit).as_return()   
        }
        else{
            Promise::new(env::predecessor_account_id()).transfer(credit).as_return()
        }
    }
}
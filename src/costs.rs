use crate::*;

#[near_bindgen]
impl Marketplace {
    pub(crate) fn charge_deposit(&mut self, required_deposit: U128) -> Promise {
        near_sdk::log!("Required cost: {}", near_sdk::Balance::from(required_deposit));
        require!(env::attached_deposit() >= near_sdk::Balance::from(required_deposit), "Insufficient Attached Deposit");

        let amount_to_refund = env::attached_deposit() - near_sdk::Balance::from(required_deposit);

        near_sdk::log!("Refunding {} excess deposit", amount_to_refund);
        Promise::new(env::predecessor_account_id()).transfer(amount_to_refund).as_return()
    }

    pub(crate) fn charge_storage(&mut self, initial_storage: u64, final_storage: u64, credit: u64) -> Promise{
        if final_storage > initial_storage {
            let storage_used = final_storage - initial_storage;
            let cost = (storage_used as u128 * env::storage_byte_cost()) as u128;
            if cost > credit as u128 {
                self.charge_deposit(near_sdk::json_types::U128(cost - credit as u128))
            }
            else{
                Promise::new(env::predecessor_account_id()).transfer(credit as u128- cost).as_return()
            }
        }
        else if final_storage < initial_storage {
            let storage_freed = initial_storage - final_storage;
            let storage_freed_cost = storage_freed as u128 * env::storage_byte_cost() as u128;
            Promise::new(env::predecessor_account_id()).transfer(storage_freed_cost + credit as u128).as_return()   
        }
        else{
            Promise::new(env::predecessor_account_id()).transfer(credit as u128).as_return()
        }
    }
}
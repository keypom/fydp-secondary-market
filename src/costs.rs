use crate::*;

#[near_bindgen]
impl Marketplace {
    pub(crate) fn charge_storage(&mut self, initial_storage: u64, final_storage: u64, mut credit: u128, account_id: AccountId){
        // get current user balance, add that to any marketplace balance they may already have
        let current_user_balance = self.marketplace_balance.get(&account_id).expect("No user balance found in charge_storage");
        let credit_and_bal = credit + current_user_balance;
        
        // Storage was used
        if final_storage > initial_storage {
            let storage_used = final_storage - initial_storage;
            let cost = (storage_used as u128 * env::storage_byte_cost()) as u128;
            near_sdk::log!("Required Storage Cost: {}, User Balance and Attached Deposit: {}", cost, credit_and_bal);
            // If total storage cost exceeds the attached deposit and marketplace balance, panic
            if cost.gt(&(credit_and_bal as u128)) {
                env::panic_str("Insufficient Attached Deposit and Marketplace Balance")
            }
            // else, subtract the cost from the user's updated balance
            else{
                self.marketplace_balance.insert(&account_id, &(credit_and_bal - cost));    
            }
        }
        // Storage was freed
        else if final_storage < initial_storage {
            let storage_freed = initial_storage - final_storage;
            let storage_freed_cost = storage_freed as u128 * env::storage_byte_cost() as u128;
           
            // Add the freed storage cost and credit to the user's updated balance
            self.marketplace_balance.insert(&account_id, &(credit_and_bal + storage_freed_cost));      
        }
        // Storage stayed the same
        else{
            // Add the credit to the user's updated balance
            self.marketplace_balance.insert(&account_id, &(credit_and_bal));   
        }
        near_sdk::log!("{} New Balance: {}", account_id, self.marketplace_balance.get(&account_id).unwrap());
    }
}
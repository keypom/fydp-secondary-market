use crate::*;

#[near_bindgen]
impl Marketplace {
    pub fn withdraw_marketplace_balance(&mut self) -> Promise{
        let account_id = env::predecessor_account_id();
        let balance = self.marketplace_balance.get(&account_id).expect("No balance found for account");
        self.marketplace_balance.insert(&account_id, &0);
        Promise::new(account_id).transfer(balance).as_return()
    }

    #[payable]
    pub fn add_to_marketplace_balance(&mut self){
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();
        let current_balance = self.marketplace_balance.get(&account_id).unwrap_or(0);
        self.marketplace_balance.insert(&account_id, &(current_balance + deposit));
    }
}
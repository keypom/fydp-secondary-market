use crate::*;

#[near_bindgen]
impl Marketplace {
    pub(crate) fn charge_deposit(&mut self, required_deposit: U128) {
        let predecessor = env::predecessor_account_id();
        near_sdk::log!("Required cost: {}", near_sdk::Balance::from(required_deposit));
        require!(env::attached_deposit() >= near_sdk::Balance::from(required_deposit), "Insufficient Attached Deposit");

        let amount_to_refund = env::attached_deposit() - near_sdk::Balance::from(required_deposit);

        near_sdk::log!("Refunding {} excess deposit", amount_to_refund);
        Promise::new(predecessor).transfer(amount_to_refund);
        return;
    }
}
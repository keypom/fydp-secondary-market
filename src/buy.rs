use core::panic;
use std::string;

use near_sdk::store::{key, vec};

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    // Frontend must sort drop ID prices for tiers, same with contract side

    // Buy Initial Sale Ticket (add_key)
    #[payable]
    pub fn buy_initial_sale(
        &mut self,
        event_id: EventID,
        drop_id: DropId,
        new_keys: Vec<ExtKeyData>,
        new_owner: Option<AccountId>,
        estimated_keypom_deposit: U128
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Ensure valid new key owner
        if new_owner.is_some(){
            require!(new_owner.clone().unwrap() != env::current_account_id(), "New owner cannot be marketplace");
        }

        // Ensure event is active
        self.assert_event_active(&event_id);

        let buyer_id = env::predecessor_account_id();

        // Split deposit into Keypom storage and ticket payment, then ensure ticket payment is sufficient
        let received_deposit = env::attached_deposit();
        let keypom_deposit = u128::from(estimated_keypom_deposit.clone());
        let ticket_payment = received_deposit - keypom_deposit;
        let binding = self.event_by_id.get(&event_id);
        let single_ticket_price = binding.as_ref().unwrap().ticket_info.get(&drop_id.to_string()).unwrap().price;
        let ticket_price = u128::from(single_ticket_price.clone()) * new_keys.len() as u128;
        require!(ticket_payment.gt(&u128::from(ticket_price.clone())), "Attached Deposit minus Keypom storage costs do not cover ticket purchase cost!");
        
        // Get a return amount in case of over-payment
        let return_amount = ticket_payment - ticket_price;

        // Get Maximum number of tickets
        let event = self.event_by_id.get(&event_id).unwrap();
        let max_tickets = event.ticket_info.get(&drop_id.to_string()).unwrap().max_tickets;

        near_sdk::log!("Trying to purchase {} Tickets on drop ID {} at price of {} per Ticket", new_keys.len(), drop_id, u128::from(single_ticket_price.clone()));
        near_sdk::log!("Received paymnet: {}", ticket_payment);
        near_sdk::log!("Received Keypom Deposit: {}", keypom_deposit);

        let public_keys = new_keys.iter().map(|x| x.public_key.clone()).collect::<Vec<PublicKey>>();
        
        // Ticket limit exists, check
        if max_tickets.is_some(){
            ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
            .get_drop_information(drop_id.to_string())
            .then(
                Self::ext(env::current_account_id())
                .add_key_pre_check(drop_id.to_string(), new_keys, max_tickets.unwrap(), buyer_id, public_keys, return_amount, event_id.clone(), keypom_deposit, ticket_payment, ticket_price)
            );
        }else{
            // Get key's drop ID and then event, in order to modify all needed data
            ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
            .with_attached_deposit(keypom_deposit)
            .add_keys(drop_id.to_string(), new_keys, None)
            .then(
                 Self::ext(env::current_account_id())
                 .buy_initial_sale_callback(buyer_id, return_amount, event_id.clone(), keypom_deposit, ticket_payment, ticket_price)
             );
        }
        
    }

    // Ensure max tickets not yet reached
    #[private]
    pub fn add_key_pre_check(
        &mut self,
        drop_id: DropId,
        keys_vec: Vec<ExtKeyData>,
        max_tickets: u64,
        buyer_id: AccountId,
        public_keys: Vec<PublicKey>,
        return_amount: u128,
        event_id: EventID,
        keypom_deposit: u128,
        ticket_payment: u128,
        ticket_price: u128
    ){
        // Parse Response and Check if more tickets can still be sold
        if let PromiseResult::Successful(val) = env::promise_result(0){
            if let Ok(drop_info) = near_sdk::serde_json::from_slice::<ExtDrop>(&val) {
                let current_tickets = drop_info.next_key_id + 1;
                if (max_tickets - current_tickets) > public_keys.len() as u64 {
                    // transfer deposit back to sender, then panic
                }else{
                    // attach keypom storage deposit here!
                    ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                    .with_attached_deposit(keypom_deposit)
                    .add_keys(drop_id.to_string(), keys_vec, None)
                    .then(
                         Self::ext(env::current_account_id())
                         // send price and marketplace storage cost as args
                         .buy_initial_sale_callback(buyer_id, return_amount, event_id.clone(), keypom_deposit, ticket_payment, ticket_price)
                     );
                }
            } else {
                env::panic_str("Could not parse drop information from Keypom Contract");
            }
        }
        else{
            env::panic_str("Could not retrieve drop infromation from Keypom Contract!")
        }
    }

    #[private]
    pub fn buy_initial_sale_callback(
        &mut self,
        buyer_id: AccountId,
        return_amount: u128,
        event_id: EventID,
        keypom_deposit: u128,
        ticket_payment: u128,
        ticket_price: u128
    ) -> Promise {
        // Get key information and add to owned keys
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            if let Ok(result) = near_sdk::serde_json::from_slice::<bool>(&val) {
                if result{
                    // refund excess to buyer and send ticket price to funder
                    let funder = self.event_by_id.get(&event_id).unwrap().funder_id;
                    near_sdk::log!("Add Key Successful, transferring funds to funder and refunding excess to buyer");
                    Promise::new(buyer_id).transfer(return_amount);
                    Promise::new(funder).transfer(ticket_price).as_return()
                }else{
                    // transfer price and keypom deposit (everything) back to buyer
                    near_sdk::log!("Add Key Failed on Keypom Contract, refunding to buyer");
                    Promise::new(buyer_id).transfer(ticket_payment + keypom_deposit).as_return()
                }
            }else {
             env::panic_str("Could not parse add key bool response from Keypom contract");
            }      
        }
        else{
            env::panic_str("Add Key Failed!")
        }  
    }
    
    // Buy Resale
    #[payable]
    pub fn buy_resale(
        &mut self,
        // TODO: RECONSIDER THIS --> frontend will pass in key and dropId
        drop_id: DropId,
        // for-sale public key inside of memo
        memo: NftTransferMemo,
        new_public_key: PublicKey,
        new_owner: Option<AccountId>,
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Parse msg to get transfer information
        let event_id = self.event_by_drop_id.get(&drop_id).expect("No event found for drop");
        self.assert_resales_active(&event_id);

        let buyer_id = env::predecessor_account_id();
        
        // Ensure deposit will cover ticket price
        let ticket_payment = env::attached_deposit();
        let public_key = memo.linkdrop_pk.clone();
        let resale_info =  self.resales.get(&drop_id).expect("No resale for drop").get(&public_key).expect("No resale found for key");
        let ticket_price = resale_info.price;
        require!(ticket_payment.gt(&u128::from(ticket_price.clone())), "Not enough attached deposit to resale ticket!");
        
        require!(new_public_key != public_key, "New and old key cannot be the same");

        let approval_id = resale_info.approval_id;
        let seller_id = resale_info.seller_id;

        let pk_string = String::from(&public_key);
        near_sdk::log!("getting key information with {:?}", pk_string);
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .nft_transfer(new_owner.clone(), approval_id, serde_json::to_string(&memo).unwrap())
                       .then(
                            Self::ext(env::current_account_id())
                            .buy_resale_middle_callback(buyer_id, seller_id, u128::from(ticket_price), ticket_payment)
                        );
        
    }

    #[private]
    pub fn buy_resale_middle_callback(
        &mut self,
        buyer_id: AccountId,
        seller_id: AccountId,
        ticket_price: u128,
        ticket_payment: u128,
    ) -> Promise{
        if let PromiseResult::Successful(_val) = env::promise_result(0) {
            // Transfer ticket price to seller and excess to buyer
            near_sdk::log!("Add Key Successful, transferring funds to funder and refunding excess to buyer");
            let excess_payment = ticket_payment - ticket_price;
            Promise::new(buyer_id).transfer(excess_payment);
            Promise::new(seller_id).transfer(ticket_price).as_return()
        }
        else{
            // transfer price and keypom deposit (everything) back to buyer
            near_sdk::log!("Resale Purchase Failed due to NFT Transfer Failure, see Keypom Logs!");
            near_sdk::log!("Refunding to buyer");
            Promise::new(buyer_id).transfer(ticket_payment).as_return()
        }  
    }

}
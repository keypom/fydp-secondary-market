use core::panic;
use std::string;

use near_sdk::store::{key, vec};

use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {
    // Buy Initial Sale Ticket (add_key)
    #[payable]
    pub fn buy_initial_sale(&mut self, drop_id: DropId, new_keys: Vec<ExtKeyData>) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);
        let event_id = self
            .event_by_drop_id
            .get(&drop_id)
            .expect("No event found for drop");

        let event = self
            .event_by_id
            .get(&event_id)
            .expect("No event found for event ID");
        // Ensure event is active
        self.assert_event_active(&event_id);

        let buyer_id = env::predecessor_account_id();
        let stripe_purchase = env::predecessor_account_id() == self.stripe_account;

        // ensure no metadata is too long, to prevent draining funder balance
        for key in new_keys.iter() {
            if key.metadata.is_some() {
                require!(
                    key.metadata.clone().unwrap().len() as u64 <= self.max_metadata_bytes_per_key,
                    "Metadata too long, must be less than 100 characters"
                );
            }
        }

        // Get payment and ticket price
        let payment = env::attached_deposit();
        let single_ticket_price = u128::from(
            event
                .ticket_info
                .get(&drop_id.to_string())
                .expect("No ticket tier found for event")
                .price,
        );

        // get total key storage cost, to be paid by funder by taking it out of their payout
        let total_metadata_bytes = new_keys
            .iter()
            .map(|x| x.metadata.clone().unwrap_or("".to_string()).len() as u64)
            .sum::<u64>();
        let total_key_storage_bytes =
            new_keys.len() as u64 * self.base_key_storage_size + total_metadata_bytes;
        // Total key costs to be decremented from funder payout
        let total_keys_cost = total_key_storage_bytes as u128 * env::storage_byte_cost();
        near_sdk::log!(
            "Total Key Storage Cost to be passed to Keypom: {}",
            total_keys_cost
        );

        let mut total_ticket_price = 0 as u128;
        let mut return_amount = 0;
        let mut free_ticket = false;

        // Paid ticket
        if single_ticket_price.gt(&(0 as u128)) {
            total_ticket_price = single_ticket_price.clone() * new_keys.len() as u128;

            // Check if payment covers ticket price
            if !stripe_purchase {
                near_sdk::log!(
                    "Received Payment: {}, Ticket Total Price {} ",
                    payment.clone(),
                    total_ticket_price.clone()
                );
                require!(
                    payment.ge(&total_ticket_price.clone()),
                    "Payment does not cover ticket price!"
                );

                require!(total_ticket_price >= total_keys_cost, "Ticket Price cannot be lower than ticket cost! Reduce key metadata or contact event host to increase price");

                near_sdk::log!(
                    "Trying to purchase {} Tickets on drop ID {} at price of {} NEAR per Ticket",
                    new_keys.len(),
                    drop_id,
                    u128::from(single_ticket_price.clone())
                );
                near_sdk::log!("Received paymnet: {}", payment);

                // Get a return amount in case of over-payment
                return_amount = payment - total_ticket_price;
            } else {
                // ensure worker passed in enough NEAR to cover storage
                require!(
                    payment.ge(&total_keys_cost),
                    "Stripe worker attached deposit does not cover key storage price price!"
                );

                free_ticket = true;
                near_sdk::log!("Received Stripe Payment");
                near_sdk::log!(
                    "Trying to purchase {} Tickets on drop ID {} at price of {} NEAR per Ticket",
                    new_keys.len(),
                    drop_id,
                    u128::from(single_ticket_price.clone())
                );
            }
        } else {
            // Free Ticket
            free_ticket = true;
            require!(
                self.marketplace_balance.get(&event.funder_id).unwrap() >= total_keys_cost,
                "Funder does not have enough balance to cover key storage costs!"
            );

            // Pre-emptively decrement funder balance, then re-increment if add keys fails
            let funder_balance = self.marketplace_balance.get(&event.funder_id).unwrap();
            self.marketplace_balance.insert(
                &event.funder_id,
                &(funder_balance.clone() - total_keys_cost),
            );
        }

        let max_tickets = event
            .ticket_info
            .get(&drop_id.to_string())
            .unwrap()
            .max_tickets;
        // Ticket limit exists, check
        if max_tickets.is_some() {
            ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                .get_drop_information(drop_id.to_string())
                .then(Self::ext(env::current_account_id()).add_key_pre_check(
                    drop_id.to_string(),
                    new_keys,
                    max_tickets.unwrap(),
                    buyer_id,
                    return_amount,
                    event_id.clone(),
                    total_keys_cost,
                    payment,
                    total_ticket_price,
                    stripe_purchase,
                    free_ticket,
                ));
        } else {
            // Get key's drop ID and then event, in order to modify all needed data
            ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                .with_attached_deposit(total_keys_cost)
                .add_keys(drop_id.to_string(), new_keys, None)
                .then(
                    Self::ext(env::current_account_id()).buy_initial_sale_callback(
                        buyer_id,
                        return_amount,
                        event_id.clone(),
                        total_keys_cost,
                        payment,
                        total_ticket_price,
                        free_ticket,
                    ),
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
        return_amount: u128,
        event_id: EventID,
        total_keys_cost: u128,
        payment: u128,
        total_ticket_price: u128,
        stripe_purchase: bool,
        free_ticket: bool,
    ) {
        // Parse Response and Check if more tickets can still be sold
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            if let Ok(drop_info) = near_sdk::serde_json::from_slice::<ExtDrop>(&val) {
                let current_tickets = drop_info.next_key_id;
                if (max_tickets - current_tickets) < keys_vec.len() as u64 && !stripe_purchase {
                    // Maximum number of tickets reached, send deposit back to buyer
                    near_sdk::log!("Maximum Number of tickets reached!");
                    near_sdk::log!(
                        "Maximim Tickets: {}, Current Tickets: {}, Tried to add {} tickets",
                        max_tickets,
                        current_tickets,
                        keys_vec.len()
                    );

                    // If the ticket was free, re-incrment funder balance
                    if free_ticket {
                        let event = self.event_by_id.get(&event_id).unwrap();
                        let funder_balance =
                            self.marketplace_balance.get(&event.funder_id).unwrap();
                        self.marketplace_balance.insert(
                            &event.funder_id,
                            &(funder_balance.clone() + total_keys_cost),
                        );
                    }

                    // Refund buyer
                    Promise::new(buyer_id).transfer(payment).as_return();
                } else {
                    // Add keys with Keypom Deposit
                    ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                        .with_attached_deposit(total_keys_cost)
                        .add_keys(drop_id.to_string(), keys_vec, None)
                        .then(
                            Self::ext(env::current_account_id())
                                // send price and marketplace storage cost as args
                                .buy_initial_sale_callback(
                                    buyer_id,
                                    return_amount,
                                    event_id.clone(),
                                    total_keys_cost,
                                    payment,
                                    total_ticket_price,
                                    free_ticket,
                                ),
                        );
                }
            } else {
                env::panic_str("Could not parse drop information from Keypom Contract");
            }
        } else {
            env::panic_str("Could not retrieve drop infromation from Keypom Contract!")
        }
    }

    #[private]
    pub fn buy_initial_sale_callback(
        &mut self,
        buyer_id: AccountId,
        return_amount: u128,
        event_id: EventID,
        total_keys_cost: u128,
        payment: u128,
        total_ticket_price: u128,
        free_ticket: bool,
    ) -> Promise {
        // Add keys will panic if it fails
        if let PromiseResult::Successful(_val) = env::promise_result(0) {
            // refund excess to buyer and send ticket price to funder
            let funder = self.event_by_id.get(&event_id).unwrap().funder_id;
            near_sdk::log!(
                "Add Key Successful, transferring funds to funder and refunding excess to buyer"
            );
            if !free_ticket {
                Promise::new(buyer_id).transfer(return_amount);
                Promise::new(funder)
                    .transfer(total_ticket_price - total_keys_cost)
                    .as_return()
            } else {
                near_sdk::log!("Free Ticket, no need to transfer anything");
                Promise::new(funder).as_return()
            }
        } else {
            near_sdk::log!("Add Key Failed on Keypom Contract, refunding to buyer");

            // If the ticket was free, re-incrment funder balance
            if free_ticket {
                let event = self.event_by_id.get(&event_id).unwrap();
                let funder_balance = self.marketplace_balance.get(&event.funder_id).unwrap();
                self.marketplace_balance.insert(
                    &event.funder_id,
                    &(funder_balance.clone() + total_keys_cost),
                );
            }

            // Refund buyer
            Promise::new(buyer_id).transfer(payment).as_return()
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
        new_owner: Option<AccountId>,
    ) {
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Parse msg to get transfer information
        let event_id = self
            .event_by_drop_id
            .get(&drop_id)
            .expect("No event found for drop");
        self.assert_resales_active(&event_id);

        let buyer_id = env::predecessor_account_id();
        let stripe_purchase = env::predecessor_account_id() == self.stripe_account;

        // Ensure deposit will cover ticket price
        let ticket_payment = env::attached_deposit();
        let public_key = memo.linkdrop_pk.clone();
        let new_public_key = memo.new_public_key.clone();
        let resale_info = self
            .resales
            .get(&drop_id)
            .expect("No resale for drop")
            .get(&public_key)
            .expect("No resale found for key");
        let ticket_price = resale_info.price;

        if !stripe_purchase {
            require!(
                ticket_payment.ge(&u128::from(ticket_price.clone())),
                "Not enough attached deposit to resale ticket!"
            );
        }

        require!(
            new_public_key != public_key,
            "New and old key cannot be the same"
        );

        let approval_id = resale_info.approval_id;
        let seller_id = resale_info.seller_id;

        let pk_string = String::from(&public_key);
        near_sdk::log!("getting key information with {:?}", pk_string);
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
            .nft_transfer(
                new_owner.clone(),
                approval_id,
                serde_json::to_string(&memo).unwrap(),
            )
            .then(Self::ext(env::current_account_id()).buy_resale_callback(
                buyer_id,
                seller_id,
                u128::from(ticket_price),
                ticket_payment,
                drop_id,
                public_key.clone()
            ));
    }

    #[private]
    pub fn buy_resale_callback(
        &mut self,
        buyer_id: AccountId,
        seller_id: AccountId,
        ticket_price: u128,
        ticket_payment: u128,
        drop_id: DropId,
        old_public_key: PublicKey,
    ) -> Promise {
        if let PromiseResult::Successful(_val) = env::promise_result(0) {
            // Transfer ticket price to seller and excess to buyer
            let mut sale_binding = self.resales.get(&drop_id); 
            let sale = sale_binding.as_mut().unwrap();
            sale.remove(&old_public_key);
            self.resales.insert(&drop_id, &sale);
            near_sdk::log!(
                "Add Key Successful, transferring funds to funder and refunding excess to buyer"
            );
            let excess_payment = ticket_payment - ticket_price;
            Promise::new(buyer_id).transfer(excess_payment);
            Promise::new(seller_id).transfer(ticket_price).as_return()
        } else {
            // transfer price and keypom deposit (everything) back to buyer
            near_sdk::log!("Resale Purchase Failed due to NFT Transfer Failure, see Keypom Logs!");
            near_sdk::log!("Refunding to buyer");
            Promise::new(buyer_id).transfer(ticket_payment).as_return()
        }
    }
}


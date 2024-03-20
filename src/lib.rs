#![allow(unused_imports)]

pub mod balance;
pub mod buy;
pub mod costs;
pub mod ext_traits;
pub mod ext_types;
pub mod helper;
pub mod list;
pub mod models;
pub mod modify_event;
pub mod modify_resales;
pub mod owner;
pub mod types;
pub mod view;

pub use balance::*;
pub use buy::*;
pub use costs::*;
pub use ext_traits::*;
pub use ext_types::*;
pub use helper::*;
pub use list::*;
pub use models::*;
pub use modify_event::*;
pub use modify_resales::*;
pub use owner::*;
pub use types::*;
pub use view::*;

use ext_traits::ext_keypom;
use models::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PublicKey;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, CryptoHash, Gas, Promise, PromiseResult,
};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use types::*;

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);
pub const TGAS: u64 = 1_000_000_000_000;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// TODO: VERIFY PUBLIC-KEY VS TOKEN_ID ON KEYPOM SIDE, WHAT IS NEEDED?

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Marketplace {
    /// **************** Admin Stuff ****************
    /// Owner of the contract that can set configurations such as global freezes etc.
    pub contract_owner_id: AccountId,
    /// Whether or not the contract is frozen and no new drops can be created / keys added.
    pub global_freeze: bool,
    /// Base number of bytes for each key stored
    pub base_key_storage_size: u64,
    /// Maximum markup price, used to calculate resale ceiling upon event creation, in percentage (200 = 2x markup, 100 = 1x markup, etc.)
    pub max_markup: u64,
    /// Stripe account
    pub stripe_account: AccountId,
    /// Maximum metadata length per key, in bytes
    pub max_metadata_bytes_per_key: u64,

    /// **************** Keypom ****************
    pub keypom_contract: AccountId,

    /// **************** By Event ID ****************
    /// Event/Drop Information per Drop
    pub event_by_id: UnorderedMap<EventID, EventDetails>,

    /// **************** By Account ****************
    /// Stripe ID for event organizers
    pub stripe_id_per_account: LookupMap<AccountId, String>,
    /// Marketplace Balance
    pub marketplace_balance: LookupMap<AccountId, Balance>,

    /// **************** By Drop ****************
    /// Event ID given a drop ID
    pub event_by_drop_id: LookupMap<DropId, EventID>,
    /// Collection of keys that have been listed per drop
    pub resales: LookupMap<DropId, UnorderedMap<PublicKey, ResaleInfo>>,
}

impl Default for Marketplace {
    fn default() -> Self {
        Self {
            /// **************** Admin Stuff ****************
            contract_owner_id: AccountId::try_from("minqi.testnet".to_string()).unwrap(),
            global_freeze: false,
            max_markup: 150, // 1.5x markup
            base_key_storage_size: 684,
            // TODO: REFINE THIS
            max_metadata_bytes_per_key: u64::MAX,
            stripe_account: AccountId::try_from("mintlu.testnet".to_string()).unwrap(),
            /// **************** Keypom ****************
            keypom_contract: AccountId::try_from("testing-nearcon-keypom.testnet".to_string())
                .unwrap(),
            // **************** By Event ID ****************
            event_by_id: UnorderedMap::new(StorageKeys::EventInfoPerID),
            // **************** By Account ****************
            stripe_id_per_account: LookupMap::new(StorageKeys::StripeByAccountId),
            marketplace_balance: LookupMap::new(StorageKeys::MarketplaceBalanceByAccountId),
            // **************** By Drop ****************
            event_by_drop_id: LookupMap::new(StorageKeys::EventByDropId),
            resales: LookupMap::new(StorageKeys::ResalesPerDrop),
        }
    }
}

#[near_bindgen]
impl Marketplace {
    #[init]
    pub fn new(
        keypom_contract: Option<String>,
        stripe_account: Option<String>,
        contract_owner: Option<String>,
        max_metadata_bytes: Option<u64>,
        base_key_storage_size: Option<u64>,
    ) -> Self {
        Self {
            /// **************** Admin Stuff ****************
            contract_owner_id: AccountId::try_from(
                contract_owner.unwrap_or("minqi.testnet".to_string()),
            )
            .unwrap(),
            global_freeze: false,
            max_markup: 150, // 1.5x markup
            base_key_storage_size: base_key_storage_size.unwrap_or(1500),
            // TODO: REFINE THIS
            max_metadata_bytes_per_key: max_metadata_bytes.unwrap_or(684),
            stripe_account: AccountId::try_from(
                stripe_account.unwrap_or("kp-market-stripe.testnet".to_string()),
            )
            .unwrap(),
            /// **************** Keypom ****************
            keypom_contract: AccountId::try_from(
                keypom_contract.unwrap_or("1709145182592-kp-ticketing.testnet".to_string()),
            )
            .unwrap(),
            // **************** By Event ID ****************
            event_by_id: UnorderedMap::new(StorageKeys::EventInfoPerID),
            // **************** By Account ****************
            stripe_id_per_account: LookupMap::new(StorageKeys::StripeByAccountId),
            marketplace_balance: LookupMap::new(StorageKeys::MarketplaceBalanceByAccountId),
            // **************** By Drop ****************
            event_by_drop_id: LookupMap::new(StorageKeys::EventByDropId),
            resales: LookupMap::new(StorageKeys::ResalesPerDrop),
        }
    }

    /// Helper function to make sure there isn't a global freeze on the contract
    pub(crate) fn assert_no_global_freeze(&self) {
        if env::predecessor_account_id() != self.contract_owner_id {
            require!(
                self.global_freeze == false,
                "Contract is frozen and no new drops or keys can be created"
            );
        }
    }

    #[private]
    pub fn change_keypom_contract(&mut self, new_contract: AccountId) {
        self.keypom_contract = new_contract
    }

    #[private]
    pub fn change_stripe_account(&mut self, new_account: AccountId) {
        self.stripe_account = new_account
    }

    #[private]
    pub fn change_base_key_cost(&mut self, new_key_size: u64) {
        self.base_key_storage_size = new_key_size
    }

    #[private]
    pub fn change_max_metadata_bytes(&mut self, new_max: u64) {
        self.max_metadata_bytes_per_key = new_max
    }

    pub fn view_base_key_cost(&self) -> u64 {
        self.base_key_storage_size
    }

    pub fn view_max_metadata_bytes(&self) -> u64 {
        self.max_metadata_bytes_per_key
    }

    pub fn view_keypom_contract(&self) -> AccountId {
        self.keypom_contract.clone()
    }
}

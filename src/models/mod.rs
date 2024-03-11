use near_sdk::BorshStorageKey;

use crate::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    //AssetById { drop_id_hash: CryptoHash },
    //TokensPerOwnerInner { account_id_hash: CryptoHash },
    ResalePerEvent,
    EventInfoPerID,
    
    EventByDropId,
    KeysByDropId,
    
    StripeByAccountId,
    MarketplaceBalanceByAccountId,
    
    MaxPricePerKey,
    ApprovalIDByPk,
    ResalesPerDrop,
    // identifier_hash = hash(drop_id)
    ResalesPerDropInner { identifier_hash: CryptoHash },

    TicketInfoPerEvent,
    // identifier_hash = hash(event_id)
    TicketInfoPerEventInner { identifier_hash: CryptoHash },
}
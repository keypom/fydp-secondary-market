use near_sdk::PanicOnDefault;

use crate::*;

/// The ID for a given drop (this is the unique identifier for the drop and is how it will be referenced)
pub type DropId = String;
/// Which specific use is something being acted on. This is not zero indexed (i.e the first use is 1)
pub type UseNumber = u32;
/// ID for NFTs that have been sent to the Keypom contract as part of NFT assets
pub type TokenId = String;

/// The ID for a given event (this is the unique identifier for the drop and is how it will be referenced)
pub type EventID = String;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub struct ResaleInfo {
    pub price: U128,
    pub public_key: PublicKey,
    pub seller_id: AccountId,
    pub approval_id: Option<u64>,
    pub event_id: EventID,
    pub drop_id: DropId
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    Active,
    NoResales,
    Inactive,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ResaleStatus {
    Active,
    Inactive,
}


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct EventDetails {
    // Event host, same as drop funders
    pub funder_id: AccountId,
    // Event ID, in case on needing to abstract on contract to multiple drops per event
    // For now, event ID is drop ID
    pub event_id: String,
    // Event Status, can only be active or inactive
    pub status: Status,
    // Sale Information
    pub ticket_info: UnorderedMap<DropId, TicketInfo>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct ExtEventDetails{
    // Event host, same as drop funders
    pub funder_id: AccountId,
    // Event ID, in case on needing to abstract on contract to multiple drops per event
    // For now, event ID is drop ID
    pub event_id: String,
    // Event Status, can only be active or inactive
    pub status: Status,
    // Sale Information
    pub ticket_info: HashMap<DropId, TicketInfo>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
pub struct TicketInfo {
    // Maximum number of tickets
    pub max_tickets: Option<u64>,
    // Tiered Pricing?
    pub price: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OwnedTicket{
    pub public_key: PublicKey,
    pub event_id: EventID
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceivedResaleInfo {
    pub price: U128,
    pub public_key: PublicKey,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NftTransferMemo {
    pub linkdrop_pk: PublicKey,
    pub signature: Option<Base64VecU8>,
    pub new_public_key: PublicKey,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
// For each drop being added to event, the following bits of info are needed.
pub struct AddedDropDetails {
    // Maximum number of tickets
    pub max_tickets: Option<u64>,
    // Tiered Pricing?
    pub price_by_drop_id: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtDrop {
    /// ID for this specific drop
    pub drop_id: DropId,
    /// Account ID who funded / owns the rights to this specific drop
    pub funder_id: AccountId,

    /// Keep track of the next nonce to give out to a key
    pub next_key_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtKeyData {
    /// What is the public key?
    pub public_key: PublicKey,
    /// A map outlining what the password should be for any given use.
    /// The password here should be a double hash and when claim is called,
    /// The user arguments are hashed and compared to the password here (i.e user passes in single hash)
    pub password_by_use: Option<HashMap<UseNumber, String>>,
    /// Metadata for the given key represented as a string. Most often, this will be JSON stringified.
    pub metadata: Option<String>,
    /// What account ID owns the given key (if any)
    pub key_owner: Option<AccountId>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtKeyInfo {
    /// How much Gas should be attached when the key is used to call `claim` or `create_account_and_claim`.
   /// It is up to the smart contract developer to calculate the required gas (which can be done either automatically on the contract or on the client-side).
   pub required_gas: String,

   /// yoctoNEAR$ amount that will be sent to the account that claims the linkdrop (either new or existing)
   /// when the key is successfully used.
   pub yoctonear: U128,

   /* CUSTOM */
   pub drop_id: DropId,
   pub pub_key: PublicKey,
   pub token_id: TokenId,
   pub owner_id: AccountId,
   
   pub uses_remaining: UseNumber
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtNFTKey {
    //token ID
    pub token_id: TokenId,
    //owner of the token
    pub owner_id: AccountId,
    //token metadata
    pub metadata: TokenMetadata,
    //list of approved account IDs that have access to transfer the token. This maps an account ID to an approval ID
    pub approved_account_ids: HashMap<AccountId, u64>,
    //keep track of the royalty percentages for the token in a hash map
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<u64>, // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>, // When token expires, Unix epoch in milliseconds
    pub starts_at: Option<u64>, // When token starts being valid, Unix epoch in milliseconds
    pub updated_at: Option<u64>, // When token was last updated, Unix epoch in milliseconds
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Injected Keypom Args struct to be sent to external contracts
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct KeypomInjectedArgs {
    /// Specifies what field the claiming account ID should go in when calling the function
    /// If None, this isn't attached to the args
    pub account_id_field: Option<String>,
    /// Specifies what field the drop ID should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    /// If Some(String), attach drop ID to args. Else, don't attach.
    pub drop_id_field: Option<String>,
    /// Specifies what field the key ID should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    /// If Some(String), attach key ID to args. Else, don't attach.
    pub key_id_field: Option<String>,
    // Specifies what field the funder id should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    // If Some(string), attach the funder ID to the args. Else, don't attach.
    pub funder_id_field: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
/// When a user provides arguments for FC drops in `claim` or `create_account_and_claim`, what behaviour is expected?
/// For `AllUser`, any arguments provided by the user will completely overwrite any previous args provided by the drop creator.
/// For `FunderPreferred`, any arguments provided by the user will be concatenated with the arguments provided by the drop creator. If there are any duplicate args, the drop funder's arguments will be used.
/// For `UserPreferred`, any arguments provided by the user will be concatenated with the arguments provided by the drop creator, but if there are any duplicate keys, the user's arguments will overwrite the drop funder's.
pub enum UserArgsRule {
    AllUser,
    FunderPreferred,
    UserPreferred
}
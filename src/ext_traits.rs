
use near_sdk::ext_contract;


use crate::*;


#[ext_contract(ext_keypom)]
trait ExtKeypom{

    fn add_keys(&mut self, drop_id: DropId, key_data: Vec<ExtKeyData>, keep_excess_deposit: Option<bool>) -> bool;

    fn get_key_information(&self, key: String) -> Result<ExtKeyInfo, String>;

    fn nft_token(&self, token_id: TokenId) -> Option<ExtNFTKey>;

    // memo contains NftTrasnferMemo
    fn nft_transfer(&mut self, receiver_id: Option<AccountId>, approval_id: Option<u64>, memo: String);

    fn get_drop_information(&self, drop_id: DropId) -> ExtDrop;
}

#[ext_contract(ext_v2_keypom)]
trait ExtV2Keypom{
    fn create_drop(&mut self, public_keys: Option<Vec<PublicKey>>, deposit_per_use: U128, drop_id: Option<DropIdJson>) -> Option<DropIdJson>;
}


// #[ext_contract(ext_self)]
// trait ContractExt{
//     fn get_roles_callback(&self);
// }



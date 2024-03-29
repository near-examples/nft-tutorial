use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: Option<TokenId>,
        metadata: TokenMetadata,
        receiver_id: Option<AccountId>,
    ) {
        /*
            FILL THIS IN
        */
    }
}
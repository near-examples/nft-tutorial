use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: Option<TokenId>,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) {
        /*
            FILL THIS IN
        */
    }
}

use crate::*;

//used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

impl Contract {
    //internal method for removing a sale from the market. This returns the previously removed sale object
    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {
        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        //get the sale object by removing the unique sale ID. If there was no sale, panic
        let sale = self.sales.remove(&contract_and_token_id).expect("No sale");

        //get the set of sales for the sale's owner. If there's no sale, panic. 
        let mut by_owner_id = self.by_owner_id.get(&sale.owner_id).expect("No sale by_owner_id");
        //remove the unique sale ID from the set of sales
        by_owner_id.remove(&contract_and_token_id);
        
        //if the set of sales is now empty after removing the unique sale ID, we simply remove that owner from the map
        if by_owner_id.is_empty() {
            self.by_owner_id.remove(&sale.owner_id);
        //if the set of sales is not empty after removing, we insert the set back into the map for the owner
        } else {
            self.by_owner_id.insert(&sale.owner_id, &by_owner_id);
        }

        //get the set of token IDs for sale for the nft contract ID. If there's no sale, panic. 
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .expect("No sale by nft_contract_id");
        
        //remove the token ID from the set 
        by_nft_contract_id.remove(&token_id);
        
        //if the set is now empty after removing the token ID, we remove that nft contract ID from the map
        if by_nft_contract_id.is_empty() {
            self.by_nft_contract_id.remove(&nft_contract_id);
        //if the set is not empty after removing, we insert the set back into the map for the nft contract ID
        } else {
            self.by_nft_contract_id
                .insert(&nft_contract_id, &by_nft_contract_id);
        }

        //return the sale object
        sale
    }
}

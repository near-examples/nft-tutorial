use crate::*;

#[near_bindgen]
impl Contract {
    /// views
    
    //returns the number of sales the marketplace has up (as a string)
    pub fn get_supply_sales(
        &self,
    ) -> U64 {
        //returns the sales object length wrapped as a U64
        U64(self.sales.len())
    }
    
    //returns the number of sales for a given account (result is a string)
    pub fn get_supply_by_owner_id(
        &self,
        account_id: AccountId,
    ) -> U64 {
        //get the set of sales for the given owner Id
        let by_owner_id = self.by_owner_id.get(&account_id);
        
        //if there as some set, we return the length but if there wasn't a set, we return 0
        if let Some(by_owner_id) = by_owner_id {
            U64(by_owner_id.len())
        } else {
            U64(0)
        }
    }

    //returns paginated sale objects for a given account. (result is a vector of sales)
    pub fn get_sales_by_owner_id(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Sale> {
        //get the set of token IDs for sale for the given account ID
        let by_owner_id = self.by_owner_id.get(&account_id);
        //if there was some set, we set the sales variable equal to that set. If there wasn't, sales is set to an empty vector
        let sales = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            return vec![];
        };
        
        //we'll convert the UnorderedSet into a vector of strings
        let keys = sales.as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0)));
        
        //iterate through the keys vector
        keys.iter()
            //skip to the index we specified in the start variable
            .skip(start as usize) 
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 0
            .take(limit.unwrap_or(0) as usize) 
            //we'll map the token IDs which are strings into Sale objects
            .map(|token_id| self.sales.get(&token_id).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    //get the number of sales for an nft contract. (returns a string)
    pub fn get_supply_by_nft_contract_id(
        &self,
        nft_contract_id: AccountId,
    ) -> U64 {
        //get the set of tokens for associated with the given nft contract
        let by_nft_contract_id = self.by_nft_contract_id.get(&nft_contract_id);
        
        //if there was some set, return it's length. Otherwise return 0
        if let Some(by_nft_contract_id) = by_nft_contract_id {
            U64(by_nft_contract_id.len())
        } else {
            U64(0)
        }
    }

    //returns paginated sale objects associated with a given nft contract. (result is a vector of sales)
    pub fn get_sales_by_nft_contract_id(
        &self,
        nft_contract_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Sale> {
        //get the set of token IDs for sale for the given contract ID
        let by_nft_contract_id = self.by_nft_contract_id.get(&nft_contract_id);
        
        //if there was some set, we set the sales variable equal to that set. If there wasn't, sales is set to an empty vector
        let sales = if let Some(by_nft_contract_id) = by_nft_contract_id {
            by_nft_contract_id
        } else {
            return vec![];
        };

        //we'll convert the UnorderedSet into a vector of strings
        let keys = sales.as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0)));
        
        //iterate through the keys vector
        keys.iter()
            //skip to the index we specified in the start variable
            .skip(start as usize) 
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 0
            .take(limit.unwrap_or(0) as usize) 
            //we'll map the token IDs which are strings into Sale objects by passing in the unique sale ID (contract + DELIMITER + token ID)
            .map(|token_id| self.sales.get(&format!("{}{}{}", nft_contract_id, DELIMETER, token_id)).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    //get a sale information for a given unique sale ID (contract + DELIMITER + token ID)
    pub fn get_sale(&self, nft_contract_token: ContractAndTokenId) -> Option<Sale> {
        //try and get the sale object for the given unique sale ID. Will return an option since
        //we're not guaranteed that the unique sale ID passed in will be valid.
        self.sales.get(&nft_contract_token)
    }
}

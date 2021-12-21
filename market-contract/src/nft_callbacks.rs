use crate::*;

/// approval callbacks from NFT Contracts

//struct for keeping track of the sale conditions for a Sale
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SalePriceInYoctoNear,
}

/*
    trait that will be used as the callback from the NFT contract. When nft_approve is
    called, it will fire a cross contract call to this marketplace and this is the function
    that is invoked. 
*/
trait NonFungibleTokenApprovalsReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

//implementation of the trait
#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for Contract {
    /// where we add the sale because we know nft owner can only call nft_approve

    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        // get the contract ID which is the predecessor
        let nft_contract_id = env::predecessor_account_id();
        //get the signer which is the person who initiated the transaction
        let signer_id = env::signer_account_id();

        //make sure that the signer isn't the predecessor. This is so that we're sure
        //this was called via a cross-contract call
        assert_ne!(
            nft_contract_id,
            signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        //make sure the owner ID is the signer. 
        assert_eq!(
            owner_id,
            signer_id,
            "owner_id should be signer_id"
        );

        //we need to enforce that the user has enough storage for 1 EXTRA sale.  

        //get the storage for a sale. dot 0 converts from U128 to u128
        let storage_amount = self.storage_minimum_balance().0;
        //get the total storage paid by the owner
        let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
        //get the storage required which is simply the storage for the number of sales they have + 1 
        let signer_storage_required = (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        
        //make sure that the total paid is >= the required storage
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage, signer_storage_required / STORAGE_PER_SALE, STORAGE_PER_SALE
        );

        //if all these checks pass we can create the sale conditions object.
        let SaleArgs { sale_conditions } =
            //the sale conditions come from the msg field. The market assumes that the user passed
            //in a proper msg. If they didn't, it panics. 
            near_sdk::serde_json::from_str(&msg).expect("Not valid SaleArgs");

        //create the unique sale ID which is the contract + DELIMITER + token ID
        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
        
        //insert the key value pair into the sales map. Key is the unique ID. value is the sale object
        self.sales.insert(
            &contract_and_token_id,
            &Sale {
                owner_id: owner_id.clone(), //owner of the sale / token
                approval_id, //approval ID for that token that was given to the market
                nft_contract_id: nft_contract_id.to_string(), //NFT contract the token was minted on
                token_id: token_id.clone(), //the actual token ID
                sale_conditions, //the sale conditions 
           },
        );

        //Extra functionality that populates collections necessary for the view calls 

        //get the sales by owner ID for the given owner. If there are none, we create a new empty set
        let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByOwnerIdInner {
                    //we get a new unique prefix for the collection by hashing the owner
                    account_id_hash: hash_account_id(&owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
        
        //insert the unique sale ID into the set
        by_owner_id.insert(&contract_and_token_id);
        //insert that set back into the collection for the owner
        self.by_owner_id.insert(&owner_id, &by_owner_id);

        //get the token IDs for the given nft contract ID. If there are none, we create a new empty set
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByNFTContractIdInner {
                        //we get a new unique prefix for the collection by hashing the owner
                        account_id_hash: hash_account_id(&nft_contract_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        
        //insert the token ID into the set
        by_nft_contract_id.insert(&token_id);
        //insert the set back into the collection for the given nft contract ID
        self.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);
    }
}

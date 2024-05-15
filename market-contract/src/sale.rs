use crate::*;
use near_sdk::{log, promise_result_as_success, NearSchema, PromiseError};
use near_sdk::serde_json::json;

//struct that holds important information about each sale on the market
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    //owner of the sale
    pub owner_id: AccountId,
    //market contract's approval ID to transfer the token on behalf of the owner
    pub approval_id: u32,
    //nft contract where the token was minted
    pub nft_contract_id: String,
    //actual token ID for sale
    pub token_id: String,
    //sale price in yoctoNEAR that the token is listed for
    pub sale_conditions: SalePriceInYoctoNear,
}

//The Json token is what will be returned from view calls. 
#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    //owner of the token
    pub owner_id: AccountId,
}

#[near_bindgen]
impl Contract {
    // lists a nft for sale on the market
    #[payable]
    pub fn list_nft_for_sale(
      &mut self,
      nft_contract_id: AccountId,
      token_id: TokenId,
      approval_id: u32,
      sale_conditions: SalePriceInYoctoNear,
    ) {
        let owner_id = env::predecessor_account_id();

        //we need to enforce that the user has enough storage for 1 EXTRA sale.

        //get the storage for a sale
        let storage_amount = self.storage_minimum_balance();
        //get the total storage paid by the owner
        let owner_paid_storage = self.storage_deposits.get(&owner_id).unwrap_or(ZERO_NEAR);
        //get the storage required which is simply the storage for the number of sales they have + 1 
        let signer_storage_required = storage_amount.saturating_mul((self.get_supply_by_owner_id(owner_id.clone()).0 + 1).into());
        
        //make sure that the total paid is >= the required storage
        assert!(
            owner_paid_storage.ge(&signer_storage_required),
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage, signer_storage_required.saturating_div(storage_per_sale().as_yoctonear()), storage_per_sale()
        );

        let nft_token_promise = Promise::new(nft_contract_id.clone()).function_call(
          "nft_token".to_owned(),
          json!({ "token_id": token_id }).to_string().into_bytes(),
          ZERO_NEAR,
          Gas::from_gas(10u64.pow(13))
        );
        let nft_is_approved_promise = Promise::new(nft_contract_id.clone()).function_call(
          "nft_is_approved".to_owned(),
          json!({ "token_id": token_id, "approved_account_id": env::current_account_id(), "approval_id": approval_id }).to_string().into_bytes(),
          ZERO_NEAR,
          Gas::from_gas(10u64.pow(13))
        );
        nft_token_promise
          .and(nft_is_approved_promise)
          .then(Self::ext(env::current_account_id()).process_listing(owner_id.clone(), nft_contract_id, token_id, approval_id, sale_conditions));
    }

    //removes a sale from the market. 
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        //get the sale object as the return value from removing the sale internally
        let sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
        //get the predecessor of the call and make sure they're the owner of the sale
        let owner_id = env::predecessor_account_id();
        //if this fails, the remove sale will revert
        assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
    }

    //updates the price for a sale on the market
    #[payable]
    pub fn update_price(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        price: NearToken,
    ) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        
        //create the unique sale ID from the nft contract and token
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If there is no token, panic. 
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");

        //assert that the caller of the function is the sale owner
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        
        //set the sale conditions equal to the passed in price
        sale.sale_conditions = price;
        //insert the sale back into the map for the unique sale ID
        self.sales.insert(&contract_and_token_id, &sale);
    }

    //place an offer on a specific sale. The sale will go through as long as your deposit is greater than or equal to the list price
    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
        //get the attached deposit and make sure it's greater than 0
        let deposit = env::attached_deposit();
        assert!(!deposit.is_zero(), "Attached deposit must be greater than 0");

        //convert the nft_contract_id from a AccountId to an AccountId
        let contract_id: AccountId = nft_contract_id.into();
        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
        let sale = self.sales.get(&contract_and_token_id).expect("No sale");
        
        //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
        let buyer_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");

        let price = sale.sale_conditions;

        //make sure the deposit is greater than the price
        assert!(deposit.ge(&price), "Attached deposit must be greater than or equal to the current price: {:?}. Your deposit: {:?}", price, deposit);

        //process the purchase (which will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties) 
        self.process_purchase(
            contract_id,
            token_id,
            deposit,
            buyer_id,
        );
    }

    //private function used when a sale is purchased. 
    //this will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties
    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        price: NearToken,
        buyer_id: AccountId,
    ) -> Promise {
        //get the sale object by removing the sale
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        //initiate a cross contract call to the nft contract. This will transfer the token to the buyer and return
        //a payout object used for the market to distribute funds to the appropriate accounts.
        ext_contract::ext(nft_contract_id)
            // Attach 1 yoctoNEAR with static GAS equal to the GAS for nft transfer. Also attach an unused GAS weight of 1 by default.
            .with_attached_deposit(ONE_YOCTONEAR)
            .with_static_gas(GAS_FOR_NFT_TRANSFER)
            .nft_transfer_payout(
                buyer_id.clone(), //purchaser (person to transfer the NFT to)
                token_id, //token ID to transfer
                sale.approval_id, //market contract's approval ID in order to transfer the token on behalf of the owner
            "payout from market".to_string(), //memo (to include some context)
            /*
                the price that the token was purchased for. This will be used in conjunction with the royalty percentages
                for the token in order to determine how much money should go to which account. 
            */
            price,
            10, //the maximum amount of accounts the market can payout at once (this is limited by GAS)
            )
        //after the transfer payout has been initiated, we resolve the promise by calling our own resolve_purchase function. 
        //resolve purchase will take the payout object returned from the nft_transfer_payout and actually pay the accounts
        .then(
            // No attached deposit with static GAS equal to the GAS for resolving the purchase. Also attach an unused GAS weight of 1 by default.
            Self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_RESOLVE_PURCHASE)
            .resolve_purchase(
                buyer_id, //the buyer and price are passed in incase something goes wrong and we need to refund the buyer
                price,
            )
        )
    }

    /*
        private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
        check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
        it will refund the buyer for the price. 
    */
    #[private]
    pub fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: NearToken,
    ) -> NearToken {
        // checking for payout information returned from the nft_transfer_payout method
        let payout_option = promise_result_as_success().and_then(|value| {
            //if we set the payout_option to None, that means something went wrong and we should refund the buyer
            near_sdk::serde_json::from_slice::<Payout>(&value)
                //converts the result to an optional value
                .ok()
                //returns None if the none. Otherwise executes the following logic
                .and_then(|payout_object| {
                    //we'll check if length of the payout object is > 10 or it's empty. In either case, we return None
                    if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
                        env::log_str("Cannot have more than 10 royalties");
                        None
                    
                    //if the payout object is the correct length, we move forward
                    } else {
                        //we'll keep track of how much the nft contract wants us to payout. Starting at the full price payed by the buyer
                        let mut remainder = price;
                        
                        //loop through the payout and subtract the values from the remainder. 
                        for &value in payout_object.payout.values() {
                            //checked sub checks for overflow or any errors and returns None if there are problems
                            remainder = remainder.checked_sub(value)?;
                        }
                        //Check to see if the NFT contract sent back a faulty payout that requires us to pay more or too little. 
                        //The remainder will be 0 if the payout summed to the total price. The remainder will be 1 if the royalties
                        //we something like 3333 + 3333 + 3333. 
                        if remainder.eq(&ZERO_NEAR) || remainder.eq(&NearToken::from_yoctonear(1)) {
                            //set the payout_option to be the payout because nothing went wrong
                            Some(payout_object.payout)
                        } else {
                            //if the remainder was anything but 1 or 0, we return None
                            None
                        }
                    }
                })
        });

        // if the payout option was some payout, we set this payout variable equal to that some payout
        let payout = if let Some(payout_option) = payout_option {
            payout_option
        //if the payout option was None, we refund the buyer for the price they payed and return
        } else {
            Promise::new(buyer_id).transfer(price);
            // leave function and return the price that was refunded
            return price;
        };

        // NEAR payouts
        for (receiver_id, amount) in payout {
            Promise::new(receiver_id).transfer(amount);
        }

        //return the price payout out
        price
    }

    #[private]
    pub fn process_listing(
        &mut self,
        owner_id: AccountId,
        nft_contract_id: AccountId,
        token_id: TokenId,
        approval_id: u32,
        sale_conditions: SalePriceInYoctoNear,
        #[callback_result] nft_token_result: Result<JsonToken, PromiseError>,
        #[callback_result] nft_is_approved_result: Result<bool, PromiseError>,
    ) {
        if let Ok(result) = nft_token_result {
            assert_eq!(
                result.owner_id,
                owner_id,
                "Signer is not NFT owner",
            )
        } else {
            log!("nft_is_approved call failed");
        }
        if let Ok(result) = nft_is_approved_result {
            assert_eq!(
                result,
                true,
                "Marketplace contract is not approved",
            )
        } else {
            log!("nft_is_approved call failed");
        } 
    
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
                )
            });
        
        //insert the token ID into the set
        by_nft_contract_id.insert(&token_id);
        //insert the set back into the collection for the given nft contract ID
        self.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);
    }
}

//this is the cross contract call that we call on our own contract. 
/*
    private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
    check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
    it will refund the buyer for the price. 
*/
#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: NearToken,
    ) -> Promise;
}
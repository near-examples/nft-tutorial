use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, BorshStorageKey, CryptoHash, Gas,
    NearToken, PanicOnDefault, Promise,
};
use std::collections::HashMap;

use crate::external::*;
use crate::internal::*;
use crate::sale::*;

mod external;
mod internal;
mod nft_callbacks;
mod sale;
mod sale_views;

//GAS constants to attach to calls
const GAS_FOR_RESOLVE_PURCHASE: Gas = Gas::from_tgas(115);
const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(15);

//Basic NEAR amounts as constants
const ZERO_NEAR: NearToken = NearToken::from_yoctonear(0);
const ONE_YOCTONEAR: NearToken = NearToken::from_yoctonear(1);

//every sale will have a unique ID which is `CONTRACT + DELIMITER + TOKEN_ID`
static DELIMETER: &str = ".";

//Creating custom types to use within the contract. This makes things more readable.
pub type SalePriceInYoctoNear = NearToken;
pub type TokenId = String;
pub type FungibleTokenId = AccountId;
pub type ContractAndTokenId = String;

//defines the payout type we'll be parsing from the NFT contract as a part of the royalty standard.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, NearToken>,
}

//main contract struct to store all the information
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    //keep track of the owner of the contract
    pub owner_id: AccountId,

    /*
        to keep track of the sales, we map the ContractAndTokenId to a Sale.
        the ContractAndTokenId is the unique identifier for every sale. It is made
        up of the `contract ID + DELIMITER + token ID`
    */
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,

    //keep track of all the Sale IDs for every account ID
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,

    //keep track of all the token IDs for sale for a given contract
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keep track of the storage that accounts have payed
    pub storage_deposits: LookupMap<AccountId, NearToken>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default data and the owner ID
        that's passed in
    */
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let this = Self {
            //set the owner_id field equal to the passed in owner_id.
            owner_id,

            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            sales: UnorderedMap::new(StorageKey::Sales),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
        };

        //return the Contract object
        this
    }

    //Allows users to deposit storage. This is to cover the cost of storing sale objects on the contract
    //Optional account ID is to users can pay for storage for other people.
    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        //get the account ID to pay for storage for
        let storage_account_id = account_id
            //convert the valid account ID into an account ID
            .map(|a| a.into())
            //if we didn't specify an account ID, we simply use the caller of the function
            .unwrap_or_else(env::predecessor_account_id);

        //get the deposit value which is how much the user wants to add to their storage
        let deposit = env::attached_deposit();

        //make sure the deposit is greater than or equal to the minimum storage for a sale (which computes like env::storage_byte_cost().saturating_mul(1000))
        assert!(
            deposit.ge(&storage_per_sale()),
            "Requires minimum deposit of {}",
            storage_per_sale()
        );

        //get the balance of the account (if the account isn't in the map we default to a balance of 0)
        let mut balance: NearToken = self
            .storage_deposits
            .get(&storage_account_id)
            .unwrap_or(ZERO_NEAR);
        //add the deposit to their balance
        balance = balance.saturating_add(deposit);
        //insert the balance back into the map for that account ID
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    //Allows users to withdraw any excess storage that they're not using. Say Bob pays 0.01N for 1 sale
    //Alice then buys Bob's token. This means bob has paid 0.01N for a sale that's no longer on the marketplace
    //Bob could then withdraw this 0.01N back into his account.
    #[payable]
    pub fn storage_withdraw(&mut self) {
        //make sure the user attaches exactly 1 yoctoNEAR for security purposes.
        //this will redirect them to the NEAR wallet (or requires a full access key).
        assert_one_yocto();

        //the account to withdraw storage to is always the function caller
        let owner_id = env::predecessor_account_id();
        //get the amount that the user has by removing them from the map. If they're not in the map, default to 0
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(ZERO_NEAR);

        //how many sales is that user taking up currently. This returns a set
        let sales = self.by_owner_id.get(&owner_id);
        //get the length of that set.
        let len = sales.map(|s| s.len()).unwrap_or_default();
        //how much NEAR is being used up for all the current sales on the account
        let diff = storage_per_sale().saturating_mul(u128::from(len));

        //the excess to withdraw is the total storage paid - storage being used up.
        amount = amount.saturating_sub(diff);

        //if that excess to withdraw is > 0, we transfer the amount to the user.
        if amount.gt(&ZERO_NEAR) {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        //we need to add back the storage being used up into the map if it's greater than 0.
        //this is so that if the user had 500 sales on the market, we insert that value here so
        //if those sales get taken down, the user can then go and withdraw 500 sales worth of storage.
        if diff.gt(&ZERO_NEAR) {
            self.storage_deposits.insert(&owner_id, &diff);
        }
    }

    /// views
    //return the minimum storage for 1 sale
    pub fn storage_minimum_balance(&self) -> NearToken {
        storage_per_sale()
    }

    //return how much storage an account has paid for
    pub fn storage_balance_of(&self, account_id: AccountId) -> NearToken {
        self.storage_deposits.get(&account_id).unwrap_or(ZERO_NEAR)
    }
}

#[cfg(test)]
mod tests;

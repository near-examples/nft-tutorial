use std::collections::HashMap;
use near_sdk::borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
  env, near_bindgen, AccountId, NearToken, CryptoHash, PanicOnDefault, Promise, PromiseOrValue, BorshStorageKey, NearSchema
};

pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::events::*;
pub use crate::approval::*;
pub use crate::royalty::*;

mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod events;
mod approval; 
mod royalty; 

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /*
        FILL THIS IN
    */
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }
}
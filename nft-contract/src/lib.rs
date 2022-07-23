use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet, LookupSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Type {
    nonce: u64,
    name: String,
    royalty: Option<HashMap<AccountId, u32>>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //approved minters (should never be large)
    pub approved_minters: LookupSet<AccountId>,

    //types
    pub types_by_id: UnorderedMap<u64, Type>,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: UnorderedMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: LookupMap<u64, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    ApprovedMinters,
    TypesByDropId,
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
        //calls the other function "new: with some default metadata and the owner_id passed in 
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "NFT Tutorial Contract".to_string(),
                symbol: "GOTEAM".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        //create a variable of type Self with all the fields initialized.
        let mut approved_minters = LookupSet::new(StorageKey::ApprovedMinters.try_to_vec().unwrap());
        approved_minters.insert(&owner_id);
        
        let this = Self {
            approved_minters,
            types_by_id: UnorderedMap::new(StorageKey::TypesByDropId.try_to_vec().unwrap()),
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: UnorderedMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: LookupMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            //set the &owner_id field equal to the passed in owner_id. 
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
        };

        //return the Contract object
        this
    }

    /// approved minters
    pub fn add_approved_minter(&mut self, minter_id: AccountId) {
        self.assert_contract_owner();
        self.approved_minters.insert(&minter_id);
    }

    pub fn remove_approved_minter(&mut self, minter_id: AccountId) {
        self.assert_contract_owner();
        self.approved_minters.remove(&minter_id);
    }

    pub fn is_approved_minter(&self, minter_id: AccountId) -> bool {
        self.approved_minters.contains(&minter_id)
    }

    /// types
    pub fn add_type(
        &mut self,
        name: String,
        media: String,
        type_id: u64,
        royalty: Option<HashMap<AccountId, u32>>
    ) {
        self.assert_contract_owner();
        self.types_by_id.insert(&type_id, &Type{
            nonce: 1,
            name,
            //we add an optional parameter for perpetual royalties
            royalty,
        });
        //insert the metadata (now it's just there for type_id
        self.token_metadata_by_id.insert(&type_id, &TokenMetadata {
            title: None, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: None, // free-form description
            media: Some(media), // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: None, // number of copies of this set of metadata in existence when token was minted.
            issued_at: None, // When token was issued or minted, Unix epoch in milliseconds
            expires_at: None, // When token expires, Unix epoch in milliseconds
            starts_at: None, // When token starts being valid, Unix epoch in milliseconds
            updated_at: None, // When token was last updated, Unix epoch in milliseconds
            extra: None, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference: None, // URL to an off-chain JSON file with more info.
            reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        });
    }
}

#[cfg(test)]
mod tests;
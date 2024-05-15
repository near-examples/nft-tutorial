use near_sdk::NearToken;

use crate::*;
pub type TokenId = String;
//defines the payout type we'll be returning as a part of the royalty standards.
#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, NearToken>,
} 

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    /*
        FILL THIS IN
    */
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    /*
        FILL THIS IN
    */
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Token {
    /*
        FILL THIS IN
    */
}

//The Json token is what will be returned from view calls. 
#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    /*
        FILL THIS IN
    */
}

pub trait NonFungibleTokenMetadata {
    //view call for returning the contract metadata
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }
}
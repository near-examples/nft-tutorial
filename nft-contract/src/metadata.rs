use crate::*;
use std::net::Ipv4Addr; // Getting Ipv4 addresses as metadata for the bbadge
pub type TokenId = String;
//defines the payout type we'll be returning as a part of the royalty standards.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String, // required, essentially a version like "cableguard-0.0.1"
    pub name: String, // required, ex. "CableGuard"
    pub symbol: String, // required, ex. "CBG"
    pub icon: Option<String>,      // Data URL // Empty for the current spec
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs // Empty for all spec versions
    pub reference: Option<String>, // URL to a JSON file with more info // Empty for all spec versions
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included. // Empty for all spec versions as pub reference is empty
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    // Boilerplate as example of a contract from 1.skeleton branch
    pub title: Option<String>, // Name of the Author
    pub description: Option<String>, // Description of the subscription
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage // Empty for this spec
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included. // Empty for this spec as pub media is empty
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted. // Probably 1 but remains to be seen if limited editions are necessary,
    //  as each token can only live in one address
    pub issued_at: Option<u64>, // When CBG bbadge was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>, // When CBG bbadge  expires, Unix epoch in milliseconds
    pub starts_at: Option<u64>, // When CBG bbadge starts being valid, Unix epoch in milliseconds
    pub updated_at: Option<u64>, // When token was last updated, Unix epoch in milliseconds // Empy for the current spec
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON. // Empty for the current spec
    pub reference: Option<String>, // URL to an off-chain JSON file with more info. //Empty for the current spec
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.  // Empty for the current spec as pub reference is empty
    // Following parameters belong to the wireguard config file section [Interface]
    pub address: Option<Ipv4Addr>, // IPAdress
    pub listenport: Option<u16>, // Port number
    pub privatekey: Option<String>, //  Ending in "="
    pub postup: Option<String>,  // ip command
    pub postdown: Option<String>, // ip command
    pub predown: Option<String>, // ip command
    pub vpnaccelerator: Option<String>, // Only on OR off
    pub saveconfig: Option<String>, // Only true OR false
    // Following parameters belong to the wireguard config file section [Peer]
    pub publickey: Option<String>, // Ending in =
    pub allowedips: Option<Ipv4Addr>, //  IPAdress range?
    pub dns: Option<Ipv4Addr>, // DNS IPAdress
    pub endpoint: Option<Ipv4Addr>, // IPAddress and port
    // Following parameters are part of the cableguard spec specific section [Signature] The enable the author of a CBG bbadge verify authorship
    pub authornftcontractid: Option<String>, // ID of authornftcontract
    pub authorsignature: Option<String>,  //  Hash of the bbadge signed with authornftcontractid's publickey
    pub kbpersecond: Option<u64>, // Bandwith of the subscription in Kb/s
    pub requestspersecond: Option<u64>, // Requests per second of the subscription
    pub authorizedlocation: Option<String>, // From what region the subscription is valid
    pub authorizednetwork: Option<Ipv4Addr>, // From what network range the subscription is valid
}

// use near_sdk::AccountId;
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
     //owner of the token
    pub owner_id: near_sdk::AccountId,
}

//The Json token is what will be returned from view calls.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    //token ID
    pub token_id: TokenId,
    //owner of the token
    pub owner_id: AccountId,
    //token metadata
    pub metadata: TokenMetadata,
}

pub trait NonFungibleTokenMetadata {
    //view call for returning the contract metadata
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

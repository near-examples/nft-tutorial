use near_sdk::json_types::U64;

use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_series(
        &mut self,
        id: u64,
        metadata: TokenMetadata,
        royalty: Option<HashMap<AccountId, u32>>,
    ) {
        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let caller = env::predecessor_account_id();
        require!(
            self.approved_creators.contains(&caller) == true,
            "only approved creators can add a type"
        );
        require!(
            self.series_by_id
                .insert(
                    &id,
                    &Series {
                        metadata,
                        //we add an optional parameter for perpetual royalties
                        royalty,
                        tokens: UnorderedSet::new(StorageKey::SeriesByIdInner {
                            // We get a new unique prefix for the collection
                            account_id_hash: hash_account_id(&format!(
                                "{}{}",
                                id, caller
                            )),
                        }),
                        owner_id: caller
                    }
                )
                .is_none(),
            "collection ID already exists"
        );

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);
    }

    #[payable]
    pub fn nft_mint(&mut self, id: U64, receiver_id: AccountId, injected_fields: U64) {

        assert!(injected_fields.0 == 3, "malicious injected fields detected");

        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let predecessor = env::predecessor_account_id();
        assert!(
            self.approved_minters.contains(&predecessor),
            "Not approved minter"
        );

        let mut series = self.series_by_id.get(&id.0).expect("Not a series");
        let cur_len = series.tokens.len();
        // Ensure we haven't overflowed on the number of copies minted
        if let Some(copies) = series.metadata.copies {
            require!(
                cur_len < copies,
                "cannot mint anymore NFTs for the given series. Limit reached"
            );
        }

        let token_id = format!("{}:{}", id.0, cur_len + 1);
        series.tokens.insert(&token_id);
        self.series_by_id.insert(&id.0, &series);

        //specify the token struct that contains the owner ID
        let token = Token {
            // Series ID that the token belongs to
            series_id: id.0,
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        require!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);
    }

    #[payable]
    /// Update the series ID for a given series. Caller must be series owner.
    pub fn update_series_id(
        &mut self,
        current_id: u64,
        new_id: u64,
    ) {
        let caller = env::predecessor_account_id();
        // Ensure the caller is the owner of the current series
        let series = self.series_by_id.remove(&current_id).expect("Not a series");
        require!(
            series.owner_id == caller,
            "Only the owner can update the series ID"
        );
        // Add the series to the new ID and make sure the new ID doesn't exist yet
        require!(
            self.series_by_id
                .insert(&new_id, &series)
                .is_none(),
            "Series ID already exists"
        );
    }
}

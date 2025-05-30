use crate::*;
use near_sdk::CryptoHash;
use std::mem::size_of;

//Basic NEAR amounts as constants
const ZERO_NEAR: NearToken = NearToken::from_yoctonear(0);
const ONE_YOCTONEAR: NearToken = NearToken::from_yoctonear(1);

//convert the royalty percentage and amount to pay into a payout
pub(crate) fn royalty_to_payout(royalty_percentage: u128, amount_to_pay: NearToken) -> NearToken {
    amount_to_pay.saturating_mul(royalty_percentage)
}

//calculate how many bytes the account ID is taking up
pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u128 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.as_str().len() as u128 + 4 + size_of::<u128>() as u128
}

//refund the storage taken up by passed in approved account IDs and send the funds to the passed in account ID.
pub(crate) fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I, //the approved account IDs must be passed in as an iterator
) -> Promise
where
    I: Iterator<Item = &'a AccountId>,
{
    //get the storage total by going through and summing all the bytes for each approved account IDs
    let storage_released = approved_account_ids
        .map(bytes_for_approved_account_id)
        .sum();
    //transfer the account the storage that is released
    Promise::new(account_id).transfer(env::storage_byte_cost().saturating_mul(storage_released))
}

//refund a map of approved account IDs and send the funds to the passed in account ID
pub(crate) fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    //call the refund_approved_account_ids_iter with the approved account IDs as keys
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

//used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &String) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

//used to make sure the user attached exactly 1 yoctoNEAR
pub(crate) fn assert_one_yocto() {
    assert_eq!(
        env::attached_deposit(),
        ONE_YOCTONEAR,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
}

//Assert that the user has attached at least 1 yoctoNEAR (for security reasons and to pay for storage)
pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        env::attached_deposit() >= ONE_YOCTONEAR,
        "Requires attached deposit of at least 1 yoctoNEAR",
    )
}

// Send all the non storage funds to the series owner
pub(crate) fn payout_series_owner(
    storage_used: u128,
    price_per_token: NearToken,
    owner_id: AccountId,
) {
    //get how much it would cost to store the information
    let required_cost = env::storage_byte_cost().saturating_mul(storage_used);
    //get the attached deposit
    let attached_deposit = env::attached_deposit();

    //make sure that the attached deposit is greater than or equal to the required cost
    assert!(
        attached_deposit.ge(&required_cost.saturating_add(price_per_token)),
        "Must attach {} yoctoNEAR to cover storage and price per token {}",
        required_cost,
        price_per_token
    );

    // If there's a price for the token, transfer everything but the storage to the series owner
    if price_per_token.gt(&ZERO_NEAR) {
        Promise::new(owner_id).transfer(attached_deposit.saturating_sub(required_cost));
    }
}

//refund the initial deposit based on the amount of storage that was used up
pub(crate) fn refund_deposit(storage_used: u128) {
    //get how much it would cost to store the information
    let required_cost = env::storage_byte_cost().saturating_mul(storage_used);
    //get the attached deposit
    let attached_deposit = env::attached_deposit();

    //make sure that the attached deposit is greater than or equal to the required cost
    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    //get the refund amount from the attached deposit - required cost
    let refund = attached_deposit.saturating_sub(required_cost);

    //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
    if refund.gt(&ONE_YOCTONEAR) {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

impl Contract {
    /// Ensure that the caller is the owner of the contract
    pub(crate) fn assert_contract_owner(&mut self) {
        assert!(
            self.owner_id == env::predecessor_account_id(),
            "only contract owner"
        )
    }

    //add a token to the set of tokens an owner has
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        //get the set of tokens for the given account
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            //if the account doesn't have any tokens, we create a new unordered set
            UnorderedSet::new(StorageKey::TokenPerOwnerInner {
                //we get a new unique prefix for the collection
                account_id_hash: hash_account_id(&account_id.to_string()),
            })
        });

        //we insert the token ID into the set
        tokens_set.insert(token_id);

        //we insert that set for the given account ID.
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    //remove a token from an owner (internal method and can't be called directly via CLI).
    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        //we get the set of tokens that the owner has
        let mut tokens_set = self
            .tokens_per_owner
            .get(account_id)
            //if there is no set of tokens for the owner, we panic with the following message:
            .expect("Token should be owned by the sender");

        //we remove the the token_id from the set of tokens
        tokens_set.remove(token_id);

        //if the token set is now empty, we remove the owner from the tokens_per_owner collection
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            //if the token set is not empty, we simply insert it back for the account ID.
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    //transfers the NFT to the receiver_id (internal method and can't be called directly via CLI).
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        //get the token object by passing in the token_id
        let token = self.tokens_by_id.get(token_id).expect("No token");

        //if the sender doesn't equal the owner, we check if the sender is in the approval list
        if sender_id != &token.owner_id {
            //if the token's approved account IDs doesn't contain the sender, we panic
            if !token.approved_account_ids.contains_key(sender_id) {
                env::panic_str("Unauthorized");
            }

            // If they included an approval_id, check if the sender's actual approval_id is the same as the one included
            if let Some(enforced_approval_id) = approval_id {
                //get the actual approval ID
                let actual_approval_id = token
                    .approved_account_ids
                    .get(sender_id)
                    //if the sender isn't in the map, we panic
                    .expect("Sender is not approved account");

                //make sure that the actual approval ID is the same as the one provided
                assert_eq!(
                    actual_approval_id, &enforced_approval_id,
                    "The actual approval_id {} is different from the given approval_id {}",
                    actual_approval_id, enforced_approval_id,
                );
            }
        }

        //we make sure that the sender isn't sending the token to themselves
        assert_ne!(
            &token.owner_id, receiver_id,
            "The token owner and the receiver should be different"
        );

        //we remove the token from it's current owner's set
        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        //we then add the token to the receiver_id's set
        self.internal_add_token_to_owner(receiver_id, token_id);

        //we create a new token struct
        let new_token = Token {
            series_id: token.series_id,
            owner_id: receiver_id.clone(),
            //reset the approval account IDs
            approved_account_ids: Default::default(),
            next_approval_id: token.next_approval_id,
        };
        //insert that new token into the tokens_by_id, replacing the old entry
        self.tokens_by_id.insert(token_id, &new_token);

        //if there was some memo attached, we log it.
        if let Some(memo) = memo.as_ref() {
            env::log_str(&format!("Memo: {}", memo).to_string());
        }

        // Default the authorized ID to be None for the logs.
        let mut authorized_id = None;
        //if the approval ID was provided, set the authorized ID equal to the sender
        if approval_id.is_some() {
            authorized_id = Some(sender_id.to_string());
        }

        // Construct the transfer log as per the events standard.
        let nft_transfer_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                // The optional authorized account ID to transfer the token on behalf of the old owner.
                authorized_id,
                // The old owner's account ID.
                old_owner_id: token.owner_id.to_string(),
                // The account ID of the new owner of the token.
                new_owner_id: receiver_id.to_string(),
                // A vector containing the token IDs as strings.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_transfer_log.to_string());

        //return the previous token object that was transferred.
        token
    }
}

use crate::*;
use near_sdk::{ext_contract, Gas};

const GAS_FOR_NFT_APPROVE: Gas = Gas(10_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

pub trait NonFungibleTokenCore {
    //approve an account ID to transfer a token on your behalf
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

    //check if the passed in account has access to approve the token ID
	fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;

    //revoke a specific account from transferring the token on your behalf
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    //revoke all accounts from transferring the token on your behalf
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleTokenApprovalsReceiver {
    //cross contract call to an external contract that is initiated during nft_approve
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    //allow a specific account ID to approve a token on your behalf
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
    /*
        assert at least one yocto for security reasons - this will cause a redirect to the NEAR wallet.
        The user needs to attach enough to pay for storage on the contract
    */
    assert_at_least_one_yocto();

    //get the token object from the token ID
    let mut token = self.tokens_by_id.get(&token_id).expect("No token");

    //make sure that the person calling the function is the owner of the token
    assert_eq!(
        &env::predecessor_account_id(),
        &token.owner_id,
        "Predecessor must be the token owner."
    );

    //get the next approval ID if we need a new approval
    let approval_id: u64 = token.next_approval_id;

    //check if the account has been approved already for this token
    let is_new_approval = token
        .approved_account_ids
        //insert returns none if the key was not present.  
        .insert(account_id.clone(), approval_id)
        //if the key was not present, .is_none() will return true so it is a new approval.
        .is_none();

    //if it was a new approval, we need to calculate how much storage is being used to add the account.
    let storage_used = if is_new_approval {
        bytes_for_approved_account_id(&account_id)
    //if it was not a new approval, we used no storage.
    } else {
        0
    };

    //increment the token's next approval ID by 1
    token.next_approval_id += 1;
    //insert the token back into the tokens_by_id collection
    self.tokens_by_id.insert(&token_id, &token);

    //refund any excess storage attached by the user. If the user didn't attach enough, panic. 
    refund_deposit(storage_used);

    //if some message was passed into the function, we initiate a cross contract call on the
    //account we're giving access to. 
    if let Some(msg) = msg {
        ext_non_fungible_approval_receiver::nft_on_approve(
            token_id,
            token.owner_id,
            approval_id,
            msg,
            account_id, //contract account we're calling
            NO_DEPOSIT, //NEAR deposit we attach to the call
            env::prepaid_gas() - GAS_FOR_NFT_APPROVE, //GAS we're attaching
        )
        .as_return(); // Returning this promise
    }
}

    //check if the passed in account has access to approve the token ID
	fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }

    //revoke a specific account from transferring the token on your behalf 
    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        /*
            FILL THIS IN
        */
    }

    //revoke all accounts from transferring the token on your behalf
    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        /*
            FILL THIS IN
        */
    }
}
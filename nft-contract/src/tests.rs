/* unit tests */
#[cfg(test)]
use crate::Contract;
use crate::TokenMetadata;
use crate::approval::NonFungibleTokenCore;
use near_sdk::json_types::{U128, U64};
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::testing_env;
use near_sdk::{env, AccountId};

use std::collections::HashMap;

const MINT_STORAGE_COST: u128 = 100_000_000_000_000_000_000_000;
const MIN_REQUIRED_APPROVAL_YOCTO: u128 = 170000000000000000000;

fn get_context(predecessor: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder.predecessor_account_id(predecessor);
    builder
}

fn sample_token_metadata() -> TokenMetadata {
    TokenMetadata {
        title: Some("Olympus Mons".into()),
        description: Some("The tallest mountain in the charted solar system".into()),
        media: None,
        media_hash: None,
        copies: Some(1u64),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    }
}

#[test]
#[should_panic(expected = "The contract is not initialized")]
fn test_default() {
    let context = get_context(accounts(1));
    testing_env!(context.build());
    let _contract = Contract::default();
}

#[test]
fn test_new_account_contract() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let contract = Contract::new_default_meta(accounts(1).into());
    testing_env!(context.is_view(true).build());
    let contract_nft_tokens = contract.nft_tokens(Some(U128(0)), None);
    assert_eq!(contract_nft_tokens.len(), 0);
}

#[test]
fn test_mint_nft() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_metadata: TokenMetadata = sample_token_metadata();
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), token_metadata, accounts(0), None);
    let contract_nft_tokens = contract.nft_tokens(Some(U128(0)), None);
    assert_eq!(contract_nft_tokens.len(), 1);

    assert_eq!(contract_nft_tokens[0].token_id, token_id);
    assert_eq!(contract_nft_tokens[0].owner_id, accounts(0));
    assert_eq!(
        contract_nft_tokens[0].metadata.title,
        sample_token_metadata().title
    );
    assert_eq!(
        contract_nft_tokens[0].metadata.description,
        sample_token_metadata().description
    );
    assert_eq!(
        contract_nft_tokens[0].metadata.media,
        sample_token_metadata().media
    );
    assert_eq!(contract_nft_tokens[0].approved_account_ids, HashMap::new());
}

#[test]
fn test_internal_transfer() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(1)
        .predecessor_account_id(accounts(0))
        .build());
    contract.internal_transfer(
        &accounts(0),
        &accounts(1),
        &token_id.clone(),
        Some(U64(1).0),
        None,
    );

    testing_env!(context
        .storage_usage(env::storage_usage())
        .account_balance(env::account_balance())
        .is_view(true)
        .attached_deposit(0)
        .build());

    let tokens = contract.nft_tokens_for_owner(accounts(1), Some(U128(0)), None);
    assert_ne!(
        tokens.len(),
        0,
        "Token not correctly created and/or sent to second account"
    );
    let token = &tokens[0];
    assert_eq!(token.token_id, token_id);
    assert_eq!(token.owner_id, accounts(1));
    assert_eq!(token.metadata.title, sample_token_metadata().title);
    assert_eq!(
        token.metadata.description,
        sample_token_metadata().description
    );
    assert_eq!(token.metadata.media, sample_token_metadata().media);
    assert_eq!(token.approved_account_ids, HashMap::new());
}

#[test]
fn test_nft_approve() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_APPROVAL_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_approve(token_id.clone(), accounts(1), None);

    testing_env!(context
        .storage_usage(env::storage_usage())
        .account_balance(env::account_balance())
        .is_view(true)
        .attached_deposit(0)
        .build());
    assert!(contract.nft_is_approved(token_id.clone(), accounts(1), None));
}

#[test]
fn test_nft_revoke() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    // alice approves bob
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_APPROVAL_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_approve(token_id.clone(), accounts(1), None);

    // alice revokes bob
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(1)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_revoke(token_id.clone(), accounts(1));
    testing_env!(context
        .storage_usage(env::storage_usage())
        .account_balance(env::account_balance())
        .is_view(true)
        .attached_deposit(0)
        .build());
    assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
}

#[test]
fn test_revoke_all() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    // alice approves bob
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_APPROVAL_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_approve(token_id.clone(), accounts(1), None);

    // alice revokes bob
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(1)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_revoke_all(token_id.clone());
    testing_env!(context
        .storage_usage(env::storage_usage())
        .account_balance(env::account_balance())
        .is_view(true)
        .attached_deposit(0)
        .build());
    assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
}

#[test]
fn test_internal_remove_token_from_owner() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    let contract_nft_tokens_before = contract.nft_tokens_for_owner(accounts(0), None, None);
    assert_eq!(contract_nft_tokens_before.len(), 1);

    contract.internal_remove_token_from_owner(&accounts(0), &token_id);
    let contract_nft_tokens_after = contract.nft_tokens_for_owner(accounts(0), None, None);
    assert_eq!(contract_nft_tokens_after.len(), 0);
}

#[test]
fn test_nft_payout() {
    use crate::royalty::NonFungibleTokenCore;
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    // alice approves bob
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_APPROVAL_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.nft_approve(token_id.clone(), accounts(1), None);

    let payout = contract.nft_payout(token_id.clone(), U128(10), 1);
    let expected = HashMap::from([(accounts(0), U128(10))]);
    assert_eq!(payout.payout, expected);
}

#[test]
fn test_nft_total_supply() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new_default_meta(accounts(0).into());

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());
    let token_id = "0".to_string();
    contract.nft_mint(token_id.clone(), sample_token_metadata(), accounts(0), None);

    let total_supply = contract.nft_total_supply();
    assert_eq!(total_supply, U128(1));
}
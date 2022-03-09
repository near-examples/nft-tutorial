/*
 * this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-counter-tutorial -- --nocapture
 * Note: 'rust-counter-tutorial' comes from cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
use crate::Contract;
use crate::TokenMetadata;
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId};
use near_sdk::serde::export::TryFrom;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;
// use near_sdk::MockedBlockchain;

// simple helper function to take a string literal and return a ValidAccountId
fn to_valid_account(account: &str) -> AccountId {
    AccountId::try_from(account.to_string()).expect("Invalid account")
}

// part of writing unit tests is setting up a mock context
// provide a `predecessor` here, it'll modify the default context
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
fn test_new() {
    let mut context = get_context(env::predecessor_account_id());
    testing_env!(context.build());
    let contract = Contract::new_default_meta(env::predecessor_account_id().into());
    testing_env!(context.is_view(true).build());
    let contract_nft_tokens = contract.nft_tokens(Some(U128(0)), None);
    assert_eq!(contract_nft_tokens.len(), 0);
}

// mark individual unit tests with #[test] for them to be registered and fired
#[test]
fn increment() {
    // set up the mock context into the testing environment
    let account = to_valid_account("foo.near");
    let context = get_context(account);
    testing_env!(context.build());
    // instantiate a contract variable with the counter at zero
    let mut contract = Contract::new_default_meta(
        env::predecessor_account_id()
    );
    let token_metadata: TokenMetadata = sample_token_metadata();
    // contract.nft_mint(
    //     "my_token_id_11", 
    //     token_metadata, 
    //     receiver_id, 
    //     perpetual_royalties
    // );
    // let mut contract = Contract {
    //      owner_id: 000,
    //      tokens_per_owner: 000,
    //      tokens_by_id: 000, 
    //      token_metadata_by_id: 000,
    //      metadata: 000,
    // }; 
    // contract.increment();
    // println!("Value after increment: {}", contract.get_num());
    // confirm that we received 1 when calling get_num
    // assert_eq!(1, contract.get_num());
}

#[test]
fn decrement() {
    // let context = VMContextBuilder::new();
    // testing_env!(context.build());
    // let mut contract = Counter { val: 0 };
    // contract.decrement();
    // println!("Value after decrement: {}", contract.get_num());
    // // confirm that we received -1 when calling get_num
    // assert_eq!(-1, contract.get_num());
}

#[test]
fn increment_and_reset() {
    // let context = VMContextBuilder::new();
    // testing_env!(context.build());
    // let mut contract = Counter { val: 0 };
    // contract.increment();
    // contract.reset();
    // println!("Value after reset: {}", contract.get_num());
    // // confirm that we received -1 when calling get_num
    // assert_eq!(0, contract.get_num());
}

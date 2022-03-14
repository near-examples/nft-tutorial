/* this file sets up unit tests */
#[cfg(test)]
use crate::Contract;
use near_sdk::{
    env,
    test_utils::{accounts, VMContextBuilder},
    testing_env, AccountId, json_types::U128,
};

const MIN_REQUIRED_APPROVAL_YOCTO: u128 = 170000000000000000000;
const MIN_REQUIRED_STORAGE_YOCTO: u128 = 10000000000000000000000;

fn get_context(predecessor: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder.predecessor_account_id(predecessor);
    builder
}

#[test]
#[should_panic(expected = "The contract is not initialized")]
fn test_default() {
    let context = get_context(accounts(0));
    testing_env!(context.build());
    let _contract = Contract::default();
}

#[test]
#[should_panic(expected = "Requires minimum deposit of 10000000000000000000000")]
fn test_storage_deposit_insufficient_deposit() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0));
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_APPROVAL_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.storage_deposit(Some(accounts(0)));
}

#[test]
fn test_storage_deposit() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0));
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_STORAGE_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.storage_deposit(Some(accounts(0)));
    let outcome = contract.storage_deposits.get(&accounts(0));
    let expected = MIN_REQUIRED_STORAGE_YOCTO;
    assert_eq!(outcome, Some(expected));
}

#[test]
fn test_storage_balance_of() {
    let mut context = get_context(accounts(0));
    testing_env!(context.build());
    let mut contract = Contract::new(accounts(0));
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MIN_REQUIRED_STORAGE_YOCTO)
        .predecessor_account_id(accounts(0))
        .build());
    contract.storage_deposit(Some(accounts(0)));
    let balance = contract.storage_balance_of(accounts(0));
    assert_eq!(balance, U128(MIN_REQUIRED_STORAGE_YOCTO));
}

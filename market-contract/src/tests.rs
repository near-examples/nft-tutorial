/* this file sets up unit tests */
#[cfg(test)]
use crate::Contract;
use near_sdk::{test_utils::{VMContextBuilder, accounts}, AccountId, testing_env};

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

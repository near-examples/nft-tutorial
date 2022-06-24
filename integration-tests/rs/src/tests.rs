use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker};

mod helpers;

const NFT_WASM_FILEPATH: &str = "../../out/main.wasm";
const MARKET_WASM_FILEPATH: &str = "../../out/market.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let nft_contract = worker.dev_deploy(&nft_wasm).await?;
    let market_wasm = std::fs::read(MARKET_WASM_FILEPATH)?;
    let market_contract = worker.dev_deploy(&market_wasm).await?;

    // create accounts
    let owner = worker.root_account();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let charlie = owner
        .create_subaccount(&worker, "charlie")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let dave = owner
        .create_subaccount(&worker, "dave")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    nft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({"owner_id": owner.id()}))?
        .transact()
        .await?;
    market_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({"owner_id": owner.id()}))?
        .transact()
        .await?;

    // begin tests
    test_nft_metadata_view(&owner, &nft_contract, &worker).await?;
    // TODO: uncomment below tests
    test_nft_mint_call(&owner, &alice, &nft_contract, &worker).await?;
    test_nft_approve_call(&bob, &nft_contract, &market_contract, &worker).await?;
    // test_nft_approve_call_long_msg_string(&alice, &ft_contract, &worker).await?;
    // test_sell_nft_listed_on_marketplace(&alice, &ft_contract, &worker).await?;
    // test_transfer_nft_when_listed_on_marketplace(&owner, &charlie, &ft_contract, &defi_contract, &worker).await?;
    // test_approval_revoke(&owner, &ft_contract, &defi_contract, &worker).await?;
    // test_reselling_and_royalties(&owner, &dave, ft_contract, &worker).await?;
    Ok(())
}

async fn test_nft_metadata_view(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let expected = json!({
        "base_uri": serde_json::Value::Null,
        "icon": serde_json::Value::Null,
        "name": "NFT Tutorial Contract",
        "reference": serde_json::Value::Null,
        "reference_hash": serde_json::Value::Null,
        "spec": "nft-1.0.0",
        "symbol": "GOTEAM",
    });
    let res: serde_json::Value = owner
        .call(&worker, contract.id(), "nft_metadata")
        .args_json(json!({  "account_id": owner.id()  }))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, expected);
    println!("      Passed ✅ test_nft_metadata_view");
    Ok(())
}

async fn test_nft_mint_call(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let request_payload = json!({
        "token_id": "1",
        "receiver_id": user.id(),
        "metadata": {
            "title": "LEEROYYYMMMJENKINSSS",
            "description": "Alright time's up, let's do this.",
            "media": "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1"
        },
    });

    user.call(&worker, contract.id(), "nft_mint")
        .args_json(request_payload)?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    let tokens: serde_json::Value = owner
        .call(&worker, contract.id(), "nft_tokens")
        .args_json(serde_json::json!({}))?
        .transact()
        .await?
        .json()?;

    let expected = json!([
        {   
            "approved_account_ids": {},
            "royalty": {},
            "token_id": "1",
            "owner_id": user.id(),
            "metadata": {
                "expires_at": serde_json::Value::Null, 
                "extra": serde_json::Value::Null, 
                "issued_at": serde_json::Value::Null, 
                "copies": serde_json::Value::Null,
                "media_hash": serde_json::Value::Null,
                "reference": serde_json::Value::Null,
                "reference_hash": serde_json::Value::Null,
                "starts_at": serde_json::Value::Null,
                "updated_at": serde_json::Value::Null,
                "title": "LEEROYYYMMMJENKINSSS",
                "description": "Alright time's up, let's do this.",
                "media": "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1"
            }
        }
    ]);

    assert_eq!(tokens, expected);
    println!("      Passed ✅ test_nft_mint_call");
    Ok(())
}

async fn test_nft_approve_call(
    user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "2";
    helpers::mint_nft(user, nft_contract, worker, token_id).await?;
    helpers::approve_nft(market_contract, user, nft_contract, worker, token_id).await?;

    let view_payload = json!({
        "token_id": token_id,
        "approved_account_id": market_contract.id(),
    });
    let result: bool = user
        .call(&worker, nft_contract.id(), "nft_is_approved")
        .args_json(view_payload)?
        .transact()
        .await?
        .json()?;
    
    assert_eq!(result, true);
    println!("      Passed ✅ test_nft_approve_call");
    Ok(())
}

async fn test_nft_approve_call_long_msg_string(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    match user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await
    {
        Ok(_result) => {
            panic!("storage_unregister worked despite account being funded")
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string
                .contains("Can't unregister the account with the positive balance without force")
            {
                panic!("storage_unregister with balance displays unexpected error message")
            }
            println!("      Passed ✅ test_close_account_non_empty_balance");
        }
    }
    Ok(())
}

async fn test_sell_nft_listed_on_marketplace(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let result: CallExecutionDetails = user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    assert_eq!(true, result.is_success());
    assert_eq!(
        result.logs()[0],
        format!(
            "Closed @{} with {}",
            user_with_funds.id(),
            parse_near!("1,000 N") // alice balance from above transfer_amount
        )
    );
    println!("      Passed ✅ test_close_account_force_non_empty_balance");
    Ok(())
}

async fn test_transfer_nft_when_listed_on_marketplace(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount_str = parse_near!("1,000,000 N").to_string();
    let ftc_amount_str = parse_near!("1,000 N").to_string();

    // register user
    owner
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // transfer ft
    owner
        .call(&worker, ft_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount_str
        }))?
        .deposit(1)
        .transact()
        .await?;

    user.call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": ftc_amount_str,
            "msg": "0",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let storage_result: CallExecutionDetails = user
        .call(&worker, ft_contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    // assert new state
    assert_eq!(
        storage_result.logs()[0],
        format!(
            "Closed @{} with {}",
            user.id(),
            parse_near!("999,000 N") // balance after defi ft transfer
        )
    );

    let total_supply: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(total_supply, U128::from(parse_near!("999,000,000 N")));

    let defi_balance: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(defi_balance, U128::from(parse_near!("999,000,000 N")));

    println!("      Passed ✅ test_transfer_call_with_burned_amount");
    Ok(())
}

async fn test_approval_revoke(
    owner: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount: u128 = parse_near!("100,000,000 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": amount_str,
            "msg": "take-my-money"
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let owner_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    assert_eq!(owner_before_balance.0 - amount, owner_after_balance.0);
    assert_eq!(defi_before_balance.0 + amount, defi_after_balance.0);
    println!("      Passed ✅ test_simulate_transfer_call_with_immediate_return_and_no_refund");
    Ok(())
}

async fn test_reselling_and_royalties(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount = parse_near!("10 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id":  owner.id()}))?
        .transact()
        .await?
        .json()?;
    let user_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;

    match owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": amount_str,
            "msg": "take-my-money",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
    {
        Ok(res) => {
            panic!("Was able to transfer FT to an unregistered account");
        }
        Err(err) => {
            let owner_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id":  owner.id()}))?
                .transact()
                .await?
                .json()?;
            let user_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id": user.id()}))?
                .transact()
                .await?
                .json()?;
            assert_eq!(user_before_balance, user_after_balance);
            assert_eq!(owner_before_balance, owner_after_balance);
            println!(
                "      Passed ✅ test_transfer_call_when_called_contract_not_registered_with_ft"
            );
        }
    }
    Ok(())
}

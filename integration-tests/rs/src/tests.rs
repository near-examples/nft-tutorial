use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
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
    test_nft_mint_call(&owner, &alice, &nft_contract, &worker).await?;
    test_nft_approve_call(&bob, &nft_contract, &market_contract, &worker).await?;
    test_nft_approve_call_long_msg_string(&alice, &nft_contract, &market_contract, &worker).await?;
    test_sell_nft_listed_on_marketplace(&alice, &nft_contract, &market_contract, &bob, &worker).await?;
    test_transfer_nft_when_listed_on_marketplace(&alice, &bob, &charlie, &nft_contract, &market_contract, &worker).await?;
    test_approval_revoke(&alice, &bob, &nft_contract, &market_contract, &worker).await?;
    test_reselling_and_royalties(&alice, &bob, &charlie, &nft_contract, &market_contract, &worker).await?;

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
    user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "3";
    helpers::mint_nft(user, nft_contract, worker, token_id).await?;
    helpers::pay_for_storage(user, market_contract, worker, 10000000000000000000000).await?;

    let approve_payload  = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": "sample message".repeat(10240),
    });

    match user.call(&worker, nft_contract.id(), "nft_approve")
        .args_json(approve_payload)?
        .deposit(helpers::DEFAULT_DEPOSIT)
        .gas(helpers::DEFAULT_GAS as u64)
        .transact()
        .await
    {
        Ok(_result) => {
            panic!("test_nft_approve_call_long_msg_string worked despite insufficient gas")
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string
                .contains("Not valid SaleArgs")
            {
                panic!("test_nft_approve_call_long_msg_string displays unexpected error message: {:?}", e_string);
            }

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
            println!("      Passed ✅ test_nft_approve_call_long_msg_string");
        }
    }
    Ok(())
}

async fn test_sell_nft_listed_on_marketplace(
    seller: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    buyer: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "4";
    let sale_price = 300000000000000000000000 as u128;  // 0.3 NEAR in yoctoNEAR
    helpers::mint_nft(seller, nft_contract, worker, token_id).await?;
    helpers::pay_for_storage(seller, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, worker, token_id, sale_price).await?;

    let before_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(buyer, worker).await?;
    helpers::purchase_listed_nft(buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(buyer, worker).await?;

    let dp = 1;  // being exact requires keeping track of gas usage 
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, dp), helpers::round_to_near_dp(before_seller_balance + sale_price, dp), "seller did not receive the sale price");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, dp), helpers::round_to_near_dp(before_buyer_balance - sale_price, dp), "buyer did not receive the sale price");

    println!("      Passed ✅ test_sell_nft_listed_on_marketplace");
    Ok(())
}

async fn test_transfer_nft_when_listed_on_marketplace(
    seller: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "5";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR
    helpers::mint_nft(seller, nft_contract, worker, token_id).await?;
    helpers::pay_for_storage(seller, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, worker, token_id, sale_price).await?;

    helpers::transfer_nft(seller, first_buyer, nft_contract, worker, token_id).await?;

    // attempt purchase NFT
    let before_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(second_buyer, worker).await?;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(seller, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(second_buyer, worker).await?;

    // assert owner remains first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances remain equal
    let dp = 1;     
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, dp), helpers::round_to_near_dp(before_seller_balance, dp), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, dp), helpers::round_to_near_dp(before_buyer_balance, dp), "buyer balance changed");

    println!("      Passed ✅ test_transfer_nft_when_listed_on_marketplace");

    Ok(())
}

async fn test_approval_revoke(
    first_user: &Account,
    second_user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "6";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR
    helpers::mint_nft(first_user, nft_contract, worker, token_id).await?;
    helpers::pay_for_storage(first_user, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(first_user, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // nft_revoke market_contract call
    let revoke_payload = json!({
        "account_id": market_contract.id(),
        "token_id": token_id,
    });
    first_user.call(&worker, nft_contract.id(), "nft_revoke")
        .args_json(revoke_payload)?
        .deposit(1)
        .transact()
        .await?;

    // market_contract attempts to nft_transfer, when second_user tries to purchase NFT on market
    let before_seller_balance: u128 = helpers::get_user_balance(first_user, worker).await?;
    let before_buyer_balance: u128 = helpers::get_user_balance(second_user, worker).await?;
    helpers::purchase_listed_nft(
        second_user, market_contract, nft_contract, worker, token_id, sale_price
    ).await?;
    let after_seller_balance: u128 = helpers::get_user_balance(first_user, worker).await?;
    let after_buyer_balance: u128 = helpers::get_user_balance(second_user, worker).await?;

    // assert owner remains first_user
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_user.id().to_string(), "token owner is not first_user");

    // assert balances unchanged
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance, 0), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance, 0), "buyer balance changed");    

    println!("      Passed ✅ test_approval_revoke");
    Ok(())
}

async fn test_reselling_and_royalties(
    user: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let token_id = "7";
    let sale_price = 3000000000000000000000000 as u128;  // 3 NEAR in yoctoNEAR

    // mint with royalties
    let request_payload = json!({
        "token_id": token_id,
        "receiver_id": user.id(),
        "metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
        "perpetual_royalties": {
            user.id().to_string(): 2000 as u128
        }
    });
    user.call(&worker, nft_contract.id(), "nft_mint")
        .args_json(request_payload)?
        .deposit(helpers::DEFAULT_DEPOSIT)
        .transact()
        .await?;

    helpers::pay_for_storage(user, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(user, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // first_buyer purchases NFT
    let mut before_seller_balance: u128 = helpers::get_user_balance(user, worker).await?;
    let mut before_buyer_balance: u128 = helpers::get_user_balance(first_buyer, worker).await?;
    helpers::purchase_listed_nft(first_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;
    let mut after_seller_balance: u128 = helpers::get_user_balance(user, worker).await?;
    let mut after_buyer_balance: u128 = helpers::get_user_balance(first_buyer, worker).await?;

    // assert owner becomes first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances changed
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance + sale_price, 0), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance - sale_price, 0), "buyer balance unchanged");

    // first buyer lists nft for sale
    helpers::pay_for_storage(first_buyer, market_contract, worker, 10000000000000000000000 as u128).await?;
    helpers::place_nft_for_sale(first_buyer, market_contract, nft_contract, worker, token_id, sale_price).await?;

    // second_buyer purchases NFT
    let resale_price = sale_price * 5;  // 15 NEAR
    before_seller_balance = helpers::get_user_balance(first_buyer, worker).await?;
    before_buyer_balance = helpers::get_user_balance(second_buyer, worker).await?;
    let before_user_balance: u128 = helpers::get_user_balance(user, worker).await?;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, worker, token_id, resale_price).await?;
    let after_user_balance: u128 = helpers::get_user_balance(user, worker).await?;
    after_seller_balance = helpers::get_user_balance(first_buyer, worker).await?;
    after_buyer_balance = helpers::get_user_balance(second_buyer, worker).await?;

    // assert owner changes to second_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, worker, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, second_buyer.id().to_string(), "token owner is not second_buyer");

    // assert balances changed
    let royalty_fee = resale_price / 5;
    assert_eq!(helpers::round_to_near_dp(after_seller_balance, 0), helpers::round_to_near_dp(before_seller_balance + resale_price - royalty_fee, 0), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance, 0), helpers::round_to_near_dp(before_buyer_balance - resale_price, 0), "buyer balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_user_balance, 0), helpers::round_to_near_dp(before_user_balance + royalty_fee, 0), "user balance unchanged");

    println!("      Passed ✅ test_reselling_and_royalties");
    Ok(())
}

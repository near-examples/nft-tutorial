use near_workspaces::{types::NearToken, Account, Contract};
use serde_json::json;

mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate environemnt
    let worker = near_workspaces::sandbox().await?;

    // deploy contracts
    let nft_wasm = near_workspaces::compile_project("../nft-contract-royalty/.").await?;
    let nft_contract = worker.dev_deploy(&nft_wasm).await?;

    let market_wasm = near_workspaces::compile_project("../market-contract/.").await?;
    let market_contract = worker.dev_deploy(&market_wasm).await?;

    // create accounts
    let owner = worker.root_account().unwrap();
    let alice = owner
        .create_subaccount("alice")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount("bob")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;
    let charlie = owner
        .create_subaccount("charlie")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    let _ = nft_contract
        .call("new_default_meta")
        .args_json(serde_json::json!({"owner_id": owner.id()}))
        .transact()
        .await?;
    let _ = market_contract
        .call("new")
        .args_json(serde_json::json!({"owner_id": owner.id()}))
        .transact()
        .await?;

    // begin tests
    test_nft_metadata_view(&owner, &nft_contract).await?;
    test_nft_mint_call(&owner, &alice, &nft_contract).await?;
    test_nft_approve_call(&bob, &nft_contract, &market_contract).await?;
    test_sell_nft_listed_on_marketplace(&alice, &nft_contract, &market_contract, &bob).await?;
    test_transfer_nft_when_listed_on_marketplace(&alice, &bob, &charlie, &nft_contract, &market_contract).await?;
    test_approval_revoke(&alice, &bob, &nft_contract, &market_contract).await?;
    test_reselling_and_royalties(&alice, &bob, &charlie, &nft_contract, &market_contract).await?;

    Ok(())
}

async fn test_nft_metadata_view(
    owner: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
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
        .call(contract.id(), "nft_metadata")
        .args_json(json!({ "account_id": owner.id() }))
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
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": "1",
        "receiver_id": user.id(),
        "metadata": {
            "title": "LEEROYYYMMMJENKINSSS",
            "description": "Alright time's up, let's do this.",
            "media": "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1"
        },
    });

    let _ = user.call(contract.id(), "nft_mint")
        .args_json(request_payload)
        .deposit(NearToken::from_millinear(80))
        .transact()
        .await;

    let tokens: serde_json::Value = owner
        .call(contract.id(), "nft_tokens")
        .args_json(serde_json::json!({}))
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
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "2";
    helpers::mint_nft(user, nft_contract, token_id).await?;
    helpers::approve_nft(market_contract, user, nft_contract, token_id).await?;

    let view_payload = json!({
        "token_id": token_id,
        "approved_account_id": market_contract.id(),
    });
    let result: bool = user
        .call(nft_contract.id(), "nft_is_approved")
        .args_json(view_payload)
        .transact()
        .await?
        .json()?;
    
    assert_eq!(result, true);
    println!("      Passed ✅ test_nft_approve_call");
    Ok(())
}

async fn test_sell_nft_listed_on_marketplace(
    seller: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    buyer: &Account,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "4";
    let approval_id = 0;
    let sale_price: NearToken = NearToken::from_yoctonear(10000000000000000000000000);

    helpers::mint_nft(seller, nft_contract, token_id).await?;
    helpers::pay_for_storage(seller, market_contract, NearToken::from_yoctonear(1000000000000000000000000)).await?;
    helpers::approve_nft(market_contract, seller, nft_contract, token_id).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, token_id, approval_id, &sale_price).await?;

    let before_seller_balance: NearToken = helpers::get_user_balance(seller).await;
    let before_buyer_balance: NearToken = helpers::get_user_balance(buyer).await;

    helpers::purchase_listed_nft(buyer, market_contract, nft_contract, token_id, sale_price).await?;

    let after_seller_balance: NearToken = helpers::get_user_balance(seller).await;
    let after_buyer_balance: NearToken = helpers::get_user_balance(buyer).await;

    let dp = 1;  // being exact requires keeping track of gas usage
    assert_eq!(helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_seller_balance.saturating_add(sale_price).as_yoctonear(), dp), "seller did not receive the sale price");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_buyer_balance.saturating_sub(sale_price).as_yoctonear(), dp), "buyer did not send the sale price");

    println!("      Passed ✅ test_sell_nft_listed_on_marketplace");
    Ok(())
}

async fn test_transfer_nft_when_listed_on_marketplace(
    seller: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "5";
    let approval_id = 0;
    let sale_price = NearToken::from_near(3);

    helpers::mint_nft(seller, nft_contract, token_id).await?;
    helpers::pay_for_storage(seller, market_contract, NearToken::from_millinear(10)).await?;
    helpers::approve_nft(market_contract, seller, nft_contract, token_id).await?;
    helpers::place_nft_for_sale(seller, market_contract, nft_contract, token_id, approval_id, &sale_price).await?;

    helpers::transfer_nft(seller, first_buyer, nft_contract, token_id).await?;

    // attempt purchase NFT
    let before_seller_balance: NearToken = helpers::get_user_balance(seller).await;
    let before_buyer_balance: NearToken = helpers::get_user_balance(second_buyer).await;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, token_id, sale_price).await?;
    let after_seller_balance: NearToken = helpers::get_user_balance(seller).await;
    let after_buyer_balance: NearToken = helpers::get_user_balance(second_buyer).await;

    // assert owner remains first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances remain equal
    let dp = 1;  // being exact requires keeping track of gas usage
    assert_eq!(helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_seller_balance.as_yoctonear(), dp), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_buyer_balance.as_yoctonear(), dp), "buyer balance changed");

    println!("      Passed ✅ test_transfer_nft_when_listed_on_marketplace");

    Ok(())
}

async fn test_approval_revoke(
    first_user: &Account,
    second_user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "6";
    let approval_id = 0;
    let sale_price = NearToken::from_near(3);

    helpers::mint_nft(first_user, nft_contract, token_id).await?;
    helpers::pay_for_storage(first_user, market_contract, NearToken::from_millinear(10)).await?;
    helpers::place_nft_for_sale(first_user, market_contract, nft_contract, token_id, approval_id, &sale_price).await?;

    // nft_revoke market_contract call
    let revoke_payload = json!({
        "account_id": market_contract.id(),
        "token_id": token_id,
    });
    let _ = first_user.call(nft_contract.id(), "nft_revoke")
        .args_json(revoke_payload)
        .deposit(helpers::ONE_YOCTO_NEAR)
        .transact()
        .await?;

    // market_contract attempts to nft_transfer, when second_user tries to purchase NFT on market
    let before_seller_balance: NearToken = helpers::get_user_balance(first_user).await;
    let before_buyer_balance: NearToken = helpers::get_user_balance(second_user).await;
    helpers::purchase_listed_nft(
        second_user, market_contract, nft_contract, token_id, sale_price
    ).await?;
    let after_seller_balance: NearToken = helpers::get_user_balance(first_user).await;
    let after_buyer_balance: NearToken = helpers::get_user_balance(second_user).await;

    // assert owner remains first_user
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_user.id().to_string(), "token owner is not first_user");

    // assert balances unchanged
    let dp = 1;  // being exact requires keeping track of gas usage
    assert_eq!(helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_seller_balance.as_yoctonear(), dp), "seller balance changed");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_buyer_balance.as_yoctonear(), dp), "buyer balance changed");

    println!("      Passed ✅ test_approval_revoke");
    Ok(())
}

async fn test_reselling_and_royalties(
    user: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "7";
    let approval_id = 0;
    let sale_price: NearToken = NearToken::from_near(1);  // 1 NEAR in yoctoNEAR

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
    let _ = user.call(nft_contract.id(), "nft_mint")
        .args_json(request_payload)
        .deposit(NearToken::from_yoctonear(helpers::DEFAULT_DEPOSIT))
        .transact()
        .await;

    helpers::pay_for_storage(user, market_contract, NearToken::from_millinear(10)).await?;
    helpers::approve_nft(market_contract, user, nft_contract, token_id).await?;
    helpers::place_nft_for_sale(user, market_contract, nft_contract, token_id, approval_id, &sale_price).await?;

    // first_buyer purchases NFT
    let mut before_seller_balance: NearToken = helpers::get_user_balance(user).await;
    let mut before_buyer_balance: NearToken = helpers::get_user_balance(first_buyer).await;
    helpers::purchase_listed_nft(first_buyer, market_contract, nft_contract, token_id, sale_price).await?;
    let mut after_seller_balance: NearToken = helpers::get_user_balance(user).await;
    let mut after_buyer_balance: NearToken = helpers::get_user_balance(first_buyer).await;

    // assert owner becomes first_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, first_buyer.id().to_string(), "token owner is not first_buyer");

    // assert balances changed
    let dp = 1;  // being exact requires keeping track of gas usage
    assert_eq!(helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_seller_balance.saturating_add(sale_price).as_yoctonear(), dp), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_buyer_balance.saturating_sub(sale_price).as_yoctonear(), dp), "buyer balance unchanged");

    // first buyer lists nft for sale
    let approval_id = 1;
    helpers::pay_for_storage(first_buyer, market_contract, NearToken::from_millinear(10)).await?;
    helpers::approve_nft(market_contract, first_buyer, nft_contract, token_id).await?;
    helpers::place_nft_for_sale(first_buyer, market_contract, nft_contract, token_id, approval_id, &sale_price).await?;

    // second_buyer purchases NFT
    let resale_price = sale_price.saturating_mul(5);  // 15 NEAR
    before_seller_balance = helpers::get_user_balance(first_buyer).await;
    before_buyer_balance = helpers::get_user_balance(second_buyer).await;
    let before_user_balance: NearToken = helpers::get_user_balance(user).await;
    helpers::purchase_listed_nft(second_buyer, market_contract, nft_contract, token_id, resale_price).await?;
    let after_user_balance: NearToken = helpers::get_user_balance(user).await;
    after_seller_balance = helpers::get_user_balance(first_buyer).await;
    after_buyer_balance = helpers::get_user_balance(second_buyer).await;

    // assert owner changes to second_buyer
    let token_info: serde_json::Value = helpers::get_nft_token_info(nft_contract, token_id).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(owner_id, second_buyer.id().to_string(), "token owner is not second_buyer");

    // assert balances changed
    let royalty_fee = resale_price.saturating_div(5);
    assert_eq!(helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_seller_balance.saturating_add(resale_price).saturating_sub(royalty_fee).as_yoctonear(), dp), "seller balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_buyer_balance.saturating_sub(resale_price).as_yoctonear(), dp), "buyer balance unchanged");
    assert_eq!(helpers::round_to_near_dp(after_user_balance.as_yoctonear(), dp), helpers::round_to_near_dp(before_user_balance.saturating_add(royalty_fee).as_yoctonear(), dp), "user balance unchanged");

    println!("      Passed ✅ test_reselling_and_royalties");
    Ok(())
}

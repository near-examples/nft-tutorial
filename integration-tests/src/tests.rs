use std::sync::Arc;

use near_api::{types::NearToken, Account, Contract, NetworkConfig, Signer};
use serde_json::json;

mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = helpers::init_sandbox().await?;
    // Initialize the accounts
    let (alice, bob, charlie) = helpers::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (nft_contract, market_contract, signer) =
        helpers::init_contracts(&sandbox, &sandbox_network).await?;

    // begin tests
    test_nft_metadata_view(&nft_contract, &sandbox_network).await?;
    test_nft_mint_call(&alice, &nft_contract, &signer, &sandbox_network).await?;
    test_nft_approve_call(
        &bob,
        &nft_contract,
        &market_contract,
        &signer,
        &sandbox_network,
    )
    .await?;
    test_sell_nft_listed_on_marketplace(
        &alice,
        &nft_contract,
        &market_contract,
        &bob,
        &signer,
        &sandbox_network,
    )
    .await?;
    test_transfer_nft_when_listed_on_marketplace(
        &alice,
        &bob,
        &charlie,
        &nft_contract,
        &market_contract,
        &signer,
        &sandbox_network,
    )
    .await?;
    test_approval_revoke(
        &alice,
        &bob,
        &nft_contract,
        &market_contract,
        &signer,
        &sandbox_network,
    )
    .await?;
    test_reselling_and_royalties(
        &alice,
        &bob,
        &charlie,
        &nft_contract,
        &market_contract,
        &signer,
        &sandbox_network,
    )
    .await?;
    test_royalties_exceeding_100_percents(
        &alice,
        &nft_contract,
        &market_contract,
        &signer,
        &sandbox_network,
    )
    .await?;

    Ok(())
}

async fn test_nft_metadata_view(
    nft_contract: &Contract,
    sandbox_network: &NetworkConfig,
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
    let res: serde_json::Value = nft_contract
        .call_function(
            "nft_metadata",
            json!({ "account_id": nft_contract.account_id() }),
        )
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(res, expected);
    println!("      Passed ✅ test_nft_metadata_view");
    Ok(())
}

async fn test_nft_mint_call(
    user: &Account,
    nft_contract: &Contract,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": "1",
        "token_owner_id": user.account_id(),
        "token_metadata": {
            "title": "LEEROYYYMMMJENKINSSS",
            "description": "Alright time's up, let's do this.",
            "media": "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1"
        },
    });

    let _ = nft_contract
        .call_function("nft_mint", json!(request_payload))
        .transaction()
        .deposit(NearToken::from_millinear(80))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let tokens: serde_json::Value = nft_contract
        .call_function("nft_tokens", json!({}))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    let expected = json!([
        {
            "approved_account_ids": {},
            "royalty": {},
            "token_id": "1",
            "owner_id": user.account_id(),
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
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "2";
    helpers::mint_nft(user, nft_contract, token_id, signer, sandbox_network).await?;
    helpers::approve_nft(
        market_contract,
        user,
        nft_contract,
        token_id,
        signer,
        sandbox_network,
    )
    .await?;

    let view_payload = json!({
        "token_id": token_id,
        "approved_account_id": market_contract.account_id(),
    });
    let result: bool = nft_contract
        .call_function("nft_is_approved", json!(view_payload))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    assert_eq!(result, true);
    println!("      Passed ✅ test_nft_approve_call");
    Ok(())
}

async fn test_sell_nft_listed_on_marketplace(
    seller: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    buyer: &Account,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "4";
    let approval_id = 0;
    let sale_price: NearToken = NearToken::from_near(1);

    helpers::mint_nft(seller, nft_contract, token_id, &signer, &sandbox_network).await?;
    helpers::pay_for_storage(
        seller,
        market_contract,
        NearToken::from_millinear(10),
        &signer,
        &sandbox_network,
    )
    .await?;
    helpers::approve_nft(
        market_contract,
        seller,
        nft_contract,
        token_id,
        &signer,
        &sandbox_network,
    )
    .await?;
    helpers::place_nft_for_sale(
        seller,
        market_contract,
        nft_contract,
        token_id,
        approval_id,
        &sale_price,
        &signer,
        &sandbox_network,
    )
    .await?;

    let before_seller_balance: NearToken =
        helpers::get_user_balance(seller, &sandbox_network).await;
    let before_buyer_balance: NearToken = helpers::get_user_balance(buyer, &sandbox_network).await;

    helpers::purchase_listed_nft(
        buyer,
        market_contract,
        nft_contract,
        token_id,
        sale_price,
        &signer,
        &sandbox_network,
    )
    .await?
    .assert_success();

    let after_seller_balance: NearToken = helpers::get_user_balance(seller, &sandbox_network).await;
    let after_buyer_balance: NearToken = helpers::get_user_balance(buyer, &sandbox_network).await;

    let dp = 1; // being exact requires keeping track of gas usage
    assert_eq!(
        helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_seller_balance
                .saturating_add(sale_price)
                .as_yoctonear(),
            dp
        ),
        "seller did not receive the sale price"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_buyer_balance
                .saturating_sub(sale_price)
                .as_yoctonear(),
            dp
        ),
        "buyer did not send the sale price"
    );

    println!("      Passed ✅ test_sell_nft_listed_on_marketplace");
    Ok(())
}

async fn test_transfer_nft_when_listed_on_marketplace(
    seller: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "5";
    let approval_id = 0;
    let sale_price = NearToken::from_near(3);

    helpers::mint_nft(seller, nft_contract, token_id, signer, sandbox_network).await?;
    helpers::pay_for_storage(
        seller,
        market_contract,
        NearToken::from_millinear(10),
        signer,
        sandbox_network,
    )
    .await?;
    helpers::approve_nft(
        market_contract,
        seller,
        nft_contract,
        token_id,
        signer,
        sandbox_network,
    )
    .await?;
    helpers::place_nft_for_sale(
        seller,
        market_contract,
        nft_contract,
        token_id,
        approval_id,
        &sale_price,
        &signer,
        &sandbox_network,
    )
    .await?;

    helpers::transfer_nft(
        seller,
        first_buyer,
        nft_contract,
        token_id,
        signer,
        sandbox_network,
    )
    .await?;

    // attempt purchase NFT
    let before_seller_balance: NearToken =
        helpers::get_user_balance(seller, &sandbox_network).await;
    let before_buyer_balance: NearToken =
        helpers::get_user_balance(second_buyer, &sandbox_network).await;
    helpers::purchase_listed_nft(
        second_buyer,
        market_contract,
        nft_contract,
        token_id,
        sale_price,
        &signer,
        &sandbox_network,
    )
    .await?
    .assert_success();
    let after_seller_balance: NearToken = helpers::get_user_balance(seller, &sandbox_network).await;
    let after_buyer_balance: NearToken =
        helpers::get_user_balance(second_buyer, &sandbox_network).await;

    // assert owner remains first_buyer
    let token_info: serde_json::Value =
        helpers::get_nft_token_info(nft_contract, token_id, &sandbox_network).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(
        owner_id,
        first_buyer.account_id().to_string(),
        "token owner is not first_buyer"
    );

    // assert balances remain equal
    let dp = 1; // being exact requires keeping track of gas usage
    assert_eq!(
        helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(before_seller_balance.as_yoctonear(), dp),
        "seller balance changed"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(before_buyer_balance.as_yoctonear(), dp),
        "buyer balance changed"
    );

    println!("      Passed ✅ test_transfer_nft_when_listed_on_marketplace");

    Ok(())
}

async fn test_approval_revoke(
    first_user: &Account,
    second_user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "6";
    let approval_id = 0;
    let sale_price = NearToken::from_near(3);

    helpers::mint_nft(first_user, nft_contract, token_id, signer, sandbox_network).await?;
    helpers::pay_for_storage(
        first_user,
        market_contract,
        NearToken::from_millinear(10),
        signer,
        sandbox_network,
    )
    .await?;
    helpers::place_nft_for_sale(
        first_user,
        market_contract,
        nft_contract,
        token_id,
        approval_id,
        &sale_price,
        &signer,
        &sandbox_network,
    )
    .await?;

    // nft_revoke market_contract call
    let revoke_payload = json!({
        "account_id": market_contract.account_id(),
        "token_id": token_id,
    });
    let _ = nft_contract
        .call_function("nft_revoke", json!(revoke_payload))
        .transaction()
        .deposit(helpers::ONE_YOCTO_NEAR)
        .with_signer(first_user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // market_contract attempts to nft_transfer, when second_user tries to purchase NFT on market
    let before_seller_balance: NearToken =
        helpers::get_user_balance(first_user, &sandbox_network).await;
    let before_buyer_balance: NearToken =
        helpers::get_user_balance(second_user, &sandbox_network).await;
    helpers::purchase_listed_nft(
        second_user,
        market_contract,
        nft_contract,
        token_id,
        sale_price,
        &signer,
        &sandbox_network,
    )
    .await?
    .assert_failure();
    let after_seller_balance: NearToken =
        helpers::get_user_balance(first_user, &sandbox_network).await;
    let after_buyer_balance: NearToken =
        helpers::get_user_balance(second_user, &sandbox_network).await;

    // assert owner remains first_user
    let token_info: serde_json::Value =
        helpers::get_nft_token_info(nft_contract, token_id, &sandbox_network).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(
        owner_id,
        first_user.account_id().to_string(),
        "token owner is not first_user"
    );

    // assert balances unchanged
    let dp = 1; // being exact requires keeping track of gas usage
    assert_eq!(
        helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(before_seller_balance.as_yoctonear(), dp),
        "seller balance changed"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(before_buyer_balance.as_yoctonear(), dp),
        "buyer balance changed"
    );

    println!("      Passed ✅ test_approval_revoke");
    Ok(())
}

async fn test_reselling_and_royalties(
    user: &Account,
    first_buyer: &Account,
    second_buyer: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "7";
    let approval_id = 0;
    let sale_price: NearToken = NearToken::from_near(1); // 1 NEAR in yoctoNEAR

    // mint with royalties
    let request_payload = json!({
        "token_id": token_id,
        "token_owner_id": user.account_id(),
        "token_metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
        "perpetual_royalties": {
            user.account_id().to_string(): 2000 as u128
        }
    });
    let _ = nft_contract
        .call_function("nft_mint", json!(request_payload))
        .transaction()
        .deposit(NearToken::from_yoctonear(helpers::DEFAULT_DEPOSIT))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    helpers::pay_for_storage(
        user,
        market_contract,
        NearToken::from_millinear(10),
        signer,
        sandbox_network,
    )
    .await?;
    helpers::approve_nft(
        market_contract,
        user,
        nft_contract,
        token_id,
        signer,
        sandbox_network,
    )
    .await?;
    helpers::place_nft_for_sale(
        user,
        market_contract,
        nft_contract,
        token_id,
        approval_id,
        &sale_price,
        &signer,
        &sandbox_network,
    )
    .await?;

    // first_buyer purchases NFT
    let mut before_seller_balance: NearToken =
        helpers::get_user_balance(user, &sandbox_network).await;
    let mut before_buyer_balance: NearToken =
        helpers::get_user_balance(first_buyer, &sandbox_network).await;
    helpers::purchase_listed_nft(
        first_buyer,
        market_contract,
        nft_contract,
        token_id,
        sale_price,
        &signer,
        &sandbox_network,
    )
    .await?
    .assert_success();
    let mut after_seller_balance: NearToken =
        helpers::get_user_balance(user, &sandbox_network).await;
    let mut after_buyer_balance: NearToken =
        helpers::get_user_balance(first_buyer, &sandbox_network).await;

    // assert owner becomes first_buyer
    let token_info: serde_json::Value =
        helpers::get_nft_token_info(nft_contract, token_id, &sandbox_network).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(
        owner_id,
        first_buyer.account_id().to_string(),
        "token owner is not first_buyer"
    );

    // assert balances changed
    let dp = 1; // being exact requires keeping track of gas usage
    assert_eq!(
        helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_seller_balance
                .saturating_add(sale_price)
                .as_yoctonear(),
            dp
        ),
        "seller balance unchanged"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_buyer_balance
                .saturating_sub(sale_price)
                .as_yoctonear(),
            dp
        ),
        "buyer balance unchanged"
    );

    // first buyer lists nft for sale
    let approval_id = 1;
    helpers::pay_for_storage(
        first_buyer,
        market_contract,
        NearToken::from_millinear(10),
        signer,
        sandbox_network,
    )
    .await?;
    helpers::approve_nft(
        market_contract,
        first_buyer,
        nft_contract,
        token_id,
        signer,
        sandbox_network,
    )
    .await?;
    helpers::place_nft_for_sale(
        first_buyer,
        market_contract,
        nft_contract,
        token_id,
        approval_id,
        &sale_price,
        &signer,
        &sandbox_network,
    )
    .await?;

    // second_buyer purchases NFT
    let resale_price = sale_price.saturating_mul(5); // 15 NEAR
    before_seller_balance = helpers::get_user_balance(first_buyer, &sandbox_network).await;
    before_buyer_balance = helpers::get_user_balance(second_buyer, &sandbox_network).await;
    let before_user_balance: NearToken = helpers::get_user_balance(user, &sandbox_network).await;
    helpers::purchase_listed_nft(
        second_buyer,
        market_contract,
        nft_contract,
        token_id,
        resale_price,
        &signer,
        &sandbox_network,
    )
    .await?
    .assert_success();
    let after_user_balance: NearToken = helpers::get_user_balance(user, &sandbox_network).await;
    after_seller_balance = helpers::get_user_balance(first_buyer, &sandbox_network).await;
    after_buyer_balance = helpers::get_user_balance(second_buyer, &sandbox_network).await;

    // assert owner changes to second_buyer
    let token_info: serde_json::Value =
        helpers::get_nft_token_info(nft_contract, token_id, &sandbox_network).await?;
    let owner_id: String = token_info["owner_id"].as_str().unwrap().to_string();
    assert_eq!(
        owner_id,
        second_buyer.account_id().to_string(),
        "token owner is not second_buyer"
    );

    // assert balances changed
    let royalty_fee = resale_price.saturating_div(5);
    assert_eq!(
        helpers::round_to_near_dp(after_seller_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_seller_balance
                .saturating_add(resale_price)
                .saturating_sub(royalty_fee)
                .as_yoctonear(),
            dp
        ),
        "seller balance unchanged"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_buyer_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_buyer_balance
                .saturating_sub(resale_price)
                .as_yoctonear(),
            dp
        ),
        "buyer balance unchanged"
    );
    assert_eq!(
        helpers::round_to_near_dp(after_user_balance.as_yoctonear(), dp),
        helpers::round_to_near_dp(
            before_user_balance
                .saturating_add(royalty_fee)
                .as_yoctonear(),
            dp
        ),
        "user balance unchanged"
    );

    println!("      Passed ✅ test_reselling_and_royalties");
    Ok(())
}

async fn test_royalties_exceeding_100_percents(
    user: &Account,
    nft_contract: &Contract,
    market_contract: &Contract,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "7";

    // mint with royalties
    let request_payload = json!({
        "token_id": token_id,
        "token_owner_id": user.account_id(),
        "token_metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
        "perpetual_royalties": {
            user.account_id().to_string(): 5000 as u128,
            market_contract.account_id().to_string(): 6000 as u128
        }
    });
    let _ = nft_contract
        .call_function("nft_mint", json!(request_payload))
        .transaction()
        .deposit(NearToken::from_yoctonear(helpers::DEFAULT_DEPOSIT))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_failure();

    println!("      Passed ✅ test_royalties_exceeding_100_percents");
    Ok(())
}

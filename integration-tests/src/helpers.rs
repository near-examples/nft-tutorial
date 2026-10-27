use near_sdk::Gas;
use near_workspaces::{
    types::{AccountDetails, NearToken},
    Account, Contract,
};
use serde_json::json;

pub const DEFAULT_DEPOSIT: u128 = 10000000000000000000000;
pub const ONE_YOCTO_NEAR: NearToken = NearToken::from_yoctonear(1);

pub async fn mint_nft(
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "token_owner_id": user.id(),
        "token_metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
    });

    let _ = user
        .call(nft_contract.id(), "nft_mint")
        .args_json(request_payload)
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn approve_nft(
    market_contract: &Contract,
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": serde_json::Value::Null,
    });

    let _ = user
        .call(nft_contract.id(), "nft_approve")
        .args_json(request_payload)
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn pay_for_storage(
    user: &Account,
    market_contract: &Contract,
    amount: NearToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({});

    let _ = user
        .call(market_contract.id(), "storage_deposit")
        .args_json(request_payload)
        .deposit(amount)
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn place_nft_for_sale(
    user: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    token_id: &str,
    approval_id: u64,
    price: &NearToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "nft_contract_id": nft_contract.id(),
        "token_id": token_id,
        "approval_id": approval_id,
        "sale_conditions": NearToken::as_yoctonear(price).to_string(),
    });
    let _ = user
        .call(market_contract.id(), "list_nft_for_sale")
        .args_json(request_payload)
        .gas(Gas::from_tgas(100))
        .transact()
        .await?
        .into_result()?;

    let sale: serde_json::Value = market_contract
        .view("get_sale")
        .args_json(json!({ "nft_contract_token": format!("{}.{}", nft_contract.id(), token_id) }))
        .await?
        .json()?;
    assert_eq!(
        sale["owner_id"],
        user.id().as_str(),
        "Listing owner must match seller"
    );
    assert_eq!(
        sale["approval_id"], approval_id,
        "Listing approval must match token approval"
    );
    assert_eq!(
        sale["sale_conditions"],
        serde_json::to_value(price)?,
        "Listing price must match request"
    );
    Ok(())
}

pub async fn get_user_balance(
    user: &Account,
    worker: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
) -> NearToken {
    // Execution can complete before gas-refund receipts reach account state.
    // Wait for later blocks, then query a known post-transaction block rather
    // than the lagging `final` head or a transient reserved balance.
    let target_height = worker.view_block().await.expect("Latest block").height() + 3;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let latest = worker.view_block().await.expect("Latest block").height();
        if latest >= target_height {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "Refund blocks did not arrive"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let details: AccountDetails = user
        .view_account()
        .block_height(target_height)
        .await
        .expect("Account has to have some balance");
    details.balance
}

pub async fn purchase_listed_nft(
    bidder: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    token_id: &str,
    offer_price: NearToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "nft_contract_id": nft_contract.id(),
    });

    let _ = bidder
        .call(market_contract.id(), "offer")
        .args_json(request_payload)
        .max_gas()
        .deposit(offer_price)
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn transfer_nft(
    sender: &Account,
    receiver: &Account,
    nft_contract: &Contract,
    token_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "receiver_id": receiver.id(),
        "approval_id": 1 as u64,
    });

    let _ = sender
        .call(nft_contract.id(), "nft_transfer")
        .args_json(request_payload)
        .max_gas()
        .deposit(ONE_YOCTO_NEAR)
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn get_nft_token_info(
    nft_contract: &Contract,
    token_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(nft_contract
        .view("nft_token")
        .args_json(json!({ "token_id": token_id }))
        .await?
        .json()?)
}

pub fn round_to_near_dp(amount: u128, sf: u128) -> String {
    let near_amount = amount as f64 / 1_000_000_000_000_000_000_000_000.0; // yocto in 1 NEAR
    return format!("{:.1$}", near_amount, sf as usize);
}

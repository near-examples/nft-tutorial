use serde_json::json;
use workspaces::{network::Sandbox, Account, Contract, Worker, AccountDetails};

pub const DEFAULT_DEPOSIT: u128 = 6760000000000000000000 as u128;
const DEFAULT_GAS: u128 = 300000000000000 as u128;

pub async fn mint_nft(
    user: &Account,
    nft_contract: &Contract,
    worker: &Worker<Sandbox>,
    token_id: &str,
) -> anyhow::Result<()> { 
    let request_payload = json!({
        "token_id": token_id,
        "receiver_id": user.id(),
        "metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
    });

    user.call(&worker, nft_contract.id(), "nft_mint")
        .args_json(request_payload)?
        .deposit(DEFAULT_DEPOSIT)
        .transact()
        .await?;
    
    Ok(())
}

pub async fn approve_nft(
    market_contract: &Contract,
    user: &Account,
    nft_contract: &Contract,
    worker: &Worker<Sandbox>,
    token_id: &str,
) -> anyhow::Result<()> {
    let request_payload  = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": serde_json::Value::Null,
    });

    user.call(&worker, nft_contract.id(), "nft_approve")
        .args_json(request_payload)?
        .deposit(DEFAULT_DEPOSIT)
        .transact()
        .await?;

    Ok(())
}

pub async fn pay_for_storage(
    user: &Account,
    market_contract: &Contract,
    worker: &Worker<Sandbox>,
    amount: u128,
) -> anyhow::Result<()> {
    let request_payload = json!({});
    
    user.call(&worker, market_contract.id(), "storage_deposit")
        .args_json(request_payload)?
        .deposit(amount)
        .transact()
        .await?;

    Ok(())
}

pub async fn place_nft_for_sale(
    user: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    worker: &Worker<Sandbox>,
    token_id: &str,
    price: u128,
) -> anyhow::Result<()> {
    let request_payload  = json!({
        "token_id": token_id,
        "account_id": market_contract.id(),
        "msg": format!(r#"{{ "sale_conditions" : "{}" }}"#, price.to_string()),
    });

    user.call(&worker, nft_contract.id(), "nft_approve")
        .args_json(request_payload)?
        .deposit(DEFAULT_DEPOSIT)
        .transact()
        .await?;

    Ok(())
}

pub async fn get_user_balance(
    user: &Account,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<u128> {
    let details: AccountDetails = user.view_account(worker).await?;
    Ok(details.balance)
}

pub async fn purchase_listed_nft(
    bidder: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    worker: &Worker<Sandbox>,
    token_id: &str,
    offer_price: u128
) -> anyhow::Result<()> {
    let request_payload  = json!({
        "token_id": token_id,
        "nft_contract_id": nft_contract.id(),
    });

    bidder.call(&worker, market_contract.id(), "offer")
        .args_json(request_payload)?
        .gas(DEFAULT_GAS as u64)
        .deposit(offer_price)
        .transact()
        .await?;

    Ok(())
}

pub fn round_to_sf(
    amount: u128,
    sf: u128,
) -> u128 {
    let value_str: String = format!("{}", amount);
    let str_len = value_str.chars().count();
    let slice = &value_str[..sf as usize];
    let var_2 = slice.to_owned() + &"0".repeat(str_len - sf as usize);
    return var_2.parse::<u128>().unwrap();
} 

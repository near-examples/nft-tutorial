use serde_json::json;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const DEFAULT_DEPOSIT: u128 = 6760000000000000000000 as u128;
// const DEFAULT_GAS: u128 = u128(9050000000000000000000);

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
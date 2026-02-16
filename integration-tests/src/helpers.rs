use std::sync::Arc;

use near_api::{
    errors::ExecuteTransactionError, types::transaction::result::ExecutionFinalResult, Account,
    AccountId, Contract, NearToken, NetworkConfig, Signer,
};
use near_sandbox::Sandbox;
use near_sdk::Gas;
use serde_json::json;

const INITIAL_BALANCE: NearToken = NearToken::from_near(30);
pub const DEFAULT_DEPOSIT: u128 = 10000000000000000000000;
pub const ONE_YOCTO_NEAR: NearToken = NearToken::from_yoctonear(1);

pub async fn init_sandbox() -> anyhow::Result<(Sandbox, NetworkConfig)> {
    // Initialize the sandbox
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    Ok((sandbox, sandbox_network))
}

pub async fn init_contracts(
    sandbox: &Sandbox,
    sandbox_network: &NetworkConfig,
) -> anyhow::Result<(Contract, Contract, Arc<Signer>)> {
    // Build the NFT contract wasm file
    let nft_contract_wasm_path = cargo_near_build::build_with_cli(
        cargo_near_build::BuildOpts::builder()
            .manifest_path("../nft-contract-royalty/Cargo.toml")
            .build(),
    )
    .unwrap();
    let nft_contract_wasm = std::fs::read(nft_contract_wasm_path)?;

    // Build the Market contract wasm file
    let market_contract_wasm_path = cargo_near_build::build_with_cli(
        cargo_near_build::BuildOpts::builder()
            .manifest_path("../market-contract/Cargo.toml")
            .build(),
    )
    .unwrap();
    let market_contract_wasm = std::fs::read(market_contract_wasm_path)?;

    // Create the NFT and Market contracts accounts
    let nft_contract = create_subaccount(&sandbox, "nft-contract.sandbox")
        .await
        .unwrap()
        .as_contract();
    let market_contract = create_subaccount(&sandbox, "market-contract.sandbox")
        .await
        .unwrap()
        .as_contract();

    // Initialize signer for the contract deployment
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    // Deploy the ft contract
    near_api::Contract::deploy(nft_contract.account_id().clone())
        .use_code(nft_contract_wasm.to_vec())
        .with_init_call(
            "new_default_meta",
            json!({"owner_id": nft_contract.account_id().clone()}),
        )?
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    near_api::Contract::deploy(market_contract.account_id().clone())
        .use_code(market_contract_wasm.to_vec())
        .with_init_call(
            "new",
            json!({"owner_id": market_contract.account_id().clone()}),
        )?
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    return Ok((nft_contract, market_contract, signer));
}

pub async fn init_accounts(sandbox: &Sandbox) -> anyhow::Result<(Account, Account, Account)> {
    // create accounts
    let alice = create_subaccount(&sandbox, "alice.sandbox").await.unwrap();
    let bob = create_subaccount(&sandbox, "bob.sandbox").await.unwrap();
    let charlie = create_subaccount(&sandbox, "charlie.sandbox")
        .await
        .unwrap();

    return Ok((alice, bob, charlie));
}

pub async fn mint_nft(
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "token_owner_id": user.account_id(),
        "token_metadata": {
            "title": "Grumpy Cat",
            "description": "Not amused.",
            "media": "https://www.adamsdrafting.com/wp-content/uploads/2018/06/More-Grumpy-Cat.jpg"
        },
    });

    let _ = nft_contract
        .call_function("nft_mint", json!(request_payload))
        .transaction()
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

pub async fn approve_nft(
    market_contract: &Contract,
    user: &Account,
    nft_contract: &Contract,
    token_id: &str,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "account_id": market_contract.as_account().account_id(),
        "msg": serde_json::Value::Null,
    });

    let _ = nft_contract
        .call_function("nft_approve", json!(request_payload))
        .transaction()
        .deposit(NearToken::from_yoctonear(DEFAULT_DEPOSIT))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

pub async fn pay_for_storage(
    user: &Account,
    market_contract: &Contract,
    amount: NearToken,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({});

    let _ = market_contract
        .call_function("storage_deposit", json!(request_payload))
        .transaction()
        .deposit(amount)
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

pub async fn place_nft_for_sale(
    user: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    token_id: &str,
    approval_id: u64,
    price: &NearToken,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "nft_contract_id": nft_contract.as_account().account_id(),
        "token_id": token_id,
        "approval_id": approval_id,
        "sale_conditions": NearToken::as_yoctonear(price).to_string(),
    });
    let _ = market_contract
        .call_function("list_nft_for_sale", json!(request_payload))
        .transaction()
        .gas(Gas::from_tgas(100))
        .with_signer(user.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

pub async fn get_user_balance(user: &Account, sandbox_network: &NetworkConfig) -> NearToken {
    user.tokens()
        .near_balance()
        .fetch_from(&sandbox_network)
        .await
        .expect("Failed to get user balance")
        .total
}

pub async fn purchase_listed_nft(
    bidder: &Account,
    market_contract: &Contract,
    nft_contract: &Contract,
    token_id: &str,
    offer_price: NearToken,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<ExecutionFinalResult, ExecuteTransactionError> {
    let request_payload = json!({
        "token_id": token_id,
        "nft_contract_id": nft_contract.as_account().account_id(),
    });

    market_contract
        .call_function("offer", json!(request_payload))
        .transaction()
        .deposit(offer_price)
        .max_gas()
        .with_signer(bidder.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await
}

pub async fn transfer_nft(
    sender: &Account,
    receiver: &Account,
    nft_contract: &Contract,
    token_id: &str,
    signer: &Arc<Signer>,
    sandbox_network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_payload = json!({
        "token_id": token_id,
        "receiver_id": receiver.account_id(),
        "approval_id": 1 as u64,
    });

    let _ = nft_contract
        .call_function("nft_transfer", json!(request_payload))
        .transaction()
        .deposit(ONE_YOCTO_NEAR)
        .with_signer(sender.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok(())
}

pub async fn get_nft_token_info(
    nft_contract: &Contract,
    token_id: &str,
    sandbox_network: &NetworkConfig,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let token_info: serde_json::Value = nft_contract
        .call_function("nft_token", json!({"token_id": token_id}))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    Ok(token_info)
}

pub fn round_to_near_dp(amount: u128, sf: u128) -> String {
    let near_amount = amount as f64 / 1_000_000_000_000_000_000_000_000.0; // yocto in 1 NEAR
    return format!("{:.1$}", near_amount, sf as usize);
}

pub async fn create_subaccount(
    sandbox: &near_sandbox::Sandbox,
    name: &str,
) -> testresult::TestResult<near_api::Account> {
    let account_id: AccountId = name.parse().unwrap();
    sandbox
        .create_account(account_id.clone())
        .initial_balance(INITIAL_BALANCE)
        .send()
        .await?;
    Ok(near_api::Account(account_id))
}

#![allow(clippy::bool_assert_comparison)]
use serde_json::json;

#[tokio::test]
async fn register_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = consumer_account
        .view(contract.id(), "is_registered_as_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert!(!outcome.json::<bool>().unwrap());

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "is_registered_as_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert!(outcome.json::<bool>().unwrap());

    Ok(())
}

#[tokio::test]
async fn register_producer() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;

    let outcome = producer_account
        .view(contract.id(), "is_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?;
    assert!(!outcome.json::<bool>().unwrap());

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "is_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?;
    assert!(outcome.json::<bool>().unwrap());

    Ok(())
}

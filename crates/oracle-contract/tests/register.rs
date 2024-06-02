use serde_json::json;

#[tokio::test]
async fn register_consumers() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = consumer_account
        .view(contract.id(), "is_registered_as_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<bool>().unwrap(), false);

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
    assert_eq!(outcome.json::<bool>().unwrap(), true);

    Ok(())
}

#[tokio::test]
async fn register_producers() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;

    let outcome = producer_account
        .view(contract.id(), "is_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<bool>().unwrap(), false);

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
    assert_eq!(outcome.json::<bool>().unwrap(), true);

    Ok(())
}

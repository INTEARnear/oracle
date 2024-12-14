use std::time::Duration;

use serde_json::json;

#[tokio::test]
async fn cannot_send_two_responses() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = &crate::CONTRACT_WASM;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    tokio::spawn(
        consumer_account
            .call(contract.id(), "request")
            .args_json(json!({
                "producer_id": producer_account.id(),
                "request_data": "Hello World!",
            }))
            .transact(),
    );
    tokio::time::sleep(Duration::from_secs(1)).await;

    let outcome = producer_account
        .call(contract.id(), "respond")
        .max_gas()
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "Hello Yielded Execution!",
            }
        }))
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_success());

    let outcome = producer_account
        .call(contract.id(), "respond")
        .max_gas()
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "Hello Yielded Execution 2!",
            }
        }))
        .transact()
        .await
        .unwrap();
    assert!(outcome.is_failure());

    Ok(())
}

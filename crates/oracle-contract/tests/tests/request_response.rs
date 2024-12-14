use serde_json::json;

#[tokio::test]
async fn contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

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

    let request = consumer_account
        .call(contract.id(), "request")
        .args_json(json!({
            "producer_id": producer_account.id(),
            "request_data": "Hello World!",
        }))
        .transact_async()
        .await?;

    sandbox.fast_forward(1).await?;

    let outcome = producer_account
        .call(contract.id(), "respond")
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "Hello Yielded Execution!",
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let request_result = request.await?;
    assert!(request_result.is_success());

    let logs = request_result.logs();
    assert_eq!(logs, vec![
        format!("EVENT_JSON:{{\"standard\":\"intear-oracle\",\"version\":\"1.0.0\",\"event\":\"request\",\"data\":{{\"producer_id\":\"{producer}\",\"consumer_id\":\"{consumer}\",\"request_id\":\"0\",\"request_data\":\"Hello World!\"}}}}", 
            producer = producer_account.id(),
            consumer = consumer_account.id()
        ),
        format!("Response from {producer} for 0: Ok(\"Hello Yielded Execution!\"), refund Ok(None)", 
            producer = producer_account.id()
        ),
    ]);

    Ok(())
}

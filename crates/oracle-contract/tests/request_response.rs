use serde_json::json;

#[tokio::test]
async fn contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

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

    let request = async {
        let outcome = consumer_account
            .call(contract.id(), "request")
            .args_json(json!({
                "producer_id": producer_account.id(),
                "request_data": "Hello World!",
            }))
            .transact()
            .await
            .unwrap();
        assert!(outcome.is_success());
        assert_eq!(outcome.logs(), vec![
            format!("EVENT_JSON:{{\"standard\":\"intear-oracle\",\"version\":\"1.0.0\",\"event\":\"request\",\"data\":{{\"consumer_id\":\"{consumer}\",\"request_id\":\"0\",\"request_data\":\"Hello World!\"}}}}", consumer = consumer_account.id()),
            format!("Response from {producer} for 0: Some(\"Hello Yielded Execution!\")", producer = producer_account.id()),
        ]);
    };
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(5));
    let response = async {
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
    };
    let response = async {
        timeout.await;
        response.await;
    };
    tokio::join!(request, response);

    Ok(())
}

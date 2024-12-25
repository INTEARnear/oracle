use intear_oracle::fees::ProducerFee;
use near_sdk::NearToken;
use serde_json::json;

#[tokio::test]
async fn example_consumer_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let oracle_contract_wasm = crate::get_contract_wasm().await;
    let example_consumer_contract_wasm = crate::get_example_consumer_contract_wasm().await;

    let contract = sandbox.dev_deploy(oracle_contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let example_consumer_contract = sandbox.dev_deploy(example_consumer_contract_wasm).await?;
    let consumer_account = sandbox.dev_create_account().await?;

    let init_outcome = example_consumer_contract
        .as_account()
        .call(example_consumer_contract.id(), "new")
        .args_json(json!({
            "oracle_contract": contract.id(),
            "producer_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(init_outcome.is_success());

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::Near {
                prepaid_amount: NearToken::from_millinear(10), // 0.01 NEAR
            },
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let request_yes = consumer_account
        .call(example_consumer_contract.id(), "test_statement")
        .max_gas()
        .args_json(json!({
            "statement": "Is slime solid?",
        }))
        .transact_async()
        .await?;

    sandbox.fast_forward(49).await?; // no idea why that many blocks are needed, 49 works but 48 doesn't

    let outcome = producer_account
        .call(contract.id(), "respond")
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "No, slime is slimy",
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let request_result = request_yes.await?;
    assert!(!request_result.clone().into_result()?.json::<bool>()?);

    let logs = request_result.logs();
    assert_eq!(logs, vec![
        format!("EVENT_JSON:{{\"standard\":\"intear-oracle\",\"version\":\"1.0.0\",\"event\":\"request\",\"data\":{{\"producer_id\":\"{producer}\",\"consumer_id\":\"{consumer}\",\"request_id\":\"0\",\"request_data\":\"Is slime solid?\"}}}}",
            producer = producer_account.id(),
            consumer = example_consumer_contract.id()
        ),
        format!("Response from {producer} for 0: Ok(\"No, slime is slimy\"), refund Ok(None)",
            producer = producer_account.id()
        ),
    ]);

    Ok(())
}

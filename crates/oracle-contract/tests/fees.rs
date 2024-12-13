use intear_oracle::fees::ProducerFee;
use near_sdk::NearToken;
use near_workspaces::result::ValueOrReceiptId;
use serde_json::json;

#[tokio::test]
async fn no_fee() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<ProducerFee>().unwrap(), ProducerFee::None);

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::None,
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<ProducerFee>().unwrap(), ProducerFee::None);

    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(0)
    );

    let outcome = consumer_account
        .call(contract.id(), "deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(1)
    );

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
                "response_data": "Hello World!",
                // no refund
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(1)
    );

    let response_is_correct = request
        .await?
        .outcomes()
        .into_iter()
        .cloned()
        .map(|outcome| outcome.into_result().unwrap())
        .any(|outcome| {
            if let ValueOrReceiptId::Value(value) = outcome {
                value.json::<serde_json::Value>().unwrap()
                    == serde_json::json!({
                        "response_data": "Hello World!",
                        "refund_amount": null,
                    })
            } else {
                false
            }
        });
    assert!(response_is_correct);

    Ok(())
}

#[tokio::test]
async fn near_fee() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<ProducerFee>().unwrap(), ProducerFee::None);

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::Near {
                prepaid_amount: NearToken::from_millinear(100), // 0.1 NEAR
            },
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<ProducerFee>().unwrap(),
        ProducerFee::Near {
            prepaid_amount: NearToken::from_millinear(100),
        }
    );

    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(0)
    );

    let outcome = consumer_account
        .call(contract.id(), "deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(1)
    );

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
                // no refund
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_millinear(1000 - 100) // 0.9 NEAR
    );

    let response_is_correct = request
        .await?
        .outcomes()
        .into_iter()
        .cloned()
        .map(|outcome| outcome.into_result().unwrap())
        .any(|outcome| {
            if let ValueOrReceiptId::Value(value) = outcome {
                value.json::<serde_json::Value>().ok()
                    == Some(serde_json::json!({
                        "response_data": "Hello Yielded Execution!",
                        "refund_amount": null,
                    }))
            } else {
                false
            }
        });
    assert!(response_is_correct);

    sandbox.fast_forward(2).await?;

    let producer_new_balance = producer_account.view_account().await?.balance;
    assert!(
        producer_new_balance
            .checked_sub(producer_initial_balance)
            .unwrap()
            < NearToken::from_millinear(100)
    );

    Ok(())
}

#[tokio::test]
async fn near_fee_refund() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<ProducerFee>().unwrap(), ProducerFee::None);

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::Near {
                prepaid_amount: NearToken::from_millinear(100), // 0.1 NEAR
            },
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_fee")
        .args_json(json!({
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<ProducerFee>().unwrap(),
        ProducerFee::Near {
            prepaid_amount: NearToken::from_millinear(100),
        }
    );

    let consumer_account = sandbox.dev_create_account().await?;

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(0)
    );

    let outcome = consumer_account
        .call(contract.id(), "deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(1)
    );

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
                "refund_amount": NearToken::from_millinear(50).as_yoctonear().to_string(), // 0.05 NEAR
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_millinear(1000 - 100) // 0.9 NEAR
    );

    let response_is_correct = request
        .await?
        .outcomes()
        .into_iter()
        .cloned()
        .map(|outcome| outcome.into_result().unwrap())
        .any(|outcome| {
            if let ValueOrReceiptId::Value(value) = outcome {
                value.json::<serde_json::Value>().ok()
                    == Some(serde_json::json!({
                        "response_data": "Hello Yielded Execution!",
                        "refund_amount": "50000000000000000000000"
                    }))
            } else {
                false
            }
        });
    assert!(response_is_correct);

    sandbox.fast_forward(2).await?;

    let producer_new_balance = producer_account.view_account().await?.balance;
    assert!(
        producer_new_balance
            .checked_sub(producer_initial_balance)
            .unwrap()
            < NearToken::from_millinear(100 - 50)
    );

    Ok(())
}

#[tokio::test]
async fn direct_near_fee() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox_with_version("2.4.0").await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::Near {
                prepaid_amount: NearToken::from_millinear(100), // 0.1 NEAR
            },
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let consumer_account = sandbox.dev_create_account().await?;
    let consumer_initial_balance = consumer_account.view_account().await?.balance;

    let request = consumer_account
        .call(contract.id(), "request")
        .args_json(json!({
            "producer_id": producer_account.id(),
            "request_data": "Hello World!",
        }))
        .deposit(NearToken::from_millinear(100)) // 0.1 NEAR
        .transact_async()
        .await
        .unwrap();

    sandbox.fast_forward(1).await?;

    let outcome = producer_account
        .call(contract.id(), "respond")
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "Hello World",
                "refund_amount": NearToken::from_millinear(50).as_yoctonear().to_string(), // 0.05 NEAR
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let response_is_correct = request
        .await?
        .outcomes()
        .into_iter()
        .cloned()
        .map(|outcome| outcome.into_result().unwrap())
        .any(|outcome| {
            if let ValueOrReceiptId::Value(value) = outcome {
                value.json::<serde_json::Value>().unwrap()
                    == serde_json::json!({
                        "response_data": "Hello World",
                        "refund_amount": "50000000000000000000000"
                    })
            } else {
                false
            }
        });
    assert!(response_is_correct);

    sandbox.fast_forward(2).await?;

    let producer_new_balance = producer_account.view_account().await?.balance;
    assert!(
        producer_new_balance
            .checked_sub(producer_initial_balance)
            .unwrap()
            > NearToken::from_millinear(40) // Should receive the 0.05 NEAR fee
    );

    let consumer_new_balance = consumer_account.view_account().await?.balance;
    assert!(consumer_initial_balance.as_millinear() - consumer_new_balance.as_millinear() > 5);

    Ok(())
}

// TODO more refund tests

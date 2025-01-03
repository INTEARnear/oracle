use intear_oracle::fees::ProducerFee;
use near_sdk::{json_types::U128, NearToken};
use near_workspaces::result::ValueOrReceiptId;
use serde_json::json;

#[tokio::test]
async fn no_fee() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
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
                value.json::<serde_json::Value>().ok()
                    == Some(serde_json::json!({
                        "response_data": "Hello World!",
                        "refund_amount": null,
                    }))
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
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
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
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
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

    sandbox.fast_forward(1).await?;

    let outcome = producer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_millinear(1000 - 100 + 50) // 0.95 NEAR
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
    let contract_wasm = crate::get_contract_wasm().await;
    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account.view_account().await?.balance;

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
                value.json::<serde_json::Value>().ok()
                    == Some(serde_json::json!({
                        "response_data": "Hello World",
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
            > NearToken::from_millinear(40) // Should receive the 0.05 NEAR fee
    );

    let consumer_new_balance = consumer_account.view_account().await?.balance;
    assert!(consumer_initial_balance.as_millinear() - consumer_new_balance.as_millinear() > 5);

    Ok(())
}

#[tokio::test]
async fn ft_fee() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let token_wasm = crate::get_ft_contract_wasm().await;
    let token = sandbox.dev_deploy(token_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account
        .view(token.id(), "ft_balance_of")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?
        .json::<U128>()
        .unwrap()
        .0;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
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
            "fee": ProducerFee::FungibleToken {
                token: token.id().clone(),
                prepaid_amount: (10u128.pow(24) / 10).into(), // 0.1 NEAR
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
        ProducerFee::FungibleToken {
            token: token.id().clone(),
            prepaid_amount: (10u128.pow(24) / 10).into(),
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
        .view(contract.id(), "get_deposit_ft")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "ft_id": token.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(0)
    );

    let outcome = consumer_account
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": contract.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "mint")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "amount": "1000000000000000000000000000",
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "ft_transfer_call")
        .max_gas()
        .args_json(json!({
            "receiver_id": contract.id(),
            "amount": U128(10u128.pow(24)),
            "msg": "{}",
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_ft")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "ft_id": token.id(),
        }))
        .await?;
    assert_eq!(outcome.json::<U128>().unwrap(), 10u128.pow(24).into());

    let request = consumer_account
        .call(contract.id(), "request")
        .max_gas()
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
                "response_data": "Hello World",
                // no refund
            }
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(dbg!(&outcome).is_success());

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_ft")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "ft_id": token.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<U128>().unwrap(),
        (10u128.pow(24) - 10u128.pow(24) / 10).into()
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
                        "response_data": "Hello World",
                        "refund_amount": null,
                    }))
            } else {
                false
            }
        });
    assert!(response_is_correct);

    sandbox.fast_forward(2).await?;

    let producer_new_balance = producer_account
        .view(token.id(), "ft_balance_of")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?
        .json::<U128>()
        .unwrap()
        .0;
    assert_eq!(
        producer_new_balance
            .checked_sub(producer_initial_balance)
            .unwrap(),
        (10u128.pow(24) / 10)
    );

    Ok(())
}

#[tokio::test]
async fn ft_fee_refund() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = crate::get_contract_wasm().await;

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let token_wasm = crate::get_ft_contract_wasm().await;
    let token = sandbox.dev_deploy(token_wasm).await?;

    let producer_account = sandbox.dev_create_account().await?;
    let producer_initial_balance = producer_account
        .view(token.id(), "ft_balance_of")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?
        .json::<U128>()
        .unwrap()
        .0;

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .call(contract.id(), "set_fee")
        .args_json(json!({
            "fee": ProducerFee::FungibleToken {
                token: token.id().clone(),
                prepaid_amount: (10u128.pow(24) / 10).into(), // 0.1 NEAR
            },
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

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
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": contract.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "mint")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "amount": "1000000000000000000000000000",
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(token.id(), "ft_transfer_call")
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "receiver_id": contract.id(),
            "amount": "1000000000000000000000000", // 1 token
            "msg": "{}",
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let request = consumer_account
        .call(contract.id(), "request")
        .max_gas()
        .args_json(json!({
            "producer_id": producer_account.id(),
            "request_data": "Hello World",
        }))
        .transact_async()
        .await?;

    sandbox.fast_forward(1).await?;

    let outcome = producer_account
        .call(contract.id(), "respond")
        .args_json(json!({
            "request_id": "0",
            "response": {
                "response_data": "Hello World",
                "refund_amount": "50000000000000000000000", // 0.5 tokens
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
                value.json::<serde_json::Value>().ok()
                    == Some(serde_json::json!({
                        "response_data": "Hello World",
                        "refund_amount": "50000000000000000000000",
                    }))
            } else {
                false
            }
        });
    assert!(response_is_correct);

    sandbox.fast_forward(2).await?;

    let producer_new_balance = producer_account
        .view(token.id(), "ft_balance_of")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .await?
        .json::<U128>()
        .unwrap()
        .0;
    assert_eq!(
        producer_new_balance - producer_initial_balance,
        50000000000000000000000
    );

    Ok(())
}

// TODO more refund tests

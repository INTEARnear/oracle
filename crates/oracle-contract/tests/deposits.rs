use near_sdk::NearToken;
use serde_json::json;

#[tokio::test]
async fn near_deposits() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let consumer_account = sandbox.dev_create_account().await?;
    let initial_balance = consumer_account.view_account().await?.balance;

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
    assert_eq!(
        outcome.logs(),
        vec![format!(
            "Deposited 1.00 NEAR to {consumer_id}",
            consumer_id = consumer_account.id()
        )]
    );

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

    let outcome = consumer_account
        .call(contract.id(), "deposit_near")
        .args_json(json!({}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());
    assert_eq!(
        outcome.logs(),
        vec![format!(
            "Deposited 1.00 NEAR to {consumer_id}",
            consumer_id = consumer_account.id()
        )]
    );

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(2)
    );

    let new_balance = consumer_account.view_account().await?.balance;
    assert!(
        initial_balance
            .checked_sub(new_balance)
            .unwrap()
            .checked_sub(NearToken::from_near(2))
            .unwrap()
            < NearToken::from_millinear(5) // gas fee
    );

    let outcome = consumer_account
        .call(contract.id(), "withdraw_near")
        .args_json(json!({
            "amount": NearToken::from_near(1),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());
    assert_eq!(
        outcome.logs(),
        vec![format!(
            "Withdrew 1.00 NEAR from {account_id}",
            account_id = consumer_account.id()
        )]
    );

    let new_balance = consumer_account.view_account().await?.balance;
    assert!(
        initial_balance
            .checked_sub(new_balance)
            .unwrap()
            .checked_sub(NearToken::from_near(2 - 1))
            .unwrap()
            < NearToken::from_millinear(5) // gas fee
    );

    let outcome = consumer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(2 - 1)
    );

    Ok(())
}

#[tokio::test]
async fn near_deposit_for_producer() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = include_bytes!("../../../target/near/intear_oracle/intear_oracle.wasm");

    let contract = sandbox.dev_deploy(contract_wasm).await?;

    let consumer_account = sandbox.dev_create_account().await?;
    let producer_account = sandbox.dev_create_account().await?;
    let initial_balance = consumer_account.view_account().await?.balance;

    let outcome = consumer_account
        .call(contract.id(), "register_consumer")
        .args_json(json!({
            "account_id": consumer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = consumer_account
        .call(contract.id(), "deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "producer_id": producer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_failure());

    let outcome = producer_account
        .call(contract.id(), "add_producer")
        .args_json(json!({
            "account_id": producer_account.id(),
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = producer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "producer_id": producer_account.id(),
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
            "producer_id": producer_account.id(),
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(outcome.is_success());
    assert_eq!(
        outcome.logs(),
        vec![format!(
            "Deposited 1.00 NEAR to {account_id} for {producer_id}",
            account_id = consumer_account.id(),
            producer_id = producer_account.id(),
        )]
    );

    let outcome = producer_account
        .view(contract.id(), "get_deposit_near")
        .args_json(json!({
            "account_id": consumer_account.id(),
            "producer_id": producer_account.id(),
        }))
        .await?;
    assert_eq!(
        outcome.json::<NearToken>().unwrap(),
        NearToken::from_near(1)
    );

    let new_balance = consumer_account.view_account().await?.balance;
    assert!(
        initial_balance
            .checked_sub(new_balance)
            .unwrap()
            .checked_sub(NearToken::from_near(1))
            .unwrap()
            < NearToken::from_millinear(5) // gas fee
    );

    Ok(())
}

// TODO ft deposits

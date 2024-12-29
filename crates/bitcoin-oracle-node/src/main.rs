mod proof_extractor;
mod types;

use std::sync::Arc;

use anyhow::Result;
use inevents_websocket_client::EventStreamClient;
use intear_events::events::log::log_nep297::LogNep297Event;
use json_filter::{Filter, Operator};
use log::{error, info, warn};
use near_api::prelude::{Account, Contract, NetworkConfig};
use near_api::signer::secret_key::SecretKeySigner;
use near_api::signer::Signer;
use near_gas::NearGas;
use near_primitives::types::AccountId;
use near_token::NearToken;
use proof_extractor::get_transaction_proof;
use serde::Deserialize;

const ORACLE_CONTRACT: &str = "dev-unaudited-v1.oracle.intear.near";

#[derive(Debug, Deserialize)]
struct OracleRequestEvent {
    producer_id: AccountId,
    #[allow(unused)]
    consumer_id: AccountId,
    request_id: String,
    request_data: String,
}

#[derive(Clone)]
struct BitcoinNode {
    account: Account,
    contract: Contract,
    signer: Arc<Signer>,
    bitcoin_rpc_url: String,
}

impl BitcoinNode {
    async fn handle_request(&self, event: OracleRequestEvent) -> Result<()> {
        let tx_id_str = &event.request_data;
        info!("Processing Bitcoin transaction: {tx_id_str}");

        let mut attempts = 0;
        let proof = loop {
            attempts += 1;
            match get_transaction_proof(tx_id_str, &self.bitcoin_rpc_url).await {
                Ok(proof) => break proof,
                Err(e) => {
                    if attempts >= 5 {
                        error!("Failed to get transaction proof after 5 attempts for tx {tx_id_str}: {e:?}");
                        return Ok(());
                    }
                    warn!("Attempt {attempts}/5 failed to get transaction proof for tx {tx_id_str}: {e:?}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        };

        info!("proof: {proof:?}");
        let tx_hash = self
            .contract
            .call_function(
                "submit",
                serde_json::json!({
                    "request_id": event.request_id,
                    "transaction_details": proof,
                }),
            )?
            .transaction()
            .gas(NearGas::from_tgas(300))
            .deposit(NearToken::from_yoctonear(0))
            .with_signer(self.account.0.clone(), self.signer.clone())
            .with_retries(5)
            .send_to(&NetworkConfig {
                rpc_url: "https://rpc.shitzuapes.xyz".parse().unwrap(),
                ..NetworkConfig::mainnet()
            })
            .await?
            .transaction
            .hash;

        info!("Responded to oracle request: {tx_hash}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        error!("Failed to load .env file: {}", e);
    }

    env_logger::init();

    let oracle = Arc::new(BitcoinNode {
        account: Account(
            std::env::var("ACCOUNT_ID")
                .expect("No ACCOUNT_ID environment variable")
                .parse()
                .expect("ACCOUNT_ID environment variable is invalid"),
        ),
        contract: Contract(
            std::env::var("CONTRACT_ID")
                .expect("No CONTRACT_ID environment variable")
                .parse()
                .expect("CONTRACT_ID environment variable is invalid"),
        ),
        signer: Signer::new(SecretKeySigner::new(
            std::env::var("PRIVATE_KEY")
                .expect("No PRIVATE_KEY environment variable")
                .parse()
                .expect("PRIVATE_KEY environment variable is invalid"),
        ))
        .expect("Failed to create a signer"),
        bitcoin_rpc_url: std::env::var("BITCOIN_RPC_URL")
            .expect("No BITCOIN_RPC_URL environment variable"),
    });

    let client = EventStreamClient::default();
    client
        .stream_events::<LogNep297Event, _, _>(
            "log_nep297",
            Some(Operator::And(vec![
                Filter {
                    path: "account_id".to_string(),
                    operator: Operator::Equals(serde_json::Value::String(
                        ORACLE_CONTRACT.to_string(),
                    )),
                },
                Filter {
                    path: "event_standard".to_string(),
                    operator: Operator::Equals(serde_json::Value::String(
                        "intear-oracle".to_string(),
                    )),
                },
                Filter {
                    path: "event_event".to_string(),
                    operator: Operator::Equals(serde_json::Value::String("request".to_string())),
                },
            ])),
            move |event| {
                let oracle = oracle.clone();
                async move {
                    if let Some(event_data) = event.event_data {
                        if let Ok(request) =
                            serde_json::from_value::<OracleRequestEvent>(event_data)
                        {
                            if request.producer_id == oracle.contract.0 {
                                if let Err(err) = oracle.handle_request(request).await {
                                    error!("Failed to handle request: {err:?}");
                                }
                            }
                        }
                    }
                }
            },
        )
        .await;

    Ok(())
}

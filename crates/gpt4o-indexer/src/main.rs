use std::sync::Arc;

use anyhow::Result;
use inevents_websocket_client::EventStreamClient;
use intear_events::events::log::log_nep297::LogNep297Event;
use json_filter::{Filter, Operator};
use log::{error, info};
use near_api::prelude::{Account, Contract, NetworkConfig};
use near_api::signer::secret_key::SecretKeySigner;
use near_api::signer::Signer;
use near_gas::NearGas;
use near_primitives::serialize::dec_format;
use near_primitives::types::{AccountId, Balance};
use near_token::NearToken;
use serde::{Deserialize, Serialize};

const MODEL: &str = "gpt-4o";
const ORACLE_CONTRACT: &str = "dev-unaudited-v1.oracle.intear.near";

#[derive(Debug, Deserialize)]
struct OracleRequestEvent {
    #[allow(dead_code)]
    producer_id: AccountId,
    consumer_id: AccountId,
    #[serde(with = "dec_format")]
    request_id: u128,
    request_data: String,
}

#[derive(Debug, Serialize)]
struct OracleResponse {
    #[serde(with = "dec_format")]
    request_id: u128,
    response: Response,
}

#[derive(Debug, Serialize)]
struct Response {
    response_data: String,
    #[serde(with = "dec_format")]
    refund_amount: Option<Balance>,
}

#[derive(Clone)]
struct GptOracle {
    model: &'static str,
    account: Account,
    signer: Arc<Signer>,
    openai_api_key: String,
}

impl GptOracle {
    async fn handle_request(&self, event: OracleRequestEvent) -> Result<()> {
        let msg = event.request_data;
        info!("Request: {msg}");

        let response = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "user": event.consumer_id,
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a helpful assistant developed by Intear that uses OpenAI GPT model. Your responses are clear and concise, up to 2-3 sentences, and strictly at most 600 characters."
                    },
                    {
                        "role": "user",
                        "content": msg
                    }
                ]
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let response = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format from OpenAI"))?
            .to_owned();

        info!("Response: {response}");

        let tx_hash = Contract(ORACLE_CONTRACT.parse()?)
            .call_function(
                "respond",
                OracleResponse {
                    request_id: event.request_id,
                    response: Response {
                        response_data: response,
                        refund_amount: None,
                    },
                },
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

    let oracle = GptOracle {
        model: MODEL,
        account: Account(
            std::env::var("ACCOUNT_ID")
                .expect("No ACCOUNT_ID environment variable")
                .parse()
                .expect("ACCOUNT_ID environment variable is invalid"),
        ),
        signer: Signer::new(SecretKeySigner::new(
            std::env::var("PRIVATE_KEY")
                .expect("No PRIVATE_KEY environment variable")
                .parse()
                .expect("PRIVATE_KEY environment variable is invalid"),
        ))
        .expect("Failed to create a signer"),
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .expect("No OPENAI_API_KEY environment variable"),
    };

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
                Filter {
                    path: "event_data.producer_id".to_string(),
                    operator: Operator::Equals(serde_json::Value::String(
                        oracle.account.0.to_string(),
                    )),
                },
            ])),
            move |event| {
                let oracle = oracle.clone();
                async move {
                    if let Some(event_data) = event.event_data {
                        if let Ok(request) =
                            serde_json::from_value::<OracleRequestEvent>(event_data)
                        {
                            if let Err(err) = oracle.handle_request(request).await {
                                error!("Failed to handle request: {err:?}");
                            }
                        }
                    }
                }
            },
        )
        .await;

    Ok(())
}

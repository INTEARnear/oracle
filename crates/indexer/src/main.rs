use std::sync::Arc;

use async_trait::async_trait;
use inindexer::{
    fastnear_data_server::FastNearDataServerProvider,
    near_indexer_primitives::{types::Balance, StreamerMessage},
    near_utils::{dec_format, EventLogData},
    run_indexer, AutoContinue, BlockIterator, IncompleteTransaction, Indexer, IndexerOptions,
    TransactionReceipt,
};
use log::LevelFilter;
use near_api::signer::Signer;
use near_api::{
    prelude::{Account, Contract},
    signer::secret_key::SecretKeySigner,
};
use near_gas::NearGas;
use near_primitives::types::AccountId;
use near_token::NearToken;
use serde::{Deserialize, Serialize};

const MODEL: &str = "gpt-4o";
const ORACLE_CONTRACT: &str = "dev-unaudited-v0.oracle.intear.near";

#[derive(Debug, Deserialize)]
struct OracleRequestEvent {
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

struct GptIndexer {
    model: &'static str,
    account: Account,
    signer: Arc<Signer>,
    openai_api_key: String,
}

#[async_trait]
impl Indexer for GptIndexer {
    type Error = String;

    async fn on_receipt(
        &mut self,
        receipt: &TransactionReceipt,
        _transaction: &IncompleteTransaction,
        _block: &StreamerMessage,
    ) -> Result<(), Self::Error> {
        if receipt.receipt.receipt.receiver_id == ORACLE_CONTRACT {
            for log in receipt.receipt.execution_outcome.outcome.logs.iter() {
                if let Ok(event) = EventLogData::<OracleRequestEvent>::deserialize(log) {
                    if event.standard == "intear-oracle"
                        && event.event == "request"
                        && event.parse_semver().map_or(false, |v| v.major == 1)
                        && event.data.producer_id == self.account.0
                    {
                        let msg = event.data.request_data;
                        log::info!("Request: {msg}");

                        let account = self.account.clone();
                        let openai_api_key = self.openai_api_key.clone();
                        let model = self.model.to_owned();
                        let signer = self.signer.clone();
                        tokio::spawn(async move {
                            let result: Result<_, anyhow::Error> = async move {
                                let response = reqwest::Client::new()
                                    .post("https://api.openai.com/v1/chat/completions")
                                    .header("Authorization", format!("Bearer {}", openai_api_key))
                                    .json(&serde_json::json!({
                                        "model": model,
                                        "user": event.data.consumer_id,
                                        "messages": [
                                            {
                                                "role": "system",
                                                "content": "You are a helpful assistant developed by Intear that uses OpenAI GPT model. Your responses are clear and concise, up to 2-3 sentences."
                                            },
                                            {
                                                "role": "user",
                                                "content": msg
                                            }
                                        ]
                                    }))
                                    .send()
                                    .await
                                    .map_err(|err| anyhow::anyhow!("Error sending request to OpenAI: {err:?}"))?
                                    .json::<serde_json::Value>()
                                    .await
                                    // TODO this will definitely panic, better to define a struct and better error handling
                                    .map_err(|err| anyhow::anyhow!("Error parsing response from OpenAI: {err:?}"))?
                                    ["choices"]
                                    [0]
                                    ["message"]
                                    ["content"]
                                    .as_str()
                                    .unwrap()
                                    .to_owned();
                                log::info!("Response: {response}");
                                Ok(Contract(ORACLE_CONTRACT.parse().unwrap())
                                    .call_function(
                                        "respond",
                                        OracleResponse {
                                            request_id: event.data.request_id,
                                            response: Response {
                                                response_data: response,
                                                refund_amount: None, // TODO price by input/output tokens?
                                            },
                                        }
                                    )
                                    .map_err(|err| anyhow::anyhow!("Error calling function: {err:?}"))?
                                    .transaction()
                                    .gas(NearGas::from_tgas(300))
                                    .deposit(NearToken::from_yoctonear(0))
                                    .with_signer(account.0.clone(), signer)
                                    .with_retries(5)
                                    .send_to_mainnet()
                                    .await
                                    .map_err(|err| anyhow::anyhow!("Error sending transaction: {err:?}"))?
                                    .transaction
                                    .hash
                                )
                            }
                            .await;
                            match result {
                                Ok(tx) => log::info!("Responded to oracle request: {tx}"),
                                Err(err) => {
                                    log::error!("Failed to respond to oracle request: {err:?}")
                                }
                            }
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("inindexer::performance", LevelFilter::Debug)
        .env()
        .init()
        .unwrap();
    let mut indexer = GptIndexer {
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
    run_indexer(
        &mut indexer,
        FastNearDataServerProvider::mainnet(),
        IndexerOptions {
            range: BlockIterator::AutoContinue(AutoContinue::default()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
}

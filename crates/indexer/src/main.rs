use std::sync::Arc;

use async_trait::async_trait;
use inindexer::{
    fastnear_data_server::FastNearDataServerProvider,
    near_indexer_primitives::{types::Balance, StreamerMessage},
    near_utils::{dec_format, EventLogData, TESTNET_GENESIS_BLOCK_HEIGHT},
    run_indexer, AutoContinue, BlockIterator, IncompleteTransaction, Indexer, IndexerOptions,
    TransactionReceipt,
};
use log::LevelFilter;
use near_api_lib::{
    primitives::{hash::CryptoHash, types::AccountId},
    Account, InMemorySigner, JsonRpcProvider,
};
use serde::{Deserialize, Serialize};

const MODEL: &str = "gpt-4o";
const ORACLE_CONTRACT: &str = "yielded-oracle.testnet";

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
                        && event.data.producer_id == self.account.account_id
                    {
                        let msg = event.data.request_data;
                        log::info!("Request: {msg}");
                        let account = Account::new(
                            self.account.account_id.clone(),
                            Arc::clone(&self.account.signer),
                            Arc::clone(&self.account.provider),
                        );
                        let openai_api_key = self.openai_api_key.clone();
                        let model = self.model.to_owned();
                        tokio::spawn(async move {
                            let result: Result<CryptoHash, anyhow::Error> = async move {
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
                                    .map_err(|err| anyhow::anyhow!("Error parsing response from OpenAI: {err:?}"))?
                                    ["choices"]
                                    [0]
                                    ["message"]
                                    ["content"]
                                    .as_str()
                                    .unwrap()
                                    .to_owned();
                                log::info!("Response: {response}");
                                Ok(account
                                    .function_call(
                                        &ORACLE_CONTRACT.parse().unwrap(),
                                        "respond".to_owned(),
                                        serde_json::to_value(OracleResponse {
                                            request_id: event.data.request_id,
                                            response: Response {
                                                response_data: response,
                                                refund_amount: None,
                                            },
                                        })
                                        .unwrap(),
                                        300_000_000_000_000,
                                        0,
                                    )
                                    .await
                                    .map_err(|err| {
                                        anyhow::anyhow!("Error signing transaction: {err:?}")
                                    })?
                                    .transact()
                                    .await
                                    .map_err(|err| {
                                        anyhow::anyhow!("Error broadcasting transaction: {err:?}")
                                    })?
                                    .final_execution_outcome
                                    .ok_or_else(|| anyhow::anyhow!("No final execution outcome"))?
                                    .into_outcome()
                                    .transaction
                                    .hash)
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
        account: Account::new(
            std::env::var("ACCOUNT_ID")
                .expect("No ACCOUNT_ID environment variable")
                .parse()
                .expect("ACCOUNT_ID environment variable is invalid"),
            Arc::new(InMemorySigner::from_secret_key(
                std::env::var("ACCOUNT_ID")
                    .expect("No ACCOUNT_ID environment variable")
                    .parse()
                    .expect("ACCOUNT_ID environment variable is invalid"),
                std::env::var("PRIVATE_KEY")
                    .expect("No PRIVATE_KEY environment variable")
                    .parse()
                    .expect("PRIVATE_KEY environment variable is invalid"),
            )),
            Arc::new(JsonRpcProvider::new("https://rpc.testnet.near.org")),
        ),
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .expect("No OPENAI_API_KEY environment variable"),
    };
    run_indexer(
        &mut indexer,
        FastNearDataServerProvider::testnet(),
        IndexerOptions {
            range: BlockIterator::AutoContinue(AutoContinue {
                start_height_if_does_not_exist: TESTNET_GENESIS_BLOCK_HEIGHT,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .await
    .unwrap();
}

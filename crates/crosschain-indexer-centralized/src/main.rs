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
use near_api::prelude::{Account, Contract};
use near_api::signer::secret_key::SecretKeySigner;
use near_api::signer::Signer;
use near_gas::NearGas;
use near_primitives::types::AccountId;
use near_token::NearToken;
use reqwest::Url;
use serde::{Deserialize, Serialize};

const ORACLE_CONTRACT: &str = "dev-unaudited-v1.oracle.intear.near";

#[derive(Debug, Deserialize)]
struct OracleRequestEvent {
    producer_id: AccountId,
    #[allow(dead_code)]
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

struct CrosschainIndexer {
    account: Account,
    signer: Arc<Signer>,
    ethereum_rpc_url: Url,
}

#[async_trait]
impl Indexer for CrosschainIndexer {
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
                        let signer = self.signer.clone();
                        let ethereum_rpc_url = self.ethereum_rpc_url.clone();
                        tokio::spawn(async move {
                            let result: Result<_, anyhow::Error> = async move {
                                let response = reqwest::Client::new()
                                    .post(ethereum_rpc_url)
                                    .json(&serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "method": "eth_getBalance",
                                    "params": [
                                        msg,
                                        "latest"
                                    ],
                                    "id": 1
                                    }))
                                    .send()
                                    .await
                                    .map_err(|err| {
                                        anyhow::anyhow!(
                                            "Error sending request to Ethereum RPC: {err:?}"
                                        )
                                    })?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|err| {
                                        anyhow::anyhow!(
                                            "Error parsing response from Ethereum RPC: {err:?}"
                                        )
                                    })?
                                    .as_object()
                                    .unwrap()
                                    .get("result")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned();
                                Ok(Contract(ORACLE_CONTRACT.parse().unwrap())
                                    .call_function(
                                        "respond",
                                        OracleResponse {
                                            request_id: event.data.request_id,
                                            response: Response {
                                                response_data: response,
                                                refund_amount: None, // TODO price by input/output tokens?
                                            },
                                        },
                                    )
                                    .map_err(|err| {
                                        anyhow::anyhow!("Error calling function: {err:?}")
                                    })?
                                    .transaction()
                                    .gas(NearGas::from_tgas(300))
                                    .deposit(NearToken::from_yoctonear(0))
                                    .with_signer(account.0.clone(), signer)
                                    .with_retries(5)
                                    .send_to_mainnet()
                                    .await
                                    .map_err(|err| {
                                        anyhow::anyhow!("Error sending transaction: {err:?}")
                                    })?
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
    let mut indexer = CrosschainIndexer {
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
        ethereum_rpc_url: std::env::var("ETHEREUM_RPC_URL")
            .expect("No ETHEREUM_RPC_URL environment variable")
            .parse()
            .expect("ETHEREUM_RPC_URL environment variable is invalid"),
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

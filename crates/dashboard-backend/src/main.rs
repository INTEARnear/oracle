use anyhow::Result;
use futures::future::join_all;
use inevents_websocket_client::EventStreamClient;
use intear_events::events::log::log_nep297::LogNep297Event;
use intear_oracle::fees::ProducerFee;
use log::{error, info, warn};
use near_api::prelude::{AccountId, Contract, Reference};
use near_primitives::serialize::dec_format;
use near_primitives::types::Balance;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use warp::Filter;

const ORACLE_CONTRACT_ID: &str = "dev-unaudited-v1.oracle.intear.near";
const UPDATE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Fee {
    #[serde(with = "dec_format")]
    amount: Balance,
    token: AccountId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Oracle {
    id: AccountId,
    name: String,
    description: String,
    successes: u64,
    failures: u64,
    fee: Fee,
    example_input: Option<String>,
}

impl From<Producer> for Oracle {
    fn from(data: Producer) -> Self {
        Self {
            id: data.account_id,
            name: data.name,
            description: data.description,
            successes: data.requests_succeded,
            failures: data.requests_timed_out,
            fee: match data.fee {
                ProducerFee::None => Fee {
                    amount: 0,
                    token: "near".parse().unwrap(),
                },
                ProducerFee::Near { prepaid_amount } => Fee {
                    amount: prepaid_amount.as_yoctonear(),
                    token: "near".parse().unwrap(),
                },
                ProducerFee::FungibleToken {
                    token,
                    prepaid_amount,
                } => Fee {
                    amount: prepaid_amount.0,
                    token,
                },
            },
            example_input: data.example_input,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Producer {
    pub account_id: AccountId,
    pub requests_succeded: u64,
    pub requests_timed_out: u64,
    pub fee: ProducerFee,
    pub send_callback: bool,
    pub name: String,
    pub description: String,
    pub example_input: Option<String>,
}

type Oracles = Arc<RwLock<Vec<Oracle>>>;

async fn get_oracle_info(oracle_id: &AccountId) -> Oracle {
    match Contract(ORACLE_CONTRACT_ID.parse().unwrap())
        .call_function(
            "get_producer_details",
            serde_json::json!({
                "account_id": oracle_id,
            }),
        )
        .unwrap()
        .read_only::<Producer>()
        .at(Reference::Final)
        .fetch_from_mainnet()
        .await
    {
        Ok(data) => Oracle::from(data.data),
        Err(err) => {
            error!("{err:?}");
            Oracle {
                id: oracle_id.clone(),
                name: "Error".to_string(),
                description: "Error".to_string(),
                successes: 0,
                failures: 0,
                fee: Fee {
                    amount: 0,
                    token: "near".parse().unwrap(),
                },
                example_input: None,
            }
        }
    }
}

async fn update_all_oracles(oracles: Arc<RwLock<Vec<Oracle>>>, oracle_ids: Vec<AccountId>) {
    let mut interval = time::interval(UPDATE_INTERVAL);
    loop {
        interval.tick().await;

        info!(
            "Updating oracle information for {} oracles",
            oracle_ids.len()
        );
        let futures = oracle_ids.iter().map(get_oracle_info);
        let updated_oracles = join_all(futures).await;
        {
            let mut oracle_list = oracles.write();
            *oracle_list = updated_oracles;
            info!("Successfully updated oracle information");
        }
    }
}

async fn listen_to_oracle_updates(oracles: Arc<RwLock<Vec<Oracle>>>) {
    let client = EventStreamClient::default();
    client
        .stream_events::<LogNep297Event, _, _>("log_nep297", move |event| {
            let oracles = oracles.clone();
            async move {
                if event.account_id == ORACLE_CONTRACT_ID && event.event_standard == "intear-oracle"
                {
                    let producer: Producer = match &event.event_event[..] {
                        "producer_created" => {
                            serde_json::from_value(event.event_data.unwrap()).unwrap()
                        }
                        "producer_updated" => {
                            serde_json::from_value(event.event_data.unwrap()).unwrap()
                        }
                        _ => return,
                    };
                    info!("Producer created or updated: {:?}", producer);
                    let mut oracle_list = oracles.write();
                    if let Some(oracle) =
                        oracle_list.iter_mut().find(|o| o.id == producer.account_id)
                    {
                        *oracle = producer.into();
                    } else {
                        oracle_list.push(producer.into());
                    }
                }
            }
        })
        .await;
}

fn initial_oracle_ids() -> Vec<AccountId> {
    vec!["gpt4o.oracle.intear.near".parse().unwrap()]
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment variables
    if let Err(e) = dotenvy::dotenv() {
        warn!("Failed to load .env file: {}", e);
    }

    // Initialize logging
    env_logger::init();

    // Load configuration from environment
    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    let bind_addr = SocketAddr::from_str(&bind_address)?;

    // Optional SSL configuration
    let tls_cert_path = std::env::var("TLS_CERT_PATH").ok();
    let tls_key_path = std::env::var("TLS_KEY_PATH").ok();

    let oracle_ids = initial_oracle_ids();
    info!("Initialized with {} oracle IDs", oracle_ids.len());

    // Initialize with empty vec and update immediately
    let oracles = Arc::new(RwLock::new(Vec::new()));

    // Spawn the background tasks to update oracle stats
    let update_oracles = oracles.clone();
    let update_oracle_ids = oracle_ids.clone();
    tokio::spawn(async move { update_all_oracles(update_oracles, update_oracle_ids).await });
    let listen_oracles = oracles.clone();
    tokio::spawn(async move { listen_to_oracle_updates(listen_oracles).await });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET"])
        .allow_headers(vec!["Content-Type"]);

    let api = warp::path("oracles")
        .and(warp::get())
        .and(warp::any().map(move || oracles.clone()))
        .map(|oracles: Oracles| {
            let oracle_list = oracles.read();
            warp::reply::json(&*oracle_list)
        })
        .with(cors)
        .with(warp::log::log("api"));

    info!("Starting server on {}", bind_address);

    match (tls_cert_path, tls_key_path) {
        (Some(cert_path), Some(key_path)) => {
            info!("Using TLS");
            warp::serve(api)
                .tls()
                .cert_path(PathBuf::from(cert_path))
                .key_path(PathBuf::from(key_path))
                .run(bind_addr)
                .await;
        }
        _ => {
            warp::serve(api).run(bind_addr).await;
        }
    }

    Ok(())
}

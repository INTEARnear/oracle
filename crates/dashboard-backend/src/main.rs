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
use std::fs;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use warp::Filter;

const ORACLE_CONTRACT_ID: &str = "dev-unaudited-v1.oracle.intear.near";
const UPDATE_INTERVAL: Duration = Duration::from_secs(60);
const ORACLE_IDS_FILE: &str = "oracle_ids.txt";

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

async fn load_oracle_ids() -> io::Result<Vec<AccountId>> {
    match fs::read_to_string(ORACLE_IDS_FILE) {
        Ok(contents) => Ok(contents
            .lines()
            .filter_map(|line| line.parse().ok())
            .collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

async fn save_oracle_ids(ids: &[AccountId]) -> io::Result<()> {
    let mut file = fs::File::create(ORACLE_IDS_FILE)?;
    for id in ids {
        writeln!(file, "{}", id)?;
    }
    Ok(())
}

async fn update_all_oracles(oracles: Arc<RwLock<Vec<Oracle>>>) {
    let mut interval = time::interval(UPDATE_INTERVAL);
    loop {
        let oracle_ids = {
            let oracle_list = oracles.read();
            oracle_list.iter().map(|o| o.id.clone()).collect::<Vec<_>>()
        };

        if let Err(e) = save_oracle_ids(&oracle_ids).await {
            error!("Failed to save oracle IDs: {}", e);
        }

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

        interval.tick().await;
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

    // Load saved oracle IDs and initialize with them
    let saved_ids = load_oracle_ids().await.unwrap_or_else(|e| {
        error!("Failed to load oracle IDs: {}", e);
        Vec::new()
    });
    let initial_oracles = join_all(saved_ids.iter().map(get_oracle_info)).await;
    let oracles = Arc::new(RwLock::new(initial_oracles));

    // Spawn the background tasks to update oracle stats
    let update_oracles = oracles.clone();
    tokio::spawn(async move { update_all_oracles(update_oracles).await });
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

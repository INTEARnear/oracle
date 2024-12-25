use near_sdk::{
    env, ext_contract,
    json_types::{U128, U64},
    near, serde_json,
    store::LookupMap,
    AccountId, CryptoHash, Gas, GasWeight, NearToken,
};

use crate::{
    balance::FtId,
    producer::{ext_producer, ProducerId},
    StorageKey,
};
#[cfg(feature = "contract")]
use crate::{Oracle, OracleExt};

const RESUMPTION_TOKEN_REGISTER: u64 = 69;

pub type ConsumerId = AccountId;
pub type RequestId = U64;

type ResumptionToken = CryptoHash;

#[near(serializers=[borsh, json])]
pub struct PendingRequest {
    pub resumption_token: CryptoHash,
}

/// A consumer is an account that requests data from a producer.
#[near(serializers=[borsh])]
pub struct Consumer {
    /// Account ID of the consumer.
    pub account_id: ConsumerId,
    /// NEAR balance of the consumer. This will be used to pay for
    /// requests to all producers.
    pub near_balance: NearToken,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to specific producers. This will be
    /// used first before the general `near_balance`.
    pub near_balance_producer: LookupMap<ProducerId, NearToken>,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to all producers.
    pub ft_balances: LookupMap<FtId, U128>,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to specific producers. This will be
    /// used first before the general `ft_balances`.
    pub ft_balances_producer: LookupMap<(ProducerId, FtId), U128>,
    /// Number of requests sent to producers.
    pub requests_succeeded: u64,
    /// Number of requests that timed out (indexer didn't respond
    /// within 200 blocks. 200 is a NEAR protocol-level parameter.
    pub requests_timed_out: u64,
}

#[near(event_json(standard = "intear-oracle"))]
pub enum OracleEvent {
    #[event_version("1.0.0")]
    Request(RequestEventV1),
}

#[near(serializers=["json"])]
pub struct RequestEventV1 {
    pub producer_id: ProducerId,
    pub consumer_id: ConsumerId,
    pub request_id: RequestId,
    pub request_data: String,
}

#[cfg(feature = "contract")]
#[near]
impl Oracle {
    pub fn register_consumer(&mut self, account_id: ConsumerId) {
        let consumer = Consumer {
            account_id: account_id.clone(),
            near_balance: NearToken::from_near(0),
            near_balance_producer: LookupMap::new(StorageKey::ConsumerNearBalanceProducer {
                consumer: account_id.clone(),
            }),
            ft_balances: LookupMap::new(StorageKey::ConsumerFtBalances {
                consumer: account_id.clone(),
            }),
            ft_balances_producer: LookupMap::new(StorageKey::ConsumerFtBalancesProducers {
                consumer: account_id.clone(),
            }),
            requests_succeeded: 0,
            requests_timed_out: 0,
        };
        self.consumers.insert(account_id, consumer);
    }

    pub fn is_registered_as_consumer(&self, account_id: &ConsumerId) -> bool {
        self.consumers.contains_key(account_id)
    }
}

#[ext_contract(ext_oracle_consumer)]
pub trait ConsumerExt {
    fn request(&mut self, producer_id: ProducerId, request_data: String);
}

#[cfg(feature = "contract")]
#[near]
impl ConsumerExt for Oracle {
    #[payable]
    fn request(&mut self, producer_id: ProducerId, request_data: String) {
        let consumer_id = env::predecessor_account_id();
        let producer = self
            .producers
            .get(&producer_id)
            .expect("Producer doesn't exist");
        let fee = producer.fee.clone();
        let send_callback = producer.send_callback;
        if let Some(charged_fee) = self.try_charge_fee(&consumer_id, &producer_id, &fee) {
            let request_id = self.next_request_id;
            self.next_request_id = self
                .next_request_id
                .0
                .checked_add(1)
                .expect("Overflow")
                .into();

            if send_callback {
                ext_producer::ext(producer_id.clone()).on_request(
                    request_id,
                    request_data.clone(),
                    charged_fee.clone(),
                );
            }

            let promise_idx = env::promise_yield_create(
                "on_response",
                &serde_json::to_vec(&serde_json::json!({
                    "producer_id": producer_id,
                    "request_id": request_id,
                    "consumer_id": consumer_id,
                    "fee": charged_fee,
                }))
                .unwrap(),
                Gas::from_tgas(5),
                GasWeight::default(),
                RESUMPTION_TOKEN_REGISTER,
            );
            let resumption_token = if let Some(data) = env::read_register(RESUMPTION_TOKEN_REGISTER)
            {
                if let Ok(resumption_token) = ResumptionToken::try_from(data) {
                    resumption_token
                } else {
                    env::panic_str("Wrong register length")
                }
            } else {
                env::panic_str("Register is empty")
            };

            self.producers
                .get_mut(&producer_id)
                .expect("Producer doesn't exist")
                .requests_pending
                .insert(request_id, PendingRequest { resumption_token });
            OracleEvent::Request(RequestEventV1 {
                producer_id,
                consumer_id,
                request_id,
                request_data,
            })
            .emit();
            env::promise_return(promise_idx);
        } else {
            env::panic_str("Not enough balance");
        }
    }
}

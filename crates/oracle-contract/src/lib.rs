use near_sdk::{
    env,
    json_types::{U128, U64},
    log, near, serde_json,
    store::{IterableMap, LookupMap},
    AccountId, BorshStorageKey, CryptoHash, Gas, GasWeight, NearToken, PromiseOrValue,
};

const RESUMPTION_TOKEN_REGISTER: u64 = 69;

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum StorageKey {
    Producers,
    Consumers,
    PendingRequests {
        producer: ConsumerId,
    },
    ConsumerNearBalanceProducer {
        consumer: ConsumerId,
    },
    ConsumerFtBalances {
        consumer: ConsumerId,
    },
    ConsumerFtBalancesProducers {
        consumer: ConsumerId,
    },
    ConsumerFtBalancesProducer {
        consumer: ConsumerId,
        producer: ProducerId,
    },
}

// TODO: Storage management
#[near(contract_state)]
pub struct Contract {
    producers: LookupMap<ProducerId, Producer>,
    consumers: LookupMap<ConsumerId, Consumer>,
    next_request_id: RequestId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            producers: LookupMap::new(StorageKey::Producers),
            consumers: LookupMap::new(StorageKey::Consumers),
            next_request_id: 0.into(),
        }
    }
}

#[near]
impl Contract {
    pub fn add_producer(&mut self, account_id: ProducerId) {
        let producer = Producer {
            account_id: account_id.clone(),
            requests_succeded: 0,
            requests_timed_out: 0,
            #[cfg(not(feature = "near-sdk/__abi-generate"))]
            requests_pending: IterableMap::new(StorageKey::PendingRequests {
                producer: account_id.clone(),
            }),
            fee: None,
        };
        self.producers.insert(account_id, producer);
    }

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

    #[payable]
    pub fn deposit_near(&mut self, account_id: Option<AccountId>, producer_id: Option<ProducerId>) {
        let amount = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            if let Some(near_balance) = consumer.near_balance_producer.get_mut(&producer_id) {
                *near_balance = near_balance.checked_add(amount).expect("Overflow");
            } else {
                consumer.near_balance_producer.insert(producer_id, amount);
            }
            log!("Deposited {amount} NEAR to {account_id} for {producer_id}");
        } else {
            consumer.near_balance = consumer.near_balance.checked_add(amount).expect("Overflow");
            log!("Deposited {amount} NEAR to {account_id}");
        }
    }

    // TODO: Fee balance getters

    pub fn request(&mut self, producer_id: ProducerId, request_data: String) {
        let consumer_id = env::predecessor_account_id();
        let _consumer = self
            .consumers
            .get_mut(&consumer_id)
            .expect("Consumer is not registered");
        let producer = self
            .producers
            .get_mut(&producer_id)
            .expect("Producer doesn't exist");
        // TODO: Charge fee
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .0
            .checked_add(1)
            .expect("Overflow")
            .into();

        let promise_idx = env::promise_yield_create(
            "on_response",
            &serde_json::to_vec(&serde_json::json!({
                "producer_id": producer_id,
                "request_id": request_id,
            }))
            .unwrap(),
            Gas::from_tgas(5),
            GasWeight::default(),
            RESUMPTION_TOKEN_REGISTER,
        );
        let resumption_token = if let Some(data) = env::read_register(RESUMPTION_TOKEN_REGISTER) {
            if let Ok(resumption_token) = ResumptionToken::try_from(data) {
                resumption_token
            } else {
                env::panic_str("Wrong register length")
            }
        } else {
            env::panic_str("Register is empty")
        };

        producer
            .requests_pending
            .insert(request_id, PendingRequest { resumption_token });
        OracleEvent::Request(RequestEventV1 {
            consumer_id,
            request_id,
            request_data,
        })
        .emit();
        env::promise_return(promise_idx);
    }

    #[private]
    pub fn on_response(
        &mut self,
        producer_id: ProducerId,
        request_id: RequestId,
        #[callback_unwrap] response: Option<Response>,
    ) -> Option<Response> {
        let producer = self
            .producers
            .get_mut(&producer_id)
            .expect("Producer doesn't exist");
        log!(
            "Response from {producer_id} for {request_id}: {response:?}",
            request_id = request_id.0,
            response = response.as_ref().map(|r| &r.response_data)
        );
        // TODO: Fee refunds
        if let Some(_response) = response.as_ref() {
            producer.requests_succeded += 1;
        } else {
            producer.requests_timed_out += 1;
        }
        response
    }

    pub fn respond(&mut self, request_id: RequestId, response: Response) {
        let producer_id = env::predecessor_account_id();
        let producer = self
            .producers
            .get_mut(&producer_id)
            .expect("Producer is not registered");
        if let Some(pending_request) = producer.requests_pending.remove(&request_id) {
            if !env::promise_yield_resume(
                &pending_request.resumption_token,
                &serde_json::to_vec(&response).expect("Can't serialize on_response args"),
            ) {
                env::panic_str("Resumption token not found")
            }
        }
    }
}

#[near(event_json(standard = "intear-oracle"))]
pub enum OracleEvent {
    #[event_version("1.0.0")]
    Request(RequestEventV1),
}

#[near(serializers=["json"])]
pub struct RequestEventV1 {
    consumer_id: ConsumerId,
    request_id: RequestId,
    request_data: String,
}

#[near(serializers=[borsh, json])]
pub struct PendingRequest {
    resumption_token: CryptoHash,
}

#[near(serializers=[json])]
pub struct Response {
    response_data: String,
}

#[near(serializers=[json])]
struct FtDepositArgs {
    account_id: Option<AccountId>,
    producer_id: Option<ProducerId>,
}

// TODO: Can't use git near-sdk with near-sdk-contract-tools
//
// impl Nep141Receiver for Contract {
//     fn ft_on_transfer(
//         &mut self,
//         sender_id: AccountId,
//         amount: U128,
//         msg: String,
//     ) -> PromiseOrValue<U128> {
//
//     }
// }
impl Contract {
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let args = serde_json::from_str::<FtDepositArgs>(&msg).expect("Invalid msg");
        let account_id = args.account_id.unwrap_or(sender_id);
        let ft_id = env::predecessor_account_id();
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = args.producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            if let Some(ft_balance) = consumer
                .ft_balances_producer
                .get_mut(&(producer_id.clone(), ft_id.clone()))
            {
                *ft_balance = ft_balance.0.checked_add(amount.0).expect("Overflow").into();
            } else {
                consumer
                    .ft_balances_producer
                    .insert((producer_id, ft_id), amount);
            }
            log!("Deposited {amount} {account_id} to {sender_id} for {producer_id}");
        } else {
            if let Some(ft_balance) = consumer.ft_balances.get_mut(&ft_id) {
                *ft_balance = ft_balance.0.checked_add(amount.0).expect("Overflow").into();
            } else {
                consumer.ft_balances.insert(ft_id, amount);
            }
            log!("Deposited {amount} {account_id} to {sender_id}");
        }
        PromiseOrValue::Value(U128(0))
    }
}

type RequestId = U64;
type ResumptionToken = CryptoHash;
type ProducerId = AccountId;
type ConsumerId = AccountId;
type FtId = AccountId;

/// A producer is an account that provides data to consumers.
#[near(serializers=[borsh])]
struct Producer {
    /// Account ID of the producer.
    account_id: ProducerId,
    /// Number of responses sent back to the consumer.
    requests_succeded: u64,
    /// Number of requests that timed out (indexer didn't respond
    /// within 200 blocks. 200 is a NEAR protocol-level parameter.
    requests_timed_out: u64,
    /// Requests that are currently being processed.
    requests_pending: IterableMap<RequestId, PendingRequest>,
    /// Producers meant for public use may want to charge a fee.
    fee: Option<ProducerFee>,
}

/// Fees are set by producers. If the consumer's balance is less
/// than min_fee, the request will be rejected without a log generated.
/// The producer may choose to refund a part of the fee if the request
/// was successful. If the indexer times out, the fee is refunded fully.
#[near(serializers=[borsh, json])]
pub struct ProducerFee {
    /// Fee fungible token. If None, it's NEAR.
    ft_token: Option<FtId>,
    /// Fee amount in the fungible token.
    ft_min_fee: Option<NearToken>,
    /// Fee amount in NEAR. If None, it's 0.
    near_min_fee: Option<NearToken>,
}

/// A consumer is an account that requests data from a producer.
#[near(serializers=[borsh])]
pub struct Consumer {
    /// Account ID of the consumer.
    account_id: ConsumerId,
    /// NEAR balance of the consumer. This will be used to pay for
    /// requests to all producers.
    near_balance: NearToken,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to specific producers. This will be
    /// used first before the general `near_balance`.
    near_balance_producer: LookupMap<ProducerId, NearToken>,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to all producers.
    ft_balances: LookupMap<FtId, U128>,
    /// Fungible token balances of the consumer that will be used
    /// to pay for requests to specific producers. This will be
    /// used first before the general `ft_balances`.
    ft_balances_producer: LookupMap<(ProducerId, FtId), U128>,
    /// Number of requests sent to producers.
    requests_succeeded: u64,
    /// Number of requests that timed out (indexer didn't respond
    /// within 200 blocks. 200 is a NEAR protocol-level parameter.
    requests_timed_out: u64,
}

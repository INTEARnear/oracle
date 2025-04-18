use near_sdk::serde::Serialize;
use near_sdk::store::LookupMap;
use near_sdk::NearSchema;
use near_sdk::{
    env, ext_contract, json_types::U128, log, near, serde_json, AccountId, PromiseError,
};

use crate::consumer::OracleEvent;
use crate::{
    consumer::{ConsumerId, PendingRequest, RequestId},
    fees::{PrepaidFee, ProducerFee},
    StorageKey,
};
#[cfg(feature = "contract")]
use crate::{Oracle, OracleExt};

pub type ProducerId = AccountId;

#[near(serializers=[json])]
pub struct Response {
    pub response_data: String,
    pub refund_amount: Option<U128>,
}

/// A producer is an account that provides data to consumers.
#[near(serializers=[borsh])]
#[derive(Serialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Producer {
    /// Account ID of the producer.
    pub account_id: ProducerId,
    /// Number of responses sent back to the consumer.
    pub requests_succeded: u64,
    /// Number of requests that timed out (indexer didn't respond
    /// within 200 blocks. 200 is a NEAR protocol-level parameter.
    pub requests_timed_out: u64,
    /// Requests that are currently being processed.
    #[serde(skip)]
    #[schemars(skip)]
    pub requests_pending: LookupMap<RequestId, PendingRequest>,
    /// Producers meant for public use may want to charge a fee.
    pub fee: ProducerFee,
    /// If true, the contract will receive `on_request(request_id,
    /// request_data, prepaid_fee)` call
    pub send_callback: bool,
    /// Name of the producer.
    pub name: String,
    /// Description of the producer.
    pub description: String,
    /// Example input that can be used in usage examples on the
    /// oracle dashboard.
    pub example_input: Option<String>,
}

#[ext_contract(ext_producer)]
pub trait ProducerContract {
    fn on_request(&mut self, request_id: RequestId, request_data: String, prepaid_fee: PrepaidFee);
}

#[cfg(feature = "contract")]
#[near]
impl Oracle {
    pub fn add_producer(&mut self) {
        let account_id = env::predecessor_account_id();
        let producer = Producer {
            account_id: account_id.clone(),
            requests_succeded: 0,
            requests_timed_out: 0,
            requests_pending: LookupMap::new(StorageKey::PendingRequests {
                producer: account_id.clone(),
            }),
            fee: ProducerFee::None,
            send_callback: false,
            name: "Unnamed".to_string(),
            description: "No description".to_string(),
            example_input: None,
        };
        OracleEvent::ProducerCreated(Producer {
            account_id: producer.account_id.clone(),
            requests_succeded: producer.requests_succeded,
            requests_timed_out: producer.requests_timed_out,
            requests_pending: LookupMap::new(b"dontcare".as_slice()),
            fee: producer.fee.clone(),
            send_callback: producer.send_callback,
            name: producer.name.clone(),
            description: producer.description.clone(),
            example_input: producer.example_input.clone(),
        })
        .emit();
        self.producers.insert(account_id, producer);
    }

    pub fn get_producer_details(&self, account_id: ProducerId) -> &Producer {
        self.producers
            .get(&account_id)
            .expect("Producer doesn't exist")
    }

    pub fn edit_producer_details(
        &mut self,
        name: String,
        description: String,
        example_input: Option<String>,
    ) {
        let producer = self
            .producers
            .get_mut(&env::predecessor_account_id())
            .expect("Producer doesn't exist");
        producer.name = name;
        producer.description = description;
        producer.example_input = example_input;

        OracleEvent::ProducerUpdated(Producer {
            account_id: producer.account_id.clone(),
            requests_succeded: producer.requests_succeded,
            requests_timed_out: producer.requests_timed_out,
            requests_pending: LookupMap::new(b"dontcare".as_slice()),
            fee: producer.fee.clone(),
            send_callback: producer.send_callback,
            name: producer.name.clone(),
            description: producer.description.clone(),
            example_input: producer.example_input.clone(),
        })
        .emit();
    }

    pub fn is_producer(&self, account_id: ProducerId) -> bool {
        self.producers.contains_key(&account_id)
    }

    pub fn set_send_callback(&mut self, send_callback: bool) {
        let producer = self
            .producers
            .get_mut(&env::predecessor_account_id())
            .expect("Producer doesn't exist");
        producer.send_callback = send_callback;

        OracleEvent::ProducerUpdated(Producer {
            account_id: producer.account_id.clone(),
            requests_succeded: producer.requests_succeded,
            requests_timed_out: producer.requests_timed_out,
            requests_pending: LookupMap::new(b"dontcare".as_slice()),
            fee: producer.fee.clone(),
            send_callback: producer.send_callback,
            name: producer.name.clone(),
            description: producer.description.clone(),
            example_input: producer.example_input.clone(),
        })
        .emit();
    }

    #[private]
    pub fn on_response(
        &mut self,
        producer_id: ProducerId,
        request_id: RequestId,
        consumer_id: ConsumerId,
        fee: PrepaidFee,
        #[callback_result] response: Result<Response, PromiseError>,
    ) -> Option<Response> {
        let producer = self
            .producers
            .get_mut(&producer_id)
            .expect("Producer doesn't exist");
        log!(
            "Response from {producer_id} for {request_id}: {response:?}, refund {refund:?}",
            request_id = request_id.0,
            response = response.as_ref().map(|r| &r.response_data),
            refund = response.as_ref().map(|r| &r.refund_amount),
        );
        if let Ok(response) = response.as_ref() {
            producer.requests_succeded += 1;
            if let Some(refund_amount) = response.refund_amount {
                self.refund_partially(&consumer_id, &producer_id, &fee, refund_amount);
            }
            self.deposit_to_producer(producer_id.clone(), &fee, response.refund_amount);
        } else {
            producer.requests_timed_out += 1;
            self.refund_fully(&consumer_id, &producer_id, &fee);
        }

        let producer = self.producers.get(&producer_id).unwrap();
        OracleEvent::ProducerUpdated(Producer {
            account_id: producer.account_id.clone(),
            requests_succeded: producer.requests_succeded,
            requests_timed_out: producer.requests_timed_out,
            requests_pending: LookupMap::new(b"dontcare".as_slice()),
            fee: producer.fee.clone(),
            send_callback: producer.send_callback,
            name: producer.name.clone(),
            description: producer.description.clone(),
            example_input: producer.example_input.clone(),
        })
        .emit();

        response.ok()
    }
}

#[ext_contract(ext_oracle_responder)]
pub trait OracleResponder {
    fn respond(&mut self, request_id: RequestId, response: Response);
}

#[cfg(feature = "contract")]
#[near]
impl OracleResponder for Oracle {
    fn respond(&mut self, request_id: RequestId, response: Response) {
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
        } else {
            env::panic_str("Request not found or already responded to")
        }
    }
}

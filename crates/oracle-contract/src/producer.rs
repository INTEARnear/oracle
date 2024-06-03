use near_sdk::{env, json_types::U128, log, near, serde_json, store::IterableMap, AccountId};

use crate::{
    consumer::{ConsumerId, PendingRequest, RequestId},
    fees::{PrepaidFee, ProducerFee},
    Contract, ContractExt, StorageKey,
};

pub type ProducerId = AccountId;

#[near(serializers=[json])]
pub struct Response {
    pub response_data: String,
    pub refund_amount: Option<U128>,
}

/// A producer is an account that provides data to consumers.
#[near(serializers=[borsh])]
pub struct Producer {
    /// Account ID of the producer.
    pub account_id: ProducerId,
    /// Number of responses sent back to the consumer.
    pub requests_succeded: u64,
    /// Number of requests that timed out (indexer didn't respond
    /// within 200 blocks. 200 is a NEAR protocol-level parameter.
    pub requests_timed_out: u64,
    /// Requests that are currently being processed.
    pub requests_pending: IterableMap<RequestId, PendingRequest>,
    /// Producers meant for public use may want to charge a fee.
    pub fee: ProducerFee,
}

#[near]
impl Contract {
    pub fn add_producer(&mut self, account_id: ProducerId) {
        let producer = Producer {
            account_id: account_id.clone(),
            requests_succeded: 0,
            requests_timed_out: 0,
            requests_pending: IterableMap::new(StorageKey::PendingRequests {
                producer: account_id.clone(),
            }),
            fee: ProducerFee::None,
        };
        self.producers.insert(account_id, producer);
    }

    pub fn is_producer(&self, account_id: ProducerId) -> bool {
        self.producers.contains_key(&account_id)
    }

    #[private]
    pub fn on_response(
        &mut self,
        producer_id: ProducerId,
        request_id: RequestId,
        consumer_id: ConsumerId,
        fee: PrepaidFee,
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
        if let Some(response) = response.as_ref() {
            producer.requests_succeded += 1;
            if let Some(refund_amount) = response.refund_amount {
                self.refund_partially(&consumer_id, &producer_id, &fee, refund_amount);
            }
            self.deposit_to_producer(producer_id, &fee, response.refund_amount);
        } else {
            producer.requests_timed_out += 1;
            self.refund_fully(&consumer_id, &producer_id, &fee);
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
        } else {
            env::panic_str("Request not found or already responded to")
        }
    }
}

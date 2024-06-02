use near_sdk::{env, log, near, serde_json, store::IterableMap, AccountId, NearToken};

use crate::{
    balance::FtId,
    consumer::{PendingRequest, RequestId},
    Contract, ContractExt, StorageKey,
};

pub type ProducerId = AccountId;

#[near(serializers=[json])]
pub struct Response {
    pub response_data: String,
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
    pub fee: Option<ProducerFee>,
}

/// Fees are set by producers. If the consumer's balance is less
/// than min_fee, the request will be rejected without a log generated.
/// The producer may choose to refund a part of the fee if the request
/// was successful. If the indexer times out, the fee is refunded fully.
#[near(serializers=[borsh, json])]
pub struct ProducerFee {
    /// Fee fungible token. If None, it's NEAR.
    pub ft_token: Option<FtId>,
    /// Fee amount in the fungible token.
    pub ft_min_fee: Option<NearToken>,
    /// Fee amount in NEAR. If None, it's 0.
    pub near_min_fee: Option<NearToken>,
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
            fee: None,
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
        } else {
            env::panic_str("Request not found or already responded to")
        }
    }
}

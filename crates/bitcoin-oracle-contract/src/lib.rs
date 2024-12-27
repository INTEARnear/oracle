mod bitcoin_light_client;
mod types;

use bitcoin_light_client::{ext_bitcoin_light_client, ProofArgs};
use intear_oracle::consumer::RequestId;
use intear_oracle::fees::PrepaidFee;
use intear_oracle::producer::{ext_oracle_responder, ProducerContract, Response};
use near_sdk::Gas;
use near_sdk::{
    env, near, store::LookupMap, AccountId, NearToken, PanicOnDefault, Promise, PromiseError,
};
use types::{TxWithProof, H256};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    bitcoin_light_client: AccountId,
    oracle_contract: AccountId,
    requests: LookupMap<RequestId, (H256, PrepaidFee)>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(bitcoin_light_client: AccountId, oracle_contract: AccountId) -> Self {
        Self {
            bitcoin_light_client,
            oracle_contract,
            requests: LookupMap::new(b"r".to_vec()),
        }
    }

    pub fn submit(&mut self, request_id: RequestId, transaction_details: TxWithProof) {
        let tx_hash = self.requests.get(&request_id).expect("Request not found").0;

        assert_eq!(tx_hash, transaction_details.tx_id);

        ext_bitcoin_light_client::ext(self.bitcoin_light_client.clone())
            .verify_transaction_inclusion(ProofArgs {
                tx_id: transaction_details.tx_id,
                tx_block_blockhash: transaction_details.tx_block_blockhash,
                tx_index: transaction_details.tx_index,
                merkle_proof: transaction_details.merkle_proof.clone(),
                confirmations: transaction_details.confirmations,
            })
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(100))
                    .on_verified(
                        request_id,
                        transaction_details,
                        env::predecessor_account_id(),
                    ),
            );
    }

    #[private]
    pub fn on_verified(
        &mut self,
        request_id: RequestId,
        transaction_details: TxWithProof,
        sender_account_id: AccountId,
        #[callback_result] result: Result<bool, PromiseError>,
    ) -> Promise {
        let (_tx_hash, prepaid_fee) = self
            .requests
            .remove(&request_id)
            .expect("Request not found");

        match result {
            Ok(true) => (),
            Ok(false) => env::panic_str("Transaction not included"),
            Err(e) => env::panic_str(&format!("Failed to verify transaction details: {e:?}")),
        }

        let prepaid_near = match prepaid_fee {
            PrepaidFee::Near { amount, .. } => amount.as_yoctonear(),
            _ => env::panic_str("Invalid prepaid fee token"),
        };

        let response = Response {
            response_data: near_sdk::serde_json::to_string(&near_sdk::serde_json::json!({
                "confirmations": transaction_details.confirmations,
            }))
            .unwrap(),
            refund_amount: None,
        };
        Promise::new(sender_account_id).transfer(NearToken::from_yoctonear(prepaid_near));
        ext_oracle_responder::ext(self.oracle_contract.clone()).respond(request_id, response)
    }
}

#[near]
impl ProducerContract for Contract {
    fn on_request(&mut self, request_id: RequestId, request_data: String, prepaid_fee: PrepaidFee) {
        if env::predecessor_account_id() != self.oracle_contract {
            env::panic_str("Only oracle contract can call this method");
        }
        let mut vec = hex::decode(request_data).unwrap();
        vec.reverse();
        let tx_hash = H256(vec.try_into().unwrap());
        self.requests.insert(request_id, (tx_hash, prepaid_fee));
    }
}

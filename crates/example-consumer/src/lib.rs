use intear_oracle::{
    consumer::ext_oracle_consumer,
    producer::{ProducerId, Response},
};
use near_sdk::{env, near, AccountId, Gas, NearToken, PanicOnDefault, Promise};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    oracle_contract: AccountId,
    producer_id: ProducerId,
}

#[near]
impl Contract {
    #[init]
    pub fn new(oracle_contract: AccountId, producer_id: AccountId) -> Self {
        Self {
            oracle_contract,
            producer_id,
        }
    }

    pub fn test_statement(&self, statement: String) -> Promise {
        let prompt = format!("Your job is to determine if the following statement is true:\n\n```\n{statement}\n```\n\nRespond with only \"Yes\" or \"No\"");
        ext_oracle_consumer::ext(self.oracle_contract.clone())
            .with_static_gas(Gas::from_tgas(10))
            .with_attached_deposit(NearToken::from_millinear(10)) // attach 0.01N fee
            .request(self.producer_id.clone(), prompt)
            .then(Self::ext(env::current_account_id()).on_response())
    }

    #[private]
    pub fn on_response(&self, #[callback_unwrap] result: Option<Response>) -> bool {
        result
            .expect("Oracle didn't submit a response in time")
            .response_data
            .to_lowercase()
            .contains("yes")
    }
}

use near_sdk::{
    ext_contract,
    json_types::{U128, U64},
    near, AccountId, NearToken,
};

#[ext_contract(ext_oracle)]
pub trait Oracle {
    fn respond(&mut self, request_id: RequestId, response: Response);
}

pub type RequestId = U64;

#[near(serializers=[json])]
pub struct Response {
    pub response_data: String,
    pub refund_amount: Option<U128>,
}

pub trait ProducerContract {
    fn on_request(&mut self, request_id: RequestId, request_data: String, prepaid_fee: PrepaidFee);
}

#[near(serializers=[borsh, json])]
pub enum PrepaidFee {
    None,
    Near {
        amount: NearToken,
        is_for_specific_producer: bool,
    },
    FungibleToken {
        token: AccountId,
        amount: U128,
        is_for_specific_producer: bool,
    },
}

mod oracle;
mod reclaim;

use std::collections::HashMap;

use near_sdk::Gas;
use near_sdk::{
    env, json_types::U128, near, require, store::LookupMap, AccountId, NearToken, PanicOnDefault,
    Promise, PromiseError,
};
use oracle::{ext_oracle, PrepaidFee, ProducerContract, RequestId};
use reclaim::{ext_reclaim, Proof};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    reclaim_contract: AccountId,
    oracle_contract: AccountId,
    requests: LookupMap<RequestId, (GptRequest, PrepaidFee)>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(reclaim_contract: AccountId, oracle_contract: AccountId) -> Self {
        Self {
            reclaim_contract,
            oracle_contract,
            requests: LookupMap::new(b"r".to_vec()),
        }
    }

    pub fn submit(&mut self, request_id: RequestId, proof: Proof) {
        ext_reclaim::ext(self.reclaim_contract.clone())
            .verify_proof(proof.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(100))
                    .on_verified(request_id, proof),
            );
    }

    #[private]
    pub fn on_verified(
        &mut self,
        request_id: RequestId,
        proof: Proof,
        #[callback_result] result: Result<(), PromiseError>,
    ) -> Promise {
        if let Err(e) = result {
            env::panic_str(&format!("Failed to verify proof: {e:?}"));
        }
        let parameters: RequestParameters =
            near_sdk::serde_json::from_str(&proof.claimInfo.parameters).unwrap();

        require!(parameters.method == "POST", "Invalid request");

        let matches = parameters.response_matches.first().unwrap();
        require!(matches.r#type == "contains", "Invalid proof");

        let (request, prepaid_fee) = self.requests.get(&request_id).expect("Request not found");

        let http_request: OpenAiRequest = near_sdk::serde_json::from_str(
            &parameters.body.expect("Request does not contain body"),
        )
        .unwrap();
        require!(http_request.model == request.model, "Invalid model");
        require!(http_request.seed == request.seed, "Invalid seed");
        require!(http_request.messages.len() == 2, "Invalid messages");
        require!(
            http_request.messages[0].role == "system",
            "The first message should be a system message"
        );
        require!(
            http_request.messages[0].content == request.system_message,
            "Invalid system message"
        );
        require!(
            http_request.messages[1].role == "user",
            "The second message should be a user message"
        );
        require!(
            http_request.messages[1].content == request.user_message,
            "Invalid user message"
        );

        let http_response: OpenAiResponse =
            near_sdk::serde_json::from_str(&format!("{{\n{}", matches.value)).unwrap();
        require!(http_response.model == request.model, "Invalid model");
        require!(
            http_response.choices.len() == 1,
            "Responses should have exactly n=1"
        );
        require!(
            http_response.choices[0].message.role == "assistant",
            "Invalid role"
        );

        let result = http_response.choices[0].message.content.clone();
        let price_near = http_response.usage.prompt_tokens as u128
            * NearToken::from_yoctonear(10_u128.pow(19)).as_yoctonear()
            + http_response.usage.completion_tokens as u128
                * NearToken::from_yoctonear(10_u128.pow(19) * 2).as_yoctonear()
            + NearToken::from_millinear(10).as_yoctonear();
        let prepaid_near = match prepaid_fee {
            PrepaidFee::Near { amount, .. } => amount.as_yoctonear(),
            _ => env::panic_str("Invalid prepaid fee token"),
        };
        let refund_amount = match prepaid_near.checked_sub(price_near) {
            Some(amount) => amount,
            None => env::panic_str("Insufficient fee"),
        };

        let response = oracle::Response {
            response_data: near_sdk::serde_json::to_string(&result).unwrap(),
            refund_amount: Some(U128(refund_amount)),
        };
        ext_oracle::ext(self.oracle_contract.clone()).respond(request_id, response)
    }
}

#[near]
impl ProducerContract for Contract {
    fn on_request(&mut self, request_id: RequestId, request_data: String, prepaid_fee: PrepaidFee) {
        if env::predecessor_account_id() != self.oracle_contract {
            env::panic_str("Only oracle contract can call this method");
        }
        let request: GptRequest = near_sdk::serde_json::from_str(&request_data).unwrap();
        self.requests.insert(request_id, (request, prepaid_fee));
    }
}

#[near(serializers=[json])]
#[serde(rename_all = "camelCase")]
struct RequestParameters {
    method: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    response_matches: Vec<ResponseMatch>,
}

#[near(serializers=[json])]
struct ResponseMatch {
    r#type: String,
    value: String,
}

#[near(serializers=[json])]
struct OpenAiResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[near(serializers=[json])]
struct Choice {
    index: u64,
    message: Message,
}

#[near(serializers=[json])]
struct Message {
    role: String,
    content: String,
}

#[near(serializers=[json])]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    prompt_tokens_details: PromptTokenDetails,
    completion_tokens_details: CompletionTokenDetails,
}

#[near(serializers=[json])]
struct PromptTokenDetails {
    cached_tokens: u64,
}

#[near(serializers=[json])]
struct CompletionTokenDetails {
    reasoning_tokens: u64,
}

#[near(serializers=[json])]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    seed: u64,
}

#[near(serializers=[borsh, json])]
struct GptRequest {
    model: String,
    seed: u64,
    system_message: String,
    user_message: String,
}

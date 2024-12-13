use near_sdk::{ext_contract, near};

#[allow(dead_code)]
#[ext_contract(ext_reclaim)]
pub trait Reclaim {
    #[handle_result]
    fn verify_proof(&mut self, proof: Proof) -> Result<(), &'static str>;
}

#[near(serializers=[json])]
#[derive(Clone)]
pub struct ClaimInfo {
    pub provider: String,
    pub parameters: String,
    pub context: String,
}

#[near(serializers=[json])]
#[derive(Clone)]
#[allow(non_snake_case)] // Could use #[serde(rename_all = "camelCase")], but this is copied from https://gitlab.reclaimprotocol.org/integrations/onchain/near-sdk/-/blob/main/src/claims.rs
pub struct CompleteClaimData {
    pub identifier: String,
    pub owner: String,
    pub epoch: u64,
    pub timestampS: u64,
}

#[near(serializers=[json])]
#[derive(Clone)]
pub struct SignedClaim {
    pub claim: CompleteClaimData,
    pub signatures: Vec<String>,
}

#[near(serializers=[json])]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Proof {
    pub claimInfo: ClaimInfo,
    pub signedClaim: SignedClaim,
}

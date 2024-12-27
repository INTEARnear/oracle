use near_sdk::{ext_contract, near};

use crate::types::H256;

#[ext_contract(ext_bitcoin_light_client)]
#[allow(dead_code)]
trait BitcoinLightClient {
    fn verify_transaction_inclusion(&self, #[serializer(borsh)] args: ProofArgs) -> bool;
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct ProofArgs {
    pub tx_id: H256,
    pub tx_block_blockhash: H256,
    pub tx_index: u64,
    pub merkle_proof: Vec<H256>,
    pub confirmations: u64,
}

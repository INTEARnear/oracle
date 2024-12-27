use near_sdk::near;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Copy)]
#[near(serializers=[borsh, json])]
pub struct H256(pub [u8; 32]);

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TxWithProof {
    pub tx_id: H256,
    pub tx_block_blockhash: H256,
    pub tx_index: u64,
    pub merkle_proof: Vec<H256>,
    pub confirmations: u64,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub prev_tx: H256,
    pub prev_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

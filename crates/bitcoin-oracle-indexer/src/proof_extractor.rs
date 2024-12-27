use std::str::FromStr;

use bitcoin::{
    hashes::{hex::FromHex, sha256d, Hash},
    Block, Txid,
};
use bitcoincore_rpc::json::GetRawTransactionResult;
use serde::{Deserialize, Serialize};

use crate::types::H256;

#[derive(Debug, Clone, Serialize)]
pub struct TxWithProof {
    pub tx_id: H256,
    pub tx_block_blockhash: H256,
    pub tx_index: u64,
    pub merkle_proof: Vec<H256>,
    pub confirmations: u64,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionInput {
    pub prev_tx: H256,
    pub prev_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: T,
}

pub async fn get_transaction_proof(
    tx_id_str: &str,
    bitcoin_rpc_url: &str,
) -> Result<TxWithProof, Box<dyn std::error::Error>> {
    // Parse transaction ID
    let tx_id = Txid::from_str(tx_id_str)?;

    let client = reqwest::Client::new();

    // Get transaction info
    let json_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getrawtransaction",
        "params": [
            tx_id_str,
            true
        ]
    });
    let response = client
        .post(bitcoin_rpc_url)
        .header("Content-Type", "application/json") // Replace with your actual credentials
        .json(&json_body)
        .send()
        .await?;
    let tx: RpcResponse<GetRawTransactionResult> = response.json().await?;
    let tx = tx.result;

    // Ensure transaction is confirmed
    if tx.confirmations.unwrap_or(0) < 1 {
        return Err("Transaction not confirmed".into());
    }

    let block_hash = tx.blockhash.ok_or("Block hash not found")?;

    // Get block information
    let json_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getblock",
        "params": [
            block_hash,
            0
        ]
    });
    let response = client.post(bitcoin_rpc_url).json(&json_body).send().await?;
    let hex: RpcResponse<String> = response.json().await?;
    let hex = hex.result;
    let bytes: Vec<u8> = FromHex::from_hex(&hex)?;
    let block: Block = bitcoin::consensus::encode::deserialize(&bytes)?;

    // Find transaction index in block
    let tx_index = block
        .txdata
        .iter()
        .position(|tx| tx.txid() == tx_id)
        .ok_or("Transaction not found in block")?;

    // Get merkle proof
    let merkle_proof = compute_merkle_proof(
        &block
            .txdata
            .iter()
            .map(|tx| tx.txid().to_string())
            .collect::<Vec<_>>(),
        tx_index,
    )?;

    // Convert inputs
    let inputs = tx
        .vin
        .into_iter()
        .map(|vin| {
            Ok(TransactionInput {
                prev_tx: H256(vin.txid.ok_or("Missing txid")?.to_vec().try_into().unwrap()),
                prev_index: vin.vout.ok_or("Missing vout")?,
                script_sig: hex::decode(&vin.script_sig.ok_or("Missing script_sig")?.hex)?,
                sequence: vin.sequence,
                witness: vin
                    .txinwitness
                    .unwrap_or_default()
                    .into_iter()
                    .map(|w| hex::decode(&w).unwrap_or_default())
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    // Convert outputs
    let outputs = tx
        .vout
        .into_iter()
        .map(|vout| {
            Ok(TransactionOutput {
                value: vout.value.to_sat(),
                script_pubkey: vout.script_pub_key.hex,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok(TxWithProof {
        tx_id: H256(tx_id.to_vec().try_into().unwrap()),
        tx_block_blockhash: H256(block_hash.to_vec().try_into().unwrap()),
        tx_index: tx_index as u64,
        merkle_proof,
        confirmations: (tx.confirmations.unwrap_or(1) - 1) as u64,
        inputs,
        outputs,
    })
}

fn compute_merkle_proof(
    txs: &[String],
    tx_index: usize,
) -> Result<Vec<H256>, Box<dyn std::error::Error>> {
    let mut proof = Vec::new();
    let mut target_index = tx_index;

    // First convert all transaction IDs to sha256d::Hash
    let mut current_level = Vec::with_capacity(txs.len());
    for tx in txs {
        let txid = Txid::from_str(tx).map_err(|e| format!("Invalid transaction ID: {}", e))?;
        let hash =
            sha256d::Hash::from_slice(txid.as_ref()).map_err(|e| format!("Invalid hash: {}", e))?;
        current_level.push(hash);
    }

    while current_level.len() > 1 {
        // If we have an odd number of elements, duplicate the last one
        if current_level.len() % 2 != 0 {
            current_level.push(current_level[current_level.len() - 1]);
        }

        let mut next_level = Vec::with_capacity(current_level.len() / 2);

        // Find the sibling for the current target index
        let sibling_index = if target_index % 2 == 0 {
            target_index + 1
        } else {
            target_index - 1
        };

        // Add the sibling to the proof
        if sibling_index < current_level.len() {
            proof.push(H256(
                current_level[sibling_index].to_vec().try_into().unwrap(),
            ));
        }

        // Build the next level
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left
            };
            let combined = sha256d::Hash::hash(&[&left[..], &right[..]].concat());
            next_level.push(combined);
        }

        // Update target index for next level
        target_index /= 2;
        current_level = next_level;
    }

    Ok(proof)
}

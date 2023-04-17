pub mod message;
pub mod types;

use axon_protocol::trie;
use axon_protocol::types::{Hash, MerkleRoot, Receipt};
use hasher::HasherKeccak;
use types::{IcsComponent, VerifyError};

pub fn verfify_event<I: IcsComponent>(
    tx_hash: Hash,
    tx_root: MerkleRoot,
    tx_proof: Vec<Vec<u8>>,
    receipt_root: MerkleRoot,
    receipt: Receipt,
    msg: I,
    receipt_proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    if tx_hash != receipt.tx_hash {
        return Err(VerifyError::TxReceiptNotMatch);
    }
    verify_tx(tx_hash, tx_root, tx_proof)?;
    verify_receipt(msg, receipt, receipt_root, receipt_proof)
}

fn verify_tx(tx_hash: Hash, root: MerkleRoot, proof: Vec<Vec<u8>>) -> Result<(), VerifyError> {
    trie::verify_proof(
        root.as_bytes(),
        tx_hash.as_bytes(),
        proof,
        HasherKeccak::new(),
    )
    .map_err(|_| VerifyError::InvalidTxProof)?;
    Ok(())
}

// Get the frist log as IcsComponent.
fn verify_receipt<I: IcsComponent>(
    expect: I,
    receipt: Receipt,
    root: MerkleRoot,
    proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    let actual = receipt
        .logs
        .into_iter()
        .next()
        .ok_or(VerifyError::FoundNoMessage)?;

    if expect.encode() != actual.data {
        return Err(VerifyError::EventNotMatch);
    }

    let key = expect.as_key();

    trie::verify_proof(root.as_bytes(), key, proof, HasherKeccak::new())
        .map_err(|_| VerifyError::InvalidReceiptProof)?;

    Ok(())
}

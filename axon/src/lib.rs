#![no_std]

extern crate alloc;

pub use alloc::vec::Vec;

pub mod cell;
pub mod message;
pub mod object;
pub mod proof;
pub mod verify_mpt;

// use axon_protocol::trie;
// use axon_protocol::types::{Hash, MerkleRoot, Receipt};
// use hasher::HasherKeccak;
use ethereum_types::H256;
use object::{Object, VerifyError};
use proof::TransactionReceipt;

pub type Bytes = Vec<u8>;
pub type MerkleRoot = Vec<u8>;
pub type U256 = Vec<u8>;

pub fn verify_message<I: Object>(
    tx_hash: H256,
    tx_root: H256,
    tx_proof: Vec<H256>,
    receipt_root: MerkleRoot,
    receipt: TransactionReceipt,
    msg: I,
    receipt_proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    if tx_hash != receipt.transaction_hash {
        return Err(VerifyError::TxReceiptNotMatch);
    }
    verify_tx(tx_hash, tx_root, tx_proof)?;
    verify_receipt(msg, receipt, receipt_root, receipt_proof)
}

fn verify_tx(tx_hash: H256, root: H256, proof: Vec<H256>) -> Result<(), VerifyError> {
    let proof = proof.into_iter().map(|h| h.as_bytes().to_vec());
    Ok(())
}

// Get the frist log as IcsComponent.
fn verify_receipt<I: Object>(
    expect: I,
    receipt: TransactionReceipt,
    root: MerkleRoot,
    proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    // let actual = receipt
    //     .logs
    //     .into_iter()
    //     .next()
    //     .ok_or(VerifyError::FoundNoMessage)?;

    // if expect.encode() != actual.data {
    //     return Err(VerifyError::EventNotMatch);
    // }

    // let key = expect.as_key();

    // trie::verify_proof(root.as_bytes(), key, proof, HasherKeccak::new())
    //     .map_err(|_| VerifyError::InvalidReceiptProof)?;

    Ok(())
}

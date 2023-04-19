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
use proof::{Transaction, TransactionReceipt};
use rlp::Encodable;
use verify_mpt::verify_proof;

pub type Bytes = Vec<u8>;
pub type MerkleRoot = Vec<u8>;
pub type U256 = Vec<u8>;

pub fn verify_message<O: Object>(
    tx_root: H256,
    tx_proof: Vec<Vec<u8>>,
    receipt_root: MerkleRoot,
    tx: Transaction,
    receipt: TransactionReceipt,
    object: O,
    idx: u64,
    receipt_proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    verify_tx(tx, tx_root, idx, tx_proof)?;
    verify_receipt(object, receipt, receipt_root, receipt_proof, idx)
}

fn verify_tx(
    tx: Transaction,
    root: H256,
    idx: u64,
    proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    let key = rlp::encode(&idx);
    let value: Vec<u8> = tx.rlp().as_ref().into();
    if verify_proof(&proof, root.as_bytes(), &key, &value) {
        Ok(())
    } else {
        Err(VerifyError::InvalidTxProof)
    }
}

fn verify_receipt<O: Object>(
    expect: O,
    receipt: TransactionReceipt,
    root: MerkleRoot,
    proof: Vec<Vec<u8>>,
    idx: u64,
) -> Result<(), VerifyError> {
    let actual = receipt
        .logs
        .iter()
        .next()
        .ok_or(VerifyError::FoundNoMessage)?;

    if expect.encode() != actual.data.as_ref() {
        return Err(VerifyError::EventNotMatch);
    }

    let key: Vec<u8> = rlp::encode(&idx).as_ref().into();

    if verify_proof(&proof, root.as_ref(), &key, receipt.rlp_bytes().as_ref()) {
        Ok(())
    } else {
        Err(VerifyError::InvalidReceiptProof)
    }
}

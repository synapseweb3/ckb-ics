use alloc::vec::Vec;
use core::cell::RefCell;
use ethereum_types::H256;

use axon_tools::types::{Block as AxonBlock, Proof as AxonBlockProof, ValidatorExtend};
use axon_types::metadata::MetadataCellDataReader;
use molecule::prelude::*;
use rlp_derive::{RlpDecodable, RlpEncodable};
use tiny_keccak::{Hasher, Keccak};

use crate::handler::Client;
use crate::object::VerifyError;
use crate::proto::client::Height;

pub mod verify;

// Make rlp_derive happy.
pub type ProofNode = Vec<u8>;

#[derive(RlpDecodable, RlpEncodable)]
pub struct AxonCommitmentProof {
    pub block: AxonBlock,
    pub previous_state_root: H256,
    pub block_proof: AxonBlockProof,
    pub account_proof: Vec<ProofNode>,
    pub storage_proof: Vec<ProofNode>,
}

#[derive(Default)]
pub struct AxonClient {
    pub ibc_handler_address: [u8; 20],
    pub validators: RefCell<Vec<ValidatorExtend>>,
}

impl Client for AxonClient {
    fn verify_membership(
        &self,
        height: Height,
        // delay_time_period: u64,
        // delay_block_period: u64,
        proof: &[u8],
        // Assume prefix is always "ibc". This is true for axon and ckb.
        // prefix: &[u8],
        path: &[u8],
        value: &[u8],
    ) -> Result<(), VerifyError> {
        let AxonCommitmentProof {
            block,
            previous_state_root,
            account_proof,
            storage_proof,
            block_proof,
        } = rlp::decode(proof).map_err(|_| VerifyError::SerdeError)?;

        let block_state_root = block.header.state_root;
        assert_eq!(
            height,
            Height {
                revision_number: 0,
                revision_height: block.header.number,
            }
        );

        axon_tools::verify_proof(
            block,
            previous_state_root,
            &mut self.validators.borrow_mut(),
            block_proof,
        )
        .map_err(|_| VerifyError::InvalidReceiptProof)?;

        verify::verify_account_and_storage(
            block_state_root.as_bytes(),
            &self.ibc_handler_address,
            &account_proof,
            commitment_slot(path),
            keccak256(value),
            &storage_proof,
        )?;

        Ok(())
    }
}

impl AxonClient {
    pub fn new(ibc_handler_address: [u8; 20], metadata_cell_data: &[u8]) -> Result<Self, VerifyError> {
        let metadata_cell_data =
            MetadataCellDataReader::from_slice(metadata_cell_data).map_err(|_| VerifyError::SerdeError)?;
        let metadata = metadata_cell_data.metadata().get(0).ok_or(VerifyError::SerdeError)?;
        let mut validators: Vec<ValidatorExtend> = Vec::new();
        for v in metadata.validators().iter() {
            let bls_pub_key = v.bls_pub_key().raw_data().to_vec();
            let pub_key = v.pub_key().raw_data().to_vec();
            let address: [u8; 20] = v
                .address()
                .raw_data()
                .try_into()
                .map_err(|_| VerifyError::SerdeError)?;
            let propose_weight =
                u32::from_le_bytes(v.propose_weight().as_slice().try_into().unwrap());
            let vote_weight = u32::from_le_bytes(v.vote_weight().as_slice().try_into().unwrap());
            let validator = ValidatorExtend {
                bls_pub_key: bls_pub_key.into(),
                pub_key: pub_key.into(),
                address: address.into(),
                propose_weight,
                vote_weight,
            };
            validators.push(validator);
        }
        Ok(AxonClient {
            ibc_handler_address,
            validators: validators.into(),
        })
    }
}

fn keccak256(slice: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(slice);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

/// Storage slot for commitment path.
pub fn commitment_slot(path: &[u8]) -> [u8; 32] {
    let mut h = Keccak::v256();
    h.update(&keccak256(path));
    h.update(&[0u8; 32]);
    let mut o = [0u8; 32];
    h.finalize(&mut o);
    o
}

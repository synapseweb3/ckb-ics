use alloc::vec::Vec;
use ethereum_types::H256;
use core::cell::RefCell;

use tiny_keccak::{Hasher, Keccak};
use axon_tools::types::{Block as AxonBlock, ValidatorExtend, Proof as AxonBlockProof};
use axon_types::metadata::Metadata;
use molecule::prelude::Entity;
use rlp_derive::{RlpDecodable, RlpEncodable};

use crate::handler::Client;
use crate::object::VerifyError;
use crate::proto::client::Height;

pub mod verify;

// Make rlp_derive happy.
pub type ProofNode = Vec<u8>;

#[derive(RlpDecodable, RlpEncodable)]
pub struct AxonCommitmentProof {
    block: AxonBlock,
    previous_state_root: H256,
    block_proof: AxonBlockProof,
    account_proof: Vec<ProofNode>,
    storage_proof: Vec<ProofNode>,
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
            false,
            false,
        )?;

        Ok(())
    }
}

impl AxonClient {
    pub fn new(ibc_handler_address: [u8; 20], slice: &[u8]) -> Result<Self, VerifyError> {
        let metadata = Metadata::from_slice(slice).map_err(|_| VerifyError::SerdeError)?;
        let validators = metadata.validators();
        let mut client_validators: Vec<ValidatorExtend> = Vec::new();
        for i in 0..validators.len() {
            let v = validators.get(i).unwrap();
            let bls_pub_key = v.bls_pub_key().raw_data().to_vec();
            let pub_key = v.pub_key().raw_data().to_vec();
            let address_data = v.address().raw_data();
            let address: [u8; 20] = address_data
                .as_ref()
                .try_into()
                .map_err(|_| VerifyError::SerdeError)?;
            let height: [u8; 4] = v.propose_weight().as_slice().try_into().unwrap();
            let weight: [u8; 4] = v.vote_weight().as_slice().try_into().unwrap();
            let validator = ValidatorExtend {
                bls_pub_key: bls_pub_key.into(),
                pub_key: pub_key.into(),
                address: address.into(),
                propose_weight: u32::from_le_bytes(height),
                vote_weight: u32::from_le_bytes(weight),
            };
            client_validators.push(validator);
        }
        Ok(AxonClient {
            ibc_handler_address,
            validators: client_validators.into(),
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

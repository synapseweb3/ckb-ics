// Reference to ehters-core
use alloc::{string::String, vec::Vec};
use axon_tools::types::{AxonBlock, AxonHeader, Proof as AxonProof};
use ethereum_types::{Address, Bloom, H256, U256, U64};
use rlp::{Decodable, Encodable, RlpStream};
use rlp_derive::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};

#[derive(Debug)]
pub struct ObjectProof {
    pub receipt: TransactionReceipt,
    pub receipt_proof: Vec<Vec<u8>>,
    pub block: AxonBlock,
    pub state_root: H256,
    pub axon_proof: AxonProof,
}

impl Encodable for ObjectProof {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append(&self.receipt)
            .append_list::<Vec<_>, Vec<_>>(&self.receipt_proof)
            .append(&self.block)
            .append(&self.state_root)
            .append(&self.axon_proof);
    }
}

impl Decodable for ObjectProof {
    fn decode(r: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let receipt: TransactionReceipt = r.val_at(0)?;
        let receipt_proof: Vec<Vec<u8>> = r.list_at(1)?;
        let block: AxonBlock = r.val_at(2)?;
        let state_root: H256 = r.val_at(3)?;
        let axon_proof: AxonProof = r.val_at(4)?;
        Ok(ObjectProof {
            receipt,
            receipt_proof,
            block,
            state_root,
            axon_proof,
        })
    }
}

impl Default for ObjectProof {
    fn default() -> Self {
        Self {
            receipt: Default::default(),
            receipt_proof: Default::default(),
            block: AxonBlock {
                header: AxonHeader {
                    prev_hash: Default::default(),
                    proposer: Default::default(),
                    state_root: Default::default(),
                    transactions_root: Default::default(),
                    signed_txs_hash: Default::default(),
                    receipts_root: Default::default(),
                    log_bloom: Default::default(),
                    difficulty: Default::default(),
                    timestamp: Default::default(),
                    number: Default::default(),
                    gas_used: Default::default(),
                    gas_limit: Default::default(),
                    extra_data: Default::default(),
                    mixed_hash: Default::default(),
                    nonce: Default::default(),
                    base_fee_per_gas: Default::default(),
                    proof: AxonProof {
                        number: Default::default(),
                        round: Default::default(),
                        block_hash: Default::default(),
                        signature: Default::default(),
                        bitmap: Default::default(),
                    },
                    call_system_script_count: Default::default(),
                    chain_id: Default::default(),
                },
                tx_hashes: Vec::new(),
            },
            state_root: Default::default(),
            axon_proof: AxonProof {
                number: Default::default(),
                round: Default::default(),
                block_hash: Default::default(),
                signature: Default::default(),
                bitmap: Default::default(),
            },
        }
    }
}

#[derive(Debug, Default, RlpEncodable, RlpDecodable)]
pub struct TransactionReceipt {
    /// Transaction hash.
    pub transaction_hash: H256,
    /// Index within the block.
    pub transaction_index: U64,
    /// Hash of the block this transaction was included within.
    pub block_hash: Option<H256>,
    /// Number of the block this transaction was included within.
    pub block_number: Option<U64>,
    /// address of the sender.
    pub from: Address,
    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<Address>,
    /// Cumulative gas used within the block after this was executed.
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    pub gas_used: Option<U256>,
    /// Contract address created, or `None` if not a deployment.
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<U64>,
    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub root: Option<H256>,
    /// Logs bloom
    pub logs_bloom: Bloom,
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    pub transaction_type: Option<U64>,
    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    pub effective_gas_price: Option<U256>,
}

#[derive(Debug, Default, RlpEncodable, RlpDecodable)]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    pub block_hash: Option<H256>,

    /// Block Number
    pub block_number: Option<U64>,

    /// Transaction Hash
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    pub transaction_index: Option<U64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    pub transaction_log_index: Option<U256>,

    /// Log Type
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    pub removed: Option<bool>,
}

#[derive(Debug, Default, RlpEncodable, RlpDecodable)]
pub struct Bytes(pub bytes::Bytes);

impl From<Vec<u8>> for Bytes {
    fn from(src: Vec<u8>) -> Self {
        Self(src.into())
    }
}

impl From<bytes::Bytes> for Bytes {
    fn from(src: bytes::Bytes) -> Self {
        Self(src)
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(RlpDecodableWrapper, RlpEncodableWrapper, Debug, Default)]
pub struct AccessList(pub Vec<AccessListItem>);

impl From<Vec<AccessListItem>> for AccessList {
    fn from(src: Vec<AccessListItem>) -> AccessList {
        AccessList(src)
    }
}

#[derive(RlpDecodable, RlpEncodable, Debug, Default)]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}

// NOTE The implementation in `ethers` is incorrect.
// Ref:
// - https://eips.ethereum.org/EIPS/eip-2718#receipts
// - https://github.com/gakonst/ethers-rs/blob/v1.0/ethers-core/src/types/transaction/response.rs#L443-L451
pub fn encode_receipt(receipt: &TransactionReceipt) -> Vec<u8> {
    let legacy_receipt_encoded = receipt.rlp_bytes();
    if let Some(tx_type) = receipt.transaction_type {
        let tx_type = tx_type.as_u64();
        if tx_type == 0 {
            legacy_receipt_encoded.to_vec()
        } else {
            [&tx_type.to_be_bytes()[7..8], &legacy_receipt_encoded].concat()
        }
    } else {
        legacy_receipt_encoded.to_vec()
    }
}

fn normalize_v(v: u64, chain_id: U64) -> u64 {
    if v > 1 {
        v - chain_id.as_u64() * 2 - 35
    } else {
        v
    }
}

// Reference to ehters-core
use alloc::vec::Vec;
use cstr_core::CString;
use ethereum_types::{Address, Bloom, H256, U256, U64};
use rlp::{Encodable, RlpStream};
use rlp_derive::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};

pub struct ObjectProof {
    pub tx: Transaction,
    pub tx_root: H256,
    pub tx_proof: Vec<H256>,

    pub receipt: TransactionReceipt,
    pub receipt_root: H256,
    pub receipt_proof: Vec<H256>,

    // the transaction idx in the block
    pub idx: u32,
}

pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    pub block_number: Option<U64>,

    /// Transaction Index. None when pending.
    pub transaction_index: Option<U64>,

    /// Sender
    pub from: Address,

    /// Recipient (None when contract creation)
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// Gas Price, null for Type 2 transactions
    pub gas_price: Option<U256>,

    /// Gas amount
    pub gas: U256,

    /// Input data
    pub input: Bytes,

    /// ECDSA recovery id
    pub v: U64,

    /// ECDSA signature r
    pub r: U256,

    /// ECDSA signature s
    pub s: U256,

    // EIP2718
    /// Transaction type, Some(2) for EIP-1559 transaction,
    /// Some(1) for AccessList transaction, None for Legacy
    pub transaction_type: Option<U64>,

    // EIP2930
    pub access_list: Option<AccessList>,

    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    pub max_priority_fee_per_gas: Option<U256>,

    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    pub max_fee_per_gas: Option<U256>,

    pub chain_id: Option<U256>,
}

impl Transaction {
    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();

        match self.transaction_type {
            // EIP-2930 (0x01)
            Some(x) if x == U64::from(1) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.gas_price);
                rlp.append(&self.gas);

                #[cfg(feature = "celo")]
                self.inject_celo_metadata(&mut rlp);

                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt_list(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
            }
            // EIP-1559 (0x02)
            Some(x) if x == U64::from(2) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.max_priority_fee_per_gas);
                rlp_opt(&mut rlp, &self.max_fee_per_gas);
                rlp.append(&self.gas);
                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt_list(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
            }
            // Legacy (0x00)
            _ => {
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.gas_price);
                rlp.append(&self.gas);

                #[cfg(feature = "celo")]
                self.inject_celo_metadata(&mut rlp);

                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp.append(&self.v);
            }
        }

        rlp.append(&self.r);
        rlp.append(&self.s);

        rlp.finalize_unbounded_list();

        let rlp_bytes: Bytes = rlp.out().freeze().into();
        let mut encoded = Vec::new();
        match self.transaction_type {
            Some(x) if x == U64::from(1) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            Some(x) if x == U64::from(2) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            _ => rlp_bytes,
        }
    }
}

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

impl Encodable for TransactionReceipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        rlp_opt(s, &self.status);
        s.append(&self.cumulative_gas_used);
        s.append(&self.logs_bloom);
        s.append_list(&self.logs);
    }
}

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
    pub log_type: Option<CString>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    pub removed: Option<bool>,
}

impl Encodable for Log {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.address);
        s.append_list(&self.topics);
        s.append(&self.data.0);
    }
}

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

fn rlp_opt<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

fn rlp_opt_list<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        // Choice of `u8` type here is arbitrary as all empty lists are encoded the same.
        rlp.append_list::<u8, u8>(&[]);
    }
}

#[derive(RlpDecodableWrapper, RlpEncodableWrapper)]
pub struct AccessList(pub Vec<AccessListItem>);

impl From<Vec<AccessListItem>> for AccessList {
    fn from(src: Vec<AccessListItem>) -> AccessList {
        AccessList(src)
    }
}

#[derive(RlpDecodable, RlpEncodable)]
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

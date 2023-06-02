#![no_std]
extern crate alloc;

pub use alloc::vec::Vec;

pub mod consts;
pub mod handler;
pub mod message;
pub mod object;
pub mod proof;
pub mod verify_mpt;

use ethereum_types::H256;
use object::{Object, VerifyError};
use proof::TransactionReceipt;
use rlp::{Encodable, RlpStream};
use verify_mpt::verify_proof;

pub type U256 = Vec<u8>;
pub type Bytes = Vec<u8>;

// The args of the connection cell's script
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConnectionArgs {
    pub client_id: [u8; 32],
}

impl ConnectionArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 32 {
            return Err(());
        }
        Ok(ConnectionArgs {
            client_id: slice[0..32].try_into().unwrap(),
        })
    }

    pub fn get_client_id(slice: &[u8]) -> &[u8] {
        &slice[0..32]
    }
}

// The args of the channel cell's script
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ChannelArgs {
    pub client_id: [u8; 32],
    // For the sake of convenience, we use a bool here to describe
    // whether this channel is open. Relayer search the the unopen channel cell
    // frequently.
    pub open: bool,
    // Relayer will search the specified channel by channel id and port id
    pub channel_id: u16,
    pub port_id: [u8; 32],
}

impl ChannelArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 67 {
            return Err(());
        }
        let client_id: [u8; 32] = slice[0..32].try_into().unwrap();
        let open = slice.get(32).unwrap() > &0;
        let channel_id = u16::from_le_bytes(slice[33..35].try_into().unwrap());
        let port_id: [u8; 32] = slice[35..67].try_into().unwrap();
        Ok(ChannelArgs {
            client_id,
            open,
            channel_id,
            port_id,
        })
    }

    pub fn get_prefix_for_searching_unopen(&self) -> Vec<u8> {
        let mut result = self.client_id.to_vec();
        let open: u8 = if self.open { 0 } else { 1 };
        result.push(open);
        result
    }

    pub fn get_prefix_for_all(&self) -> Vec<u8> {
        self.client_id.to_vec()
    }

    pub fn is_open(data: Vec<u8>) -> Result<bool, ()> {
        let open_byte = data.get(33).ok_or(())?;
        if *open_byte == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn to_args(self) -> Vec<u8> {
        let mut result = self.get_prefix_for_searching_unopen();
        result.extend(self.channel_id.to_le_bytes());
        result.extend(self.port_id);
        result
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PacketArgs {
    pub channel_id: u16,
    pub port_id: [u8; 32],
    pub sequence: u16,
    // Who pay for this capacity, the secp256k1 args
    pub owner: [u8; 32],
}

impl PacketArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 68 {
            return Err(());
        }
        let channel_id = u16::from_le_bytes(slice[0..2].try_into().unwrap());
        let port_id = slice[2..34].try_into().unwrap();
        let sequence = u16::from_le_bytes(slice[34..36].try_into().unwrap());
        let owner: [u8; 32] = slice[36..68].try_into().unwrap();
        Ok(PacketArgs {
            channel_id,
            port_id,
            sequence,
            owner,
        })
    }

    pub fn get_owner(&self) -> [u8; 32] {
        self.owner.clone()
    }

    pub fn to_args(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.channel_id.to_le_bytes());
        result.extend(self.port_id);
        result.extend(self.sequence.to_le_bytes());
        result.extend(self.owner);
        result
    }
}

pub fn verify_message<O: Object>(
    receipt_root: H256,
    receipt: TransactionReceipt,
    object: O,
    receipt_proof: Vec<Vec<u8>>,
) -> Result<(), VerifyError> {
    if let Some(first) = receipt.logs.first() {
        if object.encode() != first.data.as_ref() {
            return Err(VerifyError::InvalidReceiptProof);
        }
    } else {
        return Err(VerifyError::InvalidReceiptProof);
    }
    let idx = receipt.transaction_index.as_u64();
    verify_receipt(object, receipt, receipt_root, receipt_proof, idx)
}

fn verify_receipt<O: Object>(
    expect: O,
    receipt: TransactionReceipt,
    root: H256,
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

pub fn rlp_opt<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

pub fn rlp_opt_list<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        // Choice of `u8` type here is arbitrary as all empty lists are encoded the same.
        rlp.append_list::<u8, u8>(&[]);
    }
}

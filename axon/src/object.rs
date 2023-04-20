use crate::proof::ObjectProof;
use alloc::vec::Vec;

// use axon_protocol::types::{Bytes, U256};
use super::Bytes;
use super::U256;
use cstr_core::CString;

// ChannelEnd, ConnectionEnd
pub trait Object {
    fn encode(&self) -> Vec<u8>;
}

pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    InvalidTxProof,
    TxReceiptNotMatch,
    SerdeError,

    ConnectionsWrong,

    WrongConnectionCnt,
    WrongConnectionState,
    WrongConnectionCounterparty,
    WrongConnectionClient,
}

pub enum ClientType {
    Unknown,
    Tendermint,
    Ethereum,
    Axon,
    Ckb,
}

#[derive(PartialEq, Eq)]
pub enum State {
    Unknown,
    Init,
    OpenTry,
    Open,
    Closed,
    Frozen,
}

pub enum Ordering {
    Unknown,
    Unordered,
    Ordered,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConnectionId {
    pub client_id: CString,
    pub connection_id: Option<CString>,
    // pub commitment_prefix: Bytes,
}

pub struct ChannelId {
    pub port_id: CString,
    pub channel_id: CString,
}

pub struct Proofs {
    pub height: U256,
    pub object_proof: ObjectProof,
    pub client_proof: Vec<u8>,
}

pub struct Packet {
    pub sequence: U256,
    pub source_port_id: CString,
    pub source_channel_id: CString,
    pub destination_port_id: CString,
    pub destination_channel_id: CString,
    pub payload: Bytes,
    pub timeout_height: Bytes, // bytes32
    pub timeout_timestamp: U256,
}

pub struct ClientState {
    pub chain_id: CString,
    pub client_type: ClientType,
    pub latest_height: Bytes,
}

impl Object for ClientState {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct ConsensusState {
    pub timestamp: U256,
    pub commitment_root: Bytes, // bytes32
    pub extra_payload: Bytes,
}

#[derive(PartialEq, Eq)]
pub struct ConnectionEnd {
    pub connection_id: ConnectionId,
    pub state: State,
    pub client_id: CString,
    pub counterparty: ConnectionId,
    // pub versions: Vec<String>,
    pub delay_period: U256,
}

impl Object for ConnectionEnd {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct ChannelEnd {
    pub channel_id: ChannelId,
    pub state: State,
    pub ordering: Ordering,
    pub remote: ChannelId,
    // pub connection_hops: Vec<String>,
    // pub version: String,
}

impl Object for ChannelEnd {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

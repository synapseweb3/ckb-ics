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

#[derive(Debug)]
pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    InvalidTxProof,
    TxReceiptNotMatch,
    SerdeError,

    WrongClient,

    ConnectionsWrong,

    WrongConnectionCnt,
    WrongConnectionState,
    WrongConnectionCounterparty,
    WrongConnectionClient,
    WrongConnectionNextChannelNumber,

    WrongChannelState,
    WrongChannel,

    WrongPacketSequence,
    WrongPacketStatus,
    WrongPacketContent,
}

#[derive(Debug, Clone, Default)]
pub enum ClientType {
    #[default]
    Unknown,
    Tendermint,
    Ethereum,
    Axon,
    Ckb,
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub enum State {
    #[default]
    Unknown,
    Init,
    OpenTry,
    Open,
    Closed,
    Frozen,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Ordering {
    #[default]
    Unknown,
    Unordered,
    Ordered,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ConnectionCounterparty {
    pub client_id: CString,
    pub connection_id: Option<CString>,
    // pub commitment_prefix: Bytes,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ChannelCounterparty {
    pub port_id: CString,
    pub channel_id: CString,
}

#[derive(Debug, Default)]
pub struct Proofs {
    pub height: U256,
    pub object_proof: ObjectProof,
    pub client_proof: Vec<u8>,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Packet {
    pub sequence: u16,
    pub source_port_id: CString,
    pub source_channel_id: CString,
    pub destination_port_id: CString,
    pub destination_channel_id: CString,
    pub data: Bytes,
    pub timeout_height: Bytes, // bytes32
    pub timeout_timestamp: u64,
}

impl Object for Packet {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ConnectionEnd {
    pub state: State,
    pub client_id: CString,
    pub counterparty: ConnectionCounterparty,
    pub delay_period: u64,
    // pub versions: Vec<String>,
}

impl Object for ConnectionEnd {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct ChannelEnd {
    pub state: State,
    pub ordering: Ordering,
    pub remote: ChannelCounterparty,
    pub connection_hops: Vec<CString>,
    // pub version: CString,
}

impl Object for ChannelEnd {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

// The ack of the packet
pub struct PacketAck {
    pub ack: Vec<u8>,
    pub packet: Packet,
}

impl Object for PacketAck {
    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

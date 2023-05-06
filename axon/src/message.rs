/// These messages are used to send to CKB. We named some fields with the
/// suffix `a or b` according to Cosmos's convention.
use super::object::*;
use super::Vec;
use super::{Bytes, U256};
// use axon_protocol::types::{Bytes, U256};
use cstr_core::CString;
use rlp::{Decodable, Encodable, Rlp};

pub struct Envelope {
    pub msg_type: MsgType,
    pub content: Vec<u8>,
}

impl Encodable for Envelope {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for Envelope {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub enum MsgType {
    MsgClientCreate,
    MsgClientUpdate,
    MsgClientMisbehaviour,

    MsgConnectionOpenInit,
    MsgConnectionOpenTry,
    MsgConnectionOpenAck,
    MsgConnectionOpenConfirm,

    MsgChannelOpenInit,
    MsgChannelOpenTry,
    MsgChannelOpenAck,
    MsgChannelOpenConfirm,
    MsgChannelCloseInit,
    MsgChannelCloseConfirm,

    MsgSendPacket,
    MsgRecvPacket,
    MsgAckPacket,
    // Business side sends this message after handling MsgRecvPacket
    MsgAckOutboxPacket,
    // Business side sends this message after handling MsgAckPacket
    MsgAckInboxPacket,
    // Relayer side sends this message after
    // the packet is finsihed and they could get back their capacity
    MsgFinishPacket,
    MsgTimeoutPacket,
}

pub struct MsgClientCreate {
    pub client: ClientState,
    pub consensus_state: ConsensusState,
}

impl Encodable for MsgClientCreate {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgClientCreate {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct MsgClientUpdate {
    pub client_id: CString,
    pub header_bytes: Bytes,
}

impl Encodable for MsgClientUpdate {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgClientUpdate {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct MsgClientMisbehaviour {
    pub client_id: CString,
    pub header1_bytes: Bytes,
    pub header2_bytes: Bytes,
}

impl Encodable for MsgClientMisbehaviour {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgClientMisbehaviour {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of Chain B.
pub struct MsgConnectionOpenInit {
    // In CKB tx, Connection is discribed in the Output.
    // pub client_id_on_a: CString,
    // pub counterparty: ConnectionCounterparty,
    // pub version: Option<CString>,
    // pub delay_duration: u64,
}

impl Encodable for MsgConnectionOpenInit {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgConnectionOpenInit {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
pub struct MsgConnectionOpenTry {
    pub client_id_on_b: CString,
    // TODO: this field is useful when CKB is connecting to chains but Axon.
    // pub client_state_of_b_on_a: Bytes,
    pub counterparty: ConnectionCounterparty,
    pub proof: Proofs,
    // pub counterparty_versions: Vec<CString>,
    pub delay_period: u64,
    // deprecated
    // pub previous_connection_id: CString,
}

impl Encodable for MsgConnectionOpenTry {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgConnectionOpenTry {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
pub struct MsgConnectionOpenAck {
    // In CKB, IBC connection cells are stored in a a vector in a cell.
    // This message just convey the idx of the connection cell of it
    // and the content of that cell would be stored in witness of the tx.
    pub conn_id_on_a: usize,
    pub conn_id_on_b: CString,
    pub client_state_of_a_on_b: ClientState,
    pub proof_conn_end_on_b: Proofs,
    pub version: CString,
}

impl Encodable for MsgConnectionOpenAck {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgConnectionOpenAck {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
pub struct MsgConnectionOpenConfirm {
    // In CKB, IBC connection cells are stored in a a vector in a cell.
    // This message just convey the idx of the connection cell of it
    // and the content of that cell would be stored in witness of the tx.
    pub conn_id_on_b: usize,
    pub proofs: Proofs,
}

impl Encodable for MsgConnectionOpenConfirm {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgConnectionOpenConfirm {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

// Per our convention, this message is sent to chain A
pub struct MsgChannelOpenInit {
    pub port_id_on_a: CString,
    pub connection_hops_on_a: Vec<CString>,
    pub port_id_on_b: CString,
    pub ordering: Ordering,
    // pub version: Verison
}

impl Encodable for MsgChannelOpenInit {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelOpenInit {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain B.
pub struct MsgChannelOpenTry {
    pub port_id_on_b: CString,
    // CKB's channel doesn't have this field
    // pub connection_hops_on_b: Vec<CString>,
    pub port_id_on_a: CString,
    pub chain_id_on_a: CString,
    pub proof_chan_end_on_a: Proofs,
    pub ordering: Ordering,
    pub connection_hops_on_a: Vec<CString>,
    // pub previous_channal_id: CString,
    // pub version_proposal: Version,
}

impl Encodable for MsgChannelOpenTry {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelOpenTry {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain A.
pub struct MsgChannelOpenAck {
    // In CKB tx, these 2 fields are found in cell dep and witness.
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
    pub chain_id_on_b: CString,
    pub proofs: Proofs,
    pub connection_hops_on_b: Vec<CString>,
}

impl Encodable for MsgChannelOpenAck {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelOpenAck {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

/// Per our convention, this message is sent to chain B.
pub struct MsgChannelOpenConfirm {
    pub port_id_on_b: CString,
    pub chain_id_on_b: CString,
    pub channel_id: ChannelCounterparty,
    pub proofs: Proofs,
    pub connection_hops_on_b: Vec<CString>,
}

impl Encodable for MsgChannelOpenConfirm {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelOpenConfirm {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

// Per our convention, this message is sent to chain A.
pub struct MsgChannelCloseInit {
    // In CKB tx, these 2 fields are found in witness.
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
}

impl Encodable for MsgChannelCloseInit {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelCloseInit {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

// Per our convention, this message is sent to chain B.
pub struct MsgChannelCloseConfirm {
    // In CKB tx, these 2 fields are found in witness.
    // pub port_id_on_b: CString,
    // pub port_id_on_b: CString,
    pub proofs: Proofs,
}

impl Encodable for MsgChannelCloseConfirm {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgChannelCloseConfirm {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

// As our CKB convention, the content of the packet is stored in Witness.
// We don't need to place it again in this message.
pub struct MsgSendPacket {}

impl Encodable for MsgSendPacket {
    fn rlp_append(&self, _: &mut rlp::RlpStream) {}
}

impl Decodable for MsgSendPacket {
    fn decode(_: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(MsgSendPacket {})
    }
}

pub struct MsgRecvPacket {
    pub packet: Packet,
    pub proofs: Proofs,
}

impl Encodable for MsgRecvPacket {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgRecvPacket {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct MsgAckPacket {
    // pub packet: Packet,
    pub acknowledgement: Bytes,
    pub proofs: Proofs,
}

impl Encodable for MsgAckPacket {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgAckPacket {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct MsgTimeoutPacket {
    pub packet: Packet,
    pub next_sequence_recv: U256,
    pub proofs: Proofs,
}

impl Encodable for MsgTimeoutPacket {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgTimeoutPacket {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

// Business side sends this message after handling MsgAckPacket
pub struct MsgAckInboxPacket {}

impl Encodable for MsgAckInboxPacket {
    fn rlp_append(&self, _: &mut rlp::RlpStream) {}
}

impl Decodable for MsgAckInboxPacket {
    fn decode(_: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(MsgAckInboxPacket {})
    }
}

// Business side sends this message after handling MsgRecvPacket
pub struct MsgAckOutboxPacket {
    pub ack: Vec<u8>,
    // pub packet: Packet,
}

impl Encodable for MsgAckOutboxPacket {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for MsgAckOutboxPacket {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

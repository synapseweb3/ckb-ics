use super::types::*;

use axon_protocol::types::{Bytes, U256};
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
    MsgRecvPacket,
    MsgAckPacket,
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

pub struct MsgConnectionOpenInit {
    pub client_id: CString,
    pub counterparty: ConnectionId,
    pub version: CString,
    pub delay_duration: U256,
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

pub struct MsgConnectionOpenTry {
    pub previous_connection_id: CString,
    pub client_id: CString,
    pub client_state: ClientState,
    pub counterparty: ConnectionId,
    pub counterparty_versions: Vec<CString>,
    pub delay_period: U256,
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

pub struct MsgConnectionOpenAck {
    pub connection_id: CString,
    pub counterparty_connection_id: CString,
    pub client_state: ClientState,
    pub proofs: Proofs,
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

pub struct MsgConnectionOpenConfirm {
    pub connection_id: CString,
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

pub struct MsgChannelOpenInit {
    pub port_id: CString,
    pub channel: ChannelEnd,
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

pub struct MsgChannelOpenTry {
    pub port_id: CString,
    pub previous_channel_id: ChannelId,
    pub channel: ChannelEnd,
    pub counterparty_version: CString,
    pub proofs: Proofs,
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

pub struct MsgChannelOpenAck {
    pub port_id: CString,
    pub previous_channel_id: ChannelId,
    pub channel: ChannelEnd,
    pub counterparty_version: CString,
    pub proofs: Proofs,
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

pub struct MsgChannelOpenConfirm {
    pub port_id: CString,
    pub channel_id: ChannelId,
    pub proofs: Proofs,
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

pub struct MsgChannelCloseInit {
    pub port_id: CString,
    pub channel_id: ChannelId,
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

pub struct MsgChannelCloseConfirm {
    pub port_id: CString,
    pub channel_id: ChannelId,
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
    pub packet: Packet,
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

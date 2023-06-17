/// These messages are used to send to CKB. We named some fields with the
/// suffix `a or b` according to Cosmos's convention.
use super::object::*;
use super::Vec;
use super::{Bytes, U256};
// use axon_protocol::types::{Bytes, U256};
use rlp::{Decodable, Encodable, Rlp};
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

#[derive(RlpDecodable, RlpEncodable)]
pub struct Envelope {
    pub msg_type: MsgType,
    pub content: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    MsgClientCreate = 1,
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

impl Encodable for MsgType {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let t = self.clone() as u8;
        s.append(&t);
    }
}

impl Decodable for MsgType {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        let t: u8 = rlp.as_val()?;
        match t {
            1 => Ok(MsgType::MsgClientCreate),
            2 => Ok(MsgType::MsgClientUpdate),
            3 => Ok(MsgType::MsgClientMisbehaviour),

            4 => Ok(MsgType::MsgConnectionOpenInit),
            5 => Ok(MsgType::MsgConnectionOpenTry),
            6 => Ok(MsgType::MsgConnectionOpenAck),
            7 => Ok(MsgType::MsgConnectionOpenConfirm),

            8 => Ok(MsgType::MsgChannelOpenInit),
            9 => Ok(MsgType::MsgChannelOpenTry),
            10 => Ok(MsgType::MsgChannelOpenAck),
            11 => Ok(MsgType::MsgChannelOpenConfirm),
            12 => Ok(MsgType::MsgChannelCloseInit),
            13 => Ok(MsgType::MsgChannelCloseConfirm),

            14 => Ok(MsgType::MsgSendPacket),
            15 => Ok(MsgType::MsgRecvPacket),
            16 => Ok(MsgType::MsgAckPacket),
            17 => Ok(MsgType::MsgAckOutboxPacket),
            18 => Ok(MsgType::MsgAckInboxPacket),

            19 => Ok(MsgType::MsgFinishPacket),
            20 => Ok(MsgType::MsgTimeoutPacket),
            _ => Err(rlp::DecoderError::Custom("msg type decode error")),
        }
    }
}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgClientCreate {}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgClientUpdate {}

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of Chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgConnectionOpenInit {
    // In CKB tx, Connection is discribed in the Output.
    // pub client_id_on_a: CString,
    // pub counterparty: ConnectionCounterparty,
    // pub version: Option<CString>,
    // pub delay_duration: u64,
}

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgConnectionOpenTry {
    // pub client_id_on_b: CString,
    // TODO: this field is useful when CKB is connecting to chains but Axon.
    // pub client_state_of_b_on_a: Bytes,
    // pub counterparty: ConnectionCounterparty,
    pub proof: Proofs,
    // pub counterparty_versions: Vec<CString>,
    // pub delay_period: u64,
    // deprecated
    // pub previous_connection_id: CString,
}

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgConnectionOpenAck {
    // In CKB, IBC connection cells are stored in a a vector in a cell.
    // This message just convey the idx of the connection cell of it
    // and the content of that cell would be stored in witness of the tx.
    pub conn_id_on_a: usize,
    // pub conn_id_on_b: String,
    // pub client_state_of_a_on_b: ClientState,
    pub proof_conn_end_on_b: Proofs,
    // pub version: CString,
}

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgConnectionOpenConfirm {
    // In CKB, IBC connection cells are stored in a a vector in a cell.
    // This message just convey the idx of the connection cell of it
    // and the content of that cell would be stored in witness of the tx.
    pub conn_id_on_b: usize,
    pub proofs: Proofs,
}

// Per our convention, this message is sent to chain A
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelOpenInit {
    // pub port_id_on_a: CString,
    // pub connection_hops_on_a: Vec<CString>,
    // pub port_id_on_b: CString,
    // pub ordering: Ordering,
    // pub version: Verison
}

/// Per our convention, this message is sent to chain B.
#[derive(RlpEncodable, RlpDecodable)]
pub struct MsgChannelOpenTry {
    // pub port_id_on_b: CString,
    // CKB's channel doesn't have this field
    // pub connection_hops_on_b: Vec<CString>,
    // pub port_id_on_a: CString,
    // pub chain_id_on_a: CString,
    pub proof_chan_end_on_a: Proofs,
    // pub ordering: Ordering,
    // pub connection_hops_on_a: Vec<String>,
    // pub previous_channal_id: CString,
    // pub version_proposal: Version,
}

/// Per our convention, this message is sent to chain A.
#[derive(RlpEncodable, RlpDecodable)]
pub struct MsgChannelOpenAck {
    // In CKB tx, these 2 fields are found in cell dep and witness.
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
    // pub chain_id_on_b: CString,
    pub proofs: Proofs,
    // pub connection_hops_on_b: Vec<String>,
}

/// Per our convention, this message is sent to chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelOpenConfirm {
    // pub port_id_on_b: CString,
    // pub chain_id_on_b: CString,
    // pub channel_id: ChannelCounterparty,
    pub proofs: Proofs,
    // pub connection_hops_on_b: Vec<String>,
}

// Per our convention, this message is sent to chain A.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelCloseInit {
    // In CKB tx, these 2 fields are found in witness.
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
}

// Per our convention, this message is sent to chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelCloseConfirm {
    // In CKB tx, these 2 fields are found in witness.
    // pub port_id_on_b: CString,
    // pub port_id_on_b: CString,
    pub proofs: Proofs,
}

// As our CKB convention, the content of the packet is stored in Witness.
// We don't need to place it again in this message.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgSendPacket {}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgRecvPacket {
    pub proofs: Proofs,
}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgAckPacket {
    // pub packet: Packet,
    pub acknowledgement: Bytes,
    pub proofs: Proofs,
}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgTimeoutPacket {
    pub packet: Packet,
    pub next_sequence_recv: U256,
    pub proofs: Proofs,
}

// Business side sends this message after handling MsgAckPacket
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgAckInboxPacket {}

// Business side sends this message after handling MsgRecvPacket
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgAckOutboxPacket {
    pub ack: Vec<u8>,
    // pub packet: Packet,
}

#[cfg(test)]
mod tests {
    use crate::message::Envelope;

    use super::MsgType;
    use super::Vec;

    #[test]
    fn msg_type_encode_decode() {
        let mut types = Vec::new();
        types.push(MsgType::MsgClientCreate);
        types.push(MsgType::MsgClientUpdate);
        types.push(MsgType::MsgClientMisbehaviour);

        types.push(MsgType::MsgConnectionOpenInit);
        types.push(MsgType::MsgConnectionOpenTry);
        types.push(MsgType::MsgConnectionOpenAck);
        types.push(MsgType::MsgConnectionOpenConfirm);

        types.push(MsgType::MsgChannelOpenInit);
        types.push(MsgType::MsgChannelOpenTry);
        types.push(MsgType::MsgChannelOpenAck);
        types.push(MsgType::MsgChannelOpenConfirm);
        types.push(MsgType::MsgChannelCloseInit);
        types.push(MsgType::MsgChannelCloseConfirm);

        types.push(MsgType::MsgSendPacket);
        types.push(MsgType::MsgRecvPacket);
        types.push(MsgType::MsgAckPacket);
        types.push(MsgType::MsgAckOutboxPacket);
        types.push(MsgType::MsgAckInboxPacket);
        types.push(MsgType::MsgFinishPacket);
        types.push(MsgType::MsgTimeoutPacket);

        for i in 1..types.len() {
            let envelope = Envelope {
                msg_type: types[i - 1],
                content: Vec::new(),
            };

            let data = rlp::encode(&envelope).to_vec();
            let actual = rlp::decode::<Envelope>(&data).unwrap();
            assert_eq!(actual.msg_type, types[i - 1]);
        }
    }
}

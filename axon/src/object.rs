use crate::convert_client_id_to_string;
use crate::handler::get_channel_id_str;
use crate::proof::ObjectProof;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rlp::Decodable;
use rlp::Encodable;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use super::U256;

// ChannelEnd, ConnectionEnd
pub trait Object: Sized {
    fn encode(&self) -> Vec<u8>;

    fn decode(_: &[u8]) -> Result<Self, VerifyError>;
}

#[derive(Debug)]
pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    SerdeError,

    WrongClient,

    ConnectionsWrong,

    WrongConnectionCnt,
    WrongConnectionState,
    WrongConnectionCounterparty,
    WrongConnectionClient,
    WrongConnectionNextChannelNumber,
    WrongConnectionArgs,

    WrongChannelState,
    WrongChannel,
    WrongChannelArgs,

    WrongPacketSequence,
    WrongPacketStatus,
    WrongPacketContent,
    WrongPacketArgs,
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
#[repr(u8)]
pub enum State {
    #[default]
    Unknown = 1,
    Init,
    OpenTry,
    Open,
    Closed,
    Frozen,
}

impl Encodable for State {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let state = *self as u8;
        s.begin_list(1);
        s.append(&state);
    }
}

impl Decodable for State {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let state: u8 = rlp.val_at(0)?;
        match state {
            1 => Ok(State::Unknown),
            2 => Ok(State::Init),
            3 => Ok(State::OpenTry),
            4 => Ok(State::Open),
            5 => Ok(State::Closed),
            6 => Ok(State::Frozen),
            _ => Err(rlp::DecoderError::Custom("invalid state")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
#[repr(u8)]
pub enum Ordering {
    #[default]
    Unknown = 1,
    Unordered,
    Ordered,
}

impl Encodable for Ordering {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let ordering = *self as u8;
        s.begin_list(1);
        s.append(&ordering);
    }
}

impl Decodable for Ordering {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let ordering: u8 = rlp.val_at(0)?;
        match ordering {
            1 => Ok(Ordering::Unknown),
            2 => Ok(Ordering::Unordered),
            3 => Ok(Ordering::Ordered),
            _ => Err(rlp::DecoderError::Custom("invalid ordering")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct ConnectionCounterparty {
    pub client_id: String,
    pub connection_id: Option<String>,
    // pub commitment_prefix: Bytes,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, RlpEncodable, RlpDecodable)]
pub struct ChannelCounterparty {
    pub port_id: String,
    pub channel_id: String,
}

#[derive(Default, Debug, RlpEncodable, RlpDecodable)]
pub struct Proofs {
    pub height: U256,
    pub object_proof: ObjectProof,
    pub client_proof: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Packet {
    pub sequence: u16,
    pub source_port_id: String,
    pub source_channel_id: String,
    pub destination_port_id: String,
    pub destination_channel_id: String,
    pub data: Vec<u8>,
    // todo
    // pub timeout_height: Bytes, // bytes32
    // pub timeout_timestamp: u64,
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            sequence: Default::default(),
            source_port_id: String::from_utf8_lossy([0u8; 32].as_slice()).to_string(),
            source_channel_id: get_channel_id_str(0),
            destination_port_id: String::from_utf8_lossy([0u8; 32].as_slice()).to_string(),
            destination_channel_id: get_channel_id_str(0),
            data: Default::default(),
        }
    }
}

impl Object for Packet {
    fn encode(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, VerifyError> {
        rlp::decode(data).map_err(|_| VerifyError::SerdeError)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, RlpEncodable, RlpDecodable)]
pub struct ConnectionEnd {
    pub state: State,
    pub client_id: String,
    pub counterparty: ConnectionCounterparty,
    pub delay_period: u64,
    // pub versions: Vec<String>,
}

impl Default for ConnectionEnd {
    fn default() -> Self {
        Self {
            state: Default::default(),
            client_id: convert_client_id_to_string([0u8; 32]),
            counterparty: Default::default(),
            delay_period: Default::default(),
        }
    }
}

impl Object for ConnectionEnd {
    fn encode(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, VerifyError> {
        rlp::decode(data).map_err(|_| VerifyError::SerdeError)
    }
}

#[derive(RlpEncodable, RlpDecodable)]
pub struct ChannelEnd {
    pub state: State,
    pub ordering: Ordering,
    pub remote: ChannelCounterparty,
    pub connection_hops: Vec<String>,
    // pub version: CString,
}

impl Object for ChannelEnd {
    fn encode(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, VerifyError> {
        rlp::decode(data).map_err(|_| VerifyError::SerdeError)
    }
}

// The ack of the packet
#[derive(RlpDecodable, RlpEncodable)]
pub struct PacketAck {
    pub ack: Vec<u8>,
    pub packet: Packet,
}

impl Object for PacketAck {
    fn encode(&self) -> Vec<u8> {
        rlp::encode(self).to_vec()
    }

    fn decode(data: &[u8]) -> Result<Self, VerifyError> {
        rlp::decode(data).map_err(|_| VerifyError::SerdeError)
    }
}

#[cfg(test)]
mod tests {
    use super::Ordering;
    use super::State;
    use super::Vec;

    #[test]
    fn encode_decode_state() {
        let mut states = Vec::new();
        states.push(State::Unknown);
        states.push(State::Init);
        states.push(State::OpenTry);
        states.push(State::Open);
        states.push(State::Closed);
        states.push(State::Frozen);

        for i in 1..states.len() {
            let state = states[i - 1];
            let data = rlp::encode(&state).to_vec();
            let actual = rlp::decode::<State>(&data).unwrap();
            assert_eq!(actual, states[i - 1]);
        }
    }

    #[test]
    fn encode_decode_ordering() {
        let mut orderings = Vec::new();

        orderings.push(Ordering::Unknown);
        orderings.push(Ordering::Unordered);
        orderings.push(Ordering::Ordered);

        for i in 1..orderings.len() {
            let ordering = orderings[i - 1];
            let data = rlp::encode(&ordering).to_vec();
            let actual = rlp::decode::<Ordering>(&data).unwrap();
            assert_eq!(actual, orderings[i - 1]);
        }
    }
}

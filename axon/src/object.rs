use crate::consts::COMMITMENT_PREFIX;
use crate::convert_byte32_to_hex;
use crate::get_channel_id_str;
use crate::proof::ObjectProof;
use crate::Bytes;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use super::U256;

// ChannelEnd, ConnectionEnd
pub trait Object: Sized {
    fn encode(&self) -> Vec<u8>;

    fn decode(_: &[u8]) -> Result<Self, VerifyError>;
}

#[derive(Debug)]
#[repr(i8)]
pub enum VerifyError {
    FoundNoMessage = 100,
    EventNotMatch,
    InvalidReceiptProof,
    SerdeError,

    WrongClient,
    WrongConnectionId,
    WrongConnectionnNumber,
    WrongPortId,
    WrongCommonHexId,

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
    WrongChannelSequence,

    WrongUnusedPacket,
    WrongPacketSequence,
    WrongPacketStatus,
    WrongPacketContent,
    WrongPacketArgs,
}

impl From<VerifyError> for i8 {
    fn from(value: VerifyError) -> Self {
        value as i8
    }
}

impl_enum_rlp!(
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
    },
    u8
);

impl_enum_rlp!(
    #[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
    #[repr(u8)]
    pub enum Ordering {
        #[default]
        Unknown = 1,
        Unordered,
        Ordered,
    },
    u8
);

#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct ConnectionCounterparty {
    pub client_id: String,
    pub connection_id: Option<String>,
    pub commitment_prefix: Bytes,
}

impl Default for ConnectionCounterparty {
    fn default() -> Self {
        Self {
            client_id: Default::default(),
            connection_id: Default::default(),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        }
    }
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

#[derive(Clone, PartialEq, Eq, RlpEncodable, RlpDecodable, Debug)]
pub struct Packet {
    pub sequence: u16,
    pub source_port_id: String,
    pub source_channel_id: String,
    pub destination_port_id: String,
    pub destination_channel_id: String,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            sequence: Default::default(),
            source_port_id: convert_byte32_to_hex(&[0u8; 32]),
            source_channel_id: get_channel_id_str(0),
            destination_port_id: convert_byte32_to_hex(&[0u8; 32]),
            destination_channel_id: get_channel_id_str(0),
            data: Default::default(),
            timeout_height: 0,
            timeout_timestamp: 0,
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

impl Packet {
    pub fn equal_unless_sequence(&self, other: &Self) -> bool {
        (
            &self.source_port_id,
            &self.source_channel_id,
            &self.destination_port_id,
            &self.destination_channel_id,
            &self.data,
            self.timeout_height,
            self.timeout_timestamp,
        ) == (
            &other.source_port_id,
            &other.source_channel_id,
            &other.destination_port_id,
            &other.destination_channel_id,
            &other.data,
            other.timeout_height,
            other.timeout_timestamp,
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, RlpEncodable, RlpDecodable)]
pub struct Version {
    pub identifier: String,
    pub features: Vec<String>,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_owned(), "ORDER_UNORDERED".to_owned()],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, RlpEncodable, RlpDecodable)]
pub struct ConnectionEnd {
    pub state: State,
    pub client_id: String,
    pub counterparty: ConnectionCounterparty,
    pub delay_period: u64,
    pub versions: Vec<Version>,
}

impl Default for ConnectionEnd {
    fn default() -> Self {
        Self {
            state: Default::default(),
            client_id: convert_byte32_to_hex(&[0u8; 32]),
            counterparty: Default::default(),
            delay_period: Default::default(),
            versions: Default::default(),
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

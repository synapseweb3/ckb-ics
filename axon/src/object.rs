use crate::consts::COMMITMENT_PREFIX;
use crate::convert_byte32_to_hex;
use crate::get_channel_id_str;
use crate::proto;
use crate::Bytes;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

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
    WrongUnusedPacketOrder,
    WrongUnusedPacketUnorder,
    WrongPacketSequence,
    WrongPacketStatus,
    WrongPacketContent,
    WrongPacketArgs,
    WrongPacketAck,

    Mpt = 99,
}

impl From<VerifyError> for i8 {
    fn from(value: VerifyError) -> Self {
        value as i8
    }
}

impl From<rlp::DecoderError> for VerifyError {
    fn from(_value: rlp::DecoderError) -> Self {
        Self::Mpt
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

impl From<Ordering> for proto::channel::Order {
    fn from(value: Ordering) -> Self {
        match value {
            Ordering::Ordered => Self::Ordered,
            Ordering::Unknown => Self::NoneUnspecified,
            Ordering::Unordered => Self::Unordered,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct ConnectionCounterparty {
    pub client_id: String,
    pub connection_id: String,
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
    pub connection_id: String,
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

#[derive(Debug, PartialEq, Eq, Clone, RlpEncodable, RlpDecodable)]
pub struct Version {
    pub identifier: String,
    pub features: Vec<String>,
}

impl Version {
    pub fn version_1() -> Self {
        Version {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_owned(), "ORDER_UNORDERED".to_owned()],
        }
    }
}

impl From<Version> for proto::connection::Version {
    fn from(value: Version) -> Self {
        Self {
            features: value.features,
            identifier: value.identifier,
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
            versions: vec![Version::version_1()],
        }
    }
}

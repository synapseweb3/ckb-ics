use crate::consts::COMMITMENT_PREFIX;
use crate::proto;
use crate::Bytes;
use crate::ChannelArgs;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

#[derive(Debug)]
#[repr(i8)]
pub enum VerifyError {
    FoundNoMessage = 70,
    EventNotMatch,
    InvalidReceiptProof,
    SerdeError,

    WrongClient,
    WrongConnectionId,
    WrongConnectionnNumber,
    WrongPortId,
    WrongCommonHexId,
    WrongIBCHandlerAddress,

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

    Commitment,
    Mpt,
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
    },
    u8
);

impl State {
    pub fn proto_connection_state(self) -> proto::connection::State {
        match self {
            State::Init => proto::connection::State::Init,
            State::OpenTry => proto::connection::State::Tryopen,
            State::Open => proto::connection::State::Open,
            _ => proto::connection::State::UninitializedUnspecified,
        }
    }

    pub fn proto_channel_state(self) -> proto::channel::State {
        match self {
            State::Init => proto::channel::State::Init,
            State::OpenTry => proto::channel::State::Tryopen,
            State::Open => proto::channel::State::Open,
            State::Closed => proto::channel::State::Closed,
            _ => proto::channel::State::UninitializedUnspecified,
        }
    }
}

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
    pub sequence: u64,
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
            source_port_id: ChannelArgs::default().port_id_str(),
            source_channel_id: ChannelArgs::default().channel_id_str(),
            destination_port_id: ChannelArgs::default().port_id_str(),
            destination_channel_id: ChannelArgs::default().channel_id_str(),
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
    pub counterparty: ConnectionCounterparty,
    pub delay_period: u64,
    pub versions: Vec<Version>,
}

impl Default for ConnectionEnd {
    fn default() -> Self {
        Self {
            state: Default::default(),
            counterparty: Default::default(),
            delay_period: Default::default(),
            versions: vec![Version::version_1()],
        }
    }
}

impl ConnectionEnd {
    pub fn to_proto(self, client_id: String) -> proto::connection::ConnectionEnd {
        proto::connection::ConnectionEnd {
            state: self.state.proto_connection_state() as i32,
            counterparty: Some(proto::connection::Counterparty {
                client_id: self.counterparty.client_id,
                connection_id: self.counterparty.connection_id,
                prefix: Some(proto::commitment::MerklePrefix {
                    key_prefix: self.counterparty.commitment_prefix,
                }),
            }),
            client_id,
            versions: self.versions.into_iter().map(|v| v.into()).collect(),
            delay_period: self.delay_period,
        }
    }
}

use axon_protocol::types::{Bytes, U256};

pub trait Message {
    fn encode(&self) -> Bytes;

    fn as_key(&self) -> &[u8];
}

pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    InvalidTxProof,
    TxReceiptNotMatch,
}

pub enum ClientType {
    Unknown,
    Tendermint,
    Ethereum,
    Axon,
    Ckb,
}

pub enum State {
    Unknown,
    Init,
    OpenTry,
    Open,
    Closed,
}

pub enum Ordering {
    Unknown,
    Unordered,
    Ordered,
}

pub struct ConnectionId {
    pub client_id: String,
    pub connection_id: String,
    pub commitment_prefix: Bytes,
}

pub struct ChannelId {
    pub port_id: String,
    pub channel_id: String,
}

pub struct Proofs {
    pub height: U256,
    pub object_proof: Bytes,
    pub client_proof: Bytes,
    pub consensus_proof: Bytes,
    pub other_proof: Bytes,
}

pub struct Packet {
    pub sequence: U256,
    pub source_port_id: String,
    pub source_channel_id: String,
    pub destination_port_id: String,
    pub destination_channel_id: String,
    pub payload: Bytes,
    pub timeout_height: Bytes, // bytes32
    pub timeout_timestamp: U256,
}

pub struct ClientState {
    pub chain_id: String,
    pub client_type: ClientType,
    pub latest_height: Bytes,
}

pub struct ConsensusState {
    pub timestamp: U256,
    pub commitment_root: Bytes, // bytes32
    pub extra_payload: Bytes,
}

pub struct ConnectionEnd {
    pub connection_id: ConnectionId,
    pub state: State,
    pub client_id: String,
    pub counterparty: ConnectionId,
    // pub versions: Vec<String>,
    pub delay_period: U256,
}

pub struct ChannelEnd {
    pub channel_id: ChannelId,
    pub state: State,
    pub ordering: Ordering,
    pub remote: ChannelId,
    // pub connection_hops: Vec<String>,
    // pub version: String,
}

pub struct MsgClientCreate {
    pub client: ClientState,
    pub consensus_state: ConsensusState,
}

pub struct MsgClientUpdate {
    pub client_id: String,
    pub header_bytes: Bytes,
}

pub struct MsgClientMisbehaviour {
    pub client_id: String,
    pub header1_bytes: Bytes,
    pub header2_bytes: Bytes,
}

pub struct MsgConnectionOpenInit {
    pub client_id: String,
    pub counterparty: ConnectionId,
    pub version: String,
    pub delay_duration: U256,
}

pub struct MsgConnectionOpenTry {
    pub previous_connection_id: String,
    pub client_id: String,
    pub client_state: ClientState,
    pub counterparty: ConnectionId,
    pub counterparty_versions: Vec<String>,
    pub delay_period: U256,
}

pub struct MsgConnectionOpenAck {
    pub connection_id: String,
    pub counterparty_connection_id: String,
    pub client_state: ClientState,
    pub proofs: Proofs,
    pub version: String,
}

pub struct MsgConnectionOpenConfirm {
    pub connection_id: String,
    pub proofs: Proofs,
}

pub struct MsgChannelOpenInit {
    pub port_id: String,
    pub channel: ChannelEnd,
}

pub struct MsgChannelOpenTry {
    pub port_id: String,
    pub previous_channel_id: ChannelId,
    pub channel: ChannelEnd,
    pub counterparty_version: String,
    pub proofs: Proofs,
}

pub struct MsgChannelOpenAck {
    pub port_id: String,
    pub previous_channel_id: ChannelId,
    pub channel: ChannelEnd,
    pub counterparty_version: String,
    pub proofs: Proofs,
}

pub struct MsgChannelOpenConfirm {
    pub port_id: String,
    pub channel_id: ChannelId,
    pub proofs: Proofs,
}

pub struct MsgChannelCloseInit {
    pub port_id: String,
    pub channel_id: ChannelId,
}

pub struct MsgChannelCloseConfirm {
    pub port_id: String,
    pub channel_id: ChannelId,
    pub proofs: Proofs,
}

pub struct MsgRecvPacket {
    pub packet: Packet,
    pub proofs: Proofs,
}

pub struct MsgAckPacket {
    pub packet: Packet,
    pub acknowledgement: Bytes,
    pub proofs: Proofs,
}

pub struct MsgTimeoutPacket {
    pub packet: Packet,
    pub next_sequence_recv: U256,
    pub proofs: Proofs,
}

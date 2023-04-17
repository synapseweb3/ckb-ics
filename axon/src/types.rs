use axon_protocol::types::{Bytes, U256};

pub trait IcsComponent {
    fn encode(&self) -> Bytes;

    fn as_key(&self) -> &[u8];
}

pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    InvalidTxProof,
    TxReceiptNotMatch,
    SerdeError,
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
    // pub commitment_prefix: Bytes,
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

impl IcsComponent for ClientState {
    fn encode(&self) -> Bytes {
        todo!()
    }

    fn as_key(&self) -> &[u8] {
        todo!()
    }
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

impl IcsComponent for ConnectionEnd {
    fn encode(&self) -> Bytes {
        todo!()
    }

    fn as_key(&self) -> &[u8] {
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

impl IcsComponent for ChannelEnd {
    fn encode(&self) -> Bytes {
        todo!()
    }

    fn as_key(&self) -> &[u8] {
        todo!()
    }
}

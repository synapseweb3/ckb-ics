use axon_tools::keccak_256;
use ethereum_types::H256;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use super::object::*;
use super::Vec;
use super::U256;
use crate::proto::client::Height;
use crate::WriteOrVerifyCommitments;

#[derive(RlpDecodable, RlpEncodable)]
pub struct Envelope {
    pub msg_type: MsgType,
    pub commitments: Vec<CommitmentKV>,
    pub content: Vec<u8>,
}

// Verify.
impl WriteOrVerifyCommitments for &[CommitmentKV] {
    fn write_commitments<K, V>(
        &mut self,
        kvs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<(), VerifyError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let mut expected: Vec<CommitmentKV> = Vec::new();
        expected.write_commitments(kvs)?;
        if **self == expected {
            Ok(())
        } else {
            Err(VerifyError::Commitment)
        }
    }
}

// Write.
impl WriteOrVerifyCommitments for Vec<CommitmentKV> {
    fn write_commitments<K, V>(
        &mut self,
        kvs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<(), VerifyError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        *self = kvs
            .into_iter()
            .map(|(k, v)| CommitmentKV::hash(k, v))
            .collect();
        Ok(())
    }
}

impl_enum_rlp!(
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
        MsgWriteAckPacket,
        MsgAckPacket,
        MsgTimeoutPacket,

        MsgConsumeAckPacket,
    },
    u8
);

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
    pub proof_height: Height,
    pub proof_init: Vec<u8>,
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
    pub proof_height: Height,
    pub proof_try: Vec<u8>,
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
    pub proof_height: Height,
    pub proof_ack: Vec<u8>,
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
    /* CKB's channel doesn't have this field */
    // pub connection_hops_on_b: Vec<CString>,
    // pub port_id_on_a: CString,
    // pub chain_id_on_a: CString,
    pub proof_height: Height,
    pub proof_init: Vec<u8>,
    // pub ordering: Ordering,
    // pub connection_hops_on_a: Vec<String>,
    // pub previous_channal_id: CString,
    // pub version_proposal: Version,
}

/// Per our convention, this message is sent to chain A.
#[derive(RlpEncodable, RlpDecodable)]
pub struct MsgChannelOpenAck {
    /* In CKB tx, these 2 fields are found in cell dep and witness. */
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
    pub proof_height: Height,
    pub proof_try: Vec<u8>,
    // pub chain_id_on_b: CString,
    // pub connection_hops_on_b: Vec<String>,
}

/// Per our convention, this message is sent to chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelOpenConfirm {
    // pub port_id_on_b: CString,
    // pub chain_id_on_b: CString,
    // pub channel_id: ChannelCounterparty,
    pub proof_height: Height,
    pub proof_ack: Vec<u8>,
    // pub connection_hops_on_b: Vec<String>,
}

// Per our convention, this message is sent to chain A.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelCloseInit {
    /* In CKB tx, these 2 fields are found in witness. */
    // pub port_id_on_a: CString,
    // pub chan_id_on_a: CString,
}

// Per our convention, this message is sent to chain B.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgChannelCloseConfirm {
    /* In CKB tx, these 2 fields are found in witness. */
    // pub port_id_on_b: CString,
    // pub port_id_on_b: CString,
    pub proof_height: Height,
    pub proof_init: Vec<u8>,
}

// As our CKB convention, the content of the packet is stored in Witness.
// We don't need to place it again in this message.
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgSendPacket {}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgRecvPacket {
    pub proof_height: Height,
    pub proof_commitment: Vec<u8>,
}

#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgAckPacket {
    pub proof_height: Height,
    pub proof_acked: Vec<u8>,
}

// Business side sends this message after handling MsgRecvPacket
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgWriteAckPacket {}

// If timeout block_number is set in Packet and reached, using MsgTimeoutPacket instead
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgTimeoutPacket {
    pub packet: Packet,
    pub next_sequence_recv: U256,
    pub proof_height: Height,
    pub proof_unreceived: Vec<u8>,
}

// It's additional msg type which isn't contained in IBC, and just used
// in Business side to be consumed to obtain its capacity
#[derive(RlpDecodable, RlpEncodable)]
pub struct MsgConsumeAckPacket {}

#[derive(RlpDecodable, RlpEncodable, PartialEq, Eq, Default)]
pub struct CommitmentKV(pub H256, pub H256);

impl CommitmentKV {
    pub fn hash(path: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        Self(
            keccak_256(path.as_ref()).into(),
            keccak_256(value.as_ref()).into(),
        )
    }
}

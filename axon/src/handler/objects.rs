use alloc::{string::String, vec::Vec};
use ethereum_types::H256;
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use crate::convert_byte32_to_hex;
use crate::object::{ChannelCounterparty, ConnectionEnd, Ordering, Packet, State, VerifyError};
use crate::proto::client::Height;

#[derive(Debug, Default, Clone, RlpDecodable, RlpEncodable, PartialEq, Eq)]
pub struct IbcConnections {
    pub next_channel_number: u16,
    pub connections: Vec<ConnectionEnd>,
}

#[derive(Debug, Clone, RlpDecodable, RlpEncodable, PartialEq, Eq)]
pub struct IbcChannel {
    pub number: u16,
    pub port_id: String,
    pub state: State,
    pub order: Ordering,
    // FIXME: due to the limit of CKB cell-model, there's limiation that one channel should pair
    //        only one port to fit the sequence field's uniqueness, otherwise we have to introdue
    //        a port cell to maintain this sequence field (refactor require)
    pub sequence: Sequence,
    pub counterparty: ChannelCounterparty,
    pub connection_hops: Vec<String>,
    pub version: String,
}

impl Default for IbcChannel {
    fn default() -> Self {
        Self {
            number: Default::default(),
            port_id: convert_byte32_to_hex(&[0u8; 32]),
            state: Default::default(),
            order: Default::default(),
            sequence: Default::default(),
            counterparty: Default::default(),
            connection_hops: Default::default(),
            version: Default::default(),
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Debug, Clone, PartialEq, Eq)]
pub struct IbcPacket {
    pub packet: Packet,
    pub tx_hash: Option<H256>,
    pub status: PacketStatus,
    pub ack: Option<Vec<u8>>,
}

impl_enum_rlp! {
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    #[repr(u8)]
    pub enum PacketStatus {
        Send = 1,
        Recv,
        WriteAck,
        Ack,
    },
    u8
}

#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Sequence {
    pub next_sequence_sends: u16,
    pub next_sequence_recvs: u16,
    pub next_sequence_acks: u16,
    /// Received sequences for unordered channel. Must be ordered.
    pub received_sequences: Vec<u16>,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            next_sequence_sends: 1,
            next_sequence_recvs: 1,
            next_sequence_acks: 1,
            received_sequences: vec![],
        }
    }
}

impl Sequence {
    pub fn unorder_receive(&mut self, seq: u16) -> Result<(), VerifyError> {
        match self.received_sequences.binary_search(&seq) {
            Ok(_) => Err(VerifyError::WrongPacketSequence),
            Err(idx) => {
                self.received_sequences.insert(idx, seq);
                Ok(())
            }
        }
    }
}

pub trait Client {
    fn client_id(&self) -> &[u8; 32];

    fn verify_membership(
        &self,
        height: Height,
        // delay_time_period: u64,
        // delay_block_period: u64,
        proof: &[u8],
        // Assume prefix is always "ibc". This is true for axon and ckb.
        // prefix: &[u8],
        path: &[u8],
        value: &[u8],
    ) -> Result<(), VerifyError>;
}

#[cfg(test)]
mod tests {
    use super::Sequence;

    #[test]
    fn test_unorder_receive() {
        let mut s = Sequence::default();
        s.unorder_receive(3).unwrap();
        s.unorder_receive(5).unwrap();
        s.unorder_receive(1).unwrap();
        s.unorder_receive(2).unwrap();
        assert!(s.unorder_receive(3).is_err());
        assert_eq!(s.received_sequences, [1, 2, 3, 5]);
    }
}

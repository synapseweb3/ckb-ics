use alloc::{string::String, vec::Vec};

use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use crate::connection_id;
use crate::object::{ChannelCounterparty, ConnectionEnd, Ordering, Packet, State, VerifyError};
use crate::proto;
use crate::proto::client::Height;
use crate::ChannelArgs;

#[derive(Debug, Default, Clone, RlpDecodable, RlpEncodable, PartialEq, Eq)]
pub struct IbcConnections {
    pub next_channel_number: u64,
    pub connections: Vec<ConnectionEnd>,
}

impl IbcConnections {
    pub fn get_by_id(&self, client_id: &str, id: &str) -> Option<&ConnectionEnd> {
        let idx = extract_connection_index(id).ok()?;
        let expected_id = connection_id(client_id, idx);
        if id != expected_id {
            return None;
        }
        self.connections.get(idx)
    }
}

fn extract_connection_index(connection_id: &str) -> Result<usize, VerifyError> {
    let index_str = connection_id
        .split('-')
        .last()
        .ok_or(VerifyError::WrongConnectionId)?;
    let index = index_str
        .parse()
        .map_err(|_| VerifyError::WrongConnectionId)?;
    Ok(index)
}

#[derive(Debug, Clone, RlpDecodable, RlpEncodable, PartialEq, Eq)]
pub struct IbcChannel {
    pub number: u64,
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
            port_id: ChannelArgs::default().port_id_str(),
            state: Default::default(),
            order: Default::default(),
            sequence: Default::default(),
            counterparty: Default::default(),
            connection_hops: Default::default(),
            version: Default::default(),
        }
    }
}

impl From<IbcChannel> for proto::channel::Channel {
    fn from(value: IbcChannel) -> Self {
        Self {
            connection_hops: value.connection_hops,
            counterparty: Some(proto::channel::Counterparty {
                channel_id: value.counterparty.channel_id,
                port_id: value.counterparty.port_id,
            }),
            version: value.version,
            state: value.state.proto_channel_state() as i32,
            ordering: proto::channel::Order::from(value.order) as i32,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Debug, Clone, PartialEq, Eq)]
pub struct IbcPacket {
    pub packet: Packet,
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
    pub next_sequence_sends: u64,
    pub next_sequence_recvs: u64,
    pub next_sequence_acks: u64,
    /// Received sequences for unordered channel. Must be ordered.
    pub received_sequences: Vec<u64>,
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
    pub fn unorder_receive(&mut self, seq: u64) -> Result<(), VerifyError> {
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

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use ethereum_types::H256;
use rlp::{Decodable, Encodable};
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use crate::convert_byte32_to_hex;
use crate::object::{
    ChannelCounterparty, ConnectionEnd, Object, Ordering, Packet, State, VerifyError,
};
use crate::proof::ObjectProof;

#[derive(Debug, Default, Clone, RlpDecodable, RlpEncodable)]
pub struct IbcConnections {
    // TODO: can this be removed?
    pub next_connection_number: u16,
    pub next_channel_number: u16,
    pub connections: Vec<ConnectionEnd>,
}

#[derive(Debug, Clone, RlpDecodable, RlpEncodable)]
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
        }
    }
}

impl IbcChannel {
    pub fn equal_unless_state_and_counterparty(&self, other: &Self) -> bool {
        (self.number, &self.port_id, self.order, &self.sequence)
            == (other.number, &other.port_id, other.order, &other.sequence)
    }

    pub fn equal_unless_sequence(&self, other: &Self) -> bool {
        (
            self.number,
            &self.port_id,
            self.order,
            self.state,
            &self.counterparty,
        ) == (
            other.number,
            &other.port_id,
            other.order,
            other.state,
            &other.counterparty,
        )
    }
}

#[derive(RlpEncodable, RlpDecodable)]
pub struct IbcPacket {
    pub packet: Packet,
    pub tx_hash: Option<H256>,
    pub status: PacketStatus,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum PacketStatus {
    Send = 1,
    Recv,
    WriteAck,
    Ack,
}

impl Encodable for PacketStatus {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let status = *self as u8;
        s.append(&status);
    }
}

impl Decodable for PacketStatus {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let status: u8 = rlp.val_at(0)?;
        match status {
            1 => Ok(PacketStatus::Send),
            2 => Ok(PacketStatus::Recv),
            3 => Ok(PacketStatus::Ack),
            4 => Ok(PacketStatus::WriteAck),
            _ => Err(rlp::DecoderError::Custom("invalid packet status")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Sequence {
    pub next_sequence_sends: u16,
    pub next_sequence_recvs: u16,
    pub next_sequence_acks: u16,
    pub received_sequences: Vec<u16>,
}

impl Sequence {
    pub fn next_send_packet_is(&self, new: &Self) -> bool {
        if self.next_sequence_sends + 1 != new.next_sequence_sends
            || self.next_sequence_recvs != new.next_sequence_recvs
            || self.next_sequence_acks != new.next_sequence_acks
        {
            return false;
        }

        let old_received = self.received_sequences.iter().collect::<BTreeSet<_>>();
        let new_received = new.received_sequences.iter().collect::<BTreeSet<_>>();

        if old_received.len() != self.received_sequences.len()
            || new_received.len() != new.received_sequences.len()
            || old_received != new_received
        {
            return false;
        }

        true
    }

    pub fn next_recv_packet_is(&self, new: &Self, unorder_sequence: Option<u16>) -> bool {
        if self.next_sequence_sends != new.next_sequence_sends
            || self.next_sequence_acks != new.next_sequence_acks
        {
            return false;
        }

        if let Some(sequence) = unorder_sequence {
            let old_received = self.received_sequences.iter().collect::<BTreeSet<_>>();
            let new_received = new.received_sequences.iter().collect::<BTreeSet<_>>();

            if old_received.len() != self.received_sequences.len()
                || new_received.len() != new.received_sequences.len()
                || new_received.len() != old_received.len() + 1
            {
                return false;
            }

            if old_received.contains(&sequence) || !new_received.contains(&sequence) {
                return false;
            }
        } else if self.next_sequence_recvs + 1 != new.next_sequence_recvs {
            return false;
        }

        true
    }

    pub fn next_recv_ack_is(&self, new: &Self, is_unorder: bool) -> bool {
        if self.next_sequence_sends != new.next_sequence_sends
            || self.next_sequence_recvs != new.next_sequence_recvs
        {
            return false;
        }

        if !is_unorder && self.next_sequence_acks + 1 != new.next_sequence_acks {
            return false;
        }

        let old_received = self.received_sequences.iter().collect::<BTreeSet<_>>();
        let new_received = new.received_sequences.iter().collect::<BTreeSet<_>>();

        if old_received.len() != self.received_sequences.len()
            || new_received.len() != new.received_sequences.len()
            || old_received != new_received
        {
            return false;
        }

        true
    }
}

pub trait Client {
    fn client_id(&self) -> &[u8; 32];

    fn verify_object<O: Object>(&mut self, obj: O, proof: ObjectProof) -> Result<(), VerifyError>;
}

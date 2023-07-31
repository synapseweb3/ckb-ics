// These structs should only be used in CKB contracts.
#![allow(clippy::too_many_arguments)]

use crate::consts::{CHANNEL_ID_PREFIX, COMMITMENT_PREFIX};
use crate::message::{
    Envelope, MsgAckInboxPacket, MsgAckOutboxPacket, MsgAckPacket, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck,
    MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry, MsgRecvPacket,
    MsgSendPacket, MsgType,
};
use crate::object::{
    ChannelCounterparty, ChannelEnd, ConnectionCounterparty, Object, Ordering, Packet, PacketAck,
    State, VerifyError,
};
use crate::proof::ObjectProof;
use crate::{convert_connection_id_to_index, convert_string_to_client_id};
use crate::{ChannelArgs, ConnectionArgs, PacketArgs};

// use axon_protocol::types::Bytes;
use super::Vec;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use ethereum_types::H256;
use rlp::{decode, Decodable, Encodable};
use rlp_derive::RlpDecodable;
use rlp_derive::RlpEncodable;

use super::object::ConnectionEnd;

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
    // TODO: can this be removed?
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
            port_id: String::from_utf8_lossy([0u8; 32].as_slice()).to_string(),
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
    Ack,
    InboxAck,
    OutboxAck,
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
            4 => Ok(PacketStatus::InboxAck),
            5 => Ok(PacketStatus::OutboxAck),
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
    fn client_id(&self) -> &[u8];

    fn verify_object<O: Object>(&mut self, obj: O, proof: ObjectProof) -> Result<(), VerifyError>;
}

pub fn handle_msg_connection_open_init<C: Client>(
    client: C,
    old_connections: IbcConnections,
    old_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_args: ConnectionArgs,
    _: MsgConnectionOpenInit,
) -> Result<(), VerifyError> {
    if old_connections.connections.len() + 1 != new_connections.connections.len()
        || old_connections.next_connection_number + 1 != new_connections.next_connection_number
    {
        return Err(VerifyError::WrongConnectionCnt);
    }

    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if convert_string_to_client_id(&connection.client_id)? != client.client_id() {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::Init {
        return Err(VerifyError::WrongConnectionState);
    }

    Ok(())
}

pub fn handle_msg_connection_open_try<C: Client>(
    mut client: C,
    old_connections: IbcConnections,
    old_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenTry,
) -> Result<(), VerifyError> {
    if old_connections.connections.len() + 1 != new_connections.connections.len()
        || old_connections.next_connection_number + 1 != new_connections.next_connection_number
    {
        return Err(VerifyError::WrongConnectionCnt);
    }

    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if convert_string_to_client_id(&connection.client_id)? != client.client_id() {
        return Err(VerifyError::WrongClient);
    }

    if connection.counterparty.connection_id.is_none() {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::OpenTry {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected_connection_end_on_counterparty = ConnectionEnd {
        state: State::Init,
        client_id: connection.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: String::from_utf8_lossy(client.client_id()).to_string(),
            connection_id: None,
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        delay_period: connection.delay_period,
        versions: vec![Default::default()],
    };

    let object_proof = msg.proof.object_proof;

    client.verify_object(expected_connection_end_on_counterparty, object_proof)
}

pub fn handle_msg_connection_open_ack<C: Client>(
    mut client: C,
    old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenAck,
) -> Result<(), VerifyError> {
    if old.connections.len() != new.connections.len() {
        return Err(VerifyError::WrongConnectionCnt);
    }

    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    let conn_idx = msg.conn_id_on_a;
    for i in 0..old.connections.len() {
        if i != conn_idx && old.connections[i] != new.connections[i] {
            return Err(VerifyError::WrongClient);
        }
    }

    let old_connection = &old.connections[conn_idx];
    let new_connection = &new.connections[conn_idx];

    if old_connection.client_id != new_connection.client_id
        || old_connection.delay_period != new_connection.delay_period
        || old_connection.counterparty.client_id != new_connection.counterparty.client_id
    {
        return Err(VerifyError::WrongClient);
    }

    if new_connection.counterparty.connection_id.is_none() {
        return Err(VerifyError::ConnectionsWrong);
    }

    if old_connection.state != State::Init || new_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected = ConnectionEnd {
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: String::from_utf8_lossy(client.client_id()).to_string(),
            connection_id: Some(conn_idx.to_string()),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        delay_period: new_connection.delay_period,
        versions: vec![Default::default()],
    };
    client.verify_object(expected, msg.proof_conn_end_on_b.object_proof)
}

pub fn handle_msg_connection_open_confirm<C: Client>(
    mut client: C,
    old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenConfirm,
) -> Result<(), VerifyError> {
    if old.connections.len() != new.connections.len() {
        return Err(VerifyError::WrongConnectionCnt);
    }

    let conn_idx = msg.conn_id_on_b;
    for i in 0..old.connections.len() {
        if i != conn_idx && old.connections[i] != new.connections[i] {
            return Err(VerifyError::WrongClient);
        }
    }

    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    let old_connection = &old.connections[conn_idx];
    let new_connection = &new.connections[conn_idx];

    if old_connection.client_id != new_connection.client_id
        || old_connection.delay_period != new_connection.delay_period
        || old_connection.counterparty != new_connection.counterparty
    {
        return Err(VerifyError::WrongClient);
    }
    if old_connection.state != State::OpenTry || new_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }
    let expected = ConnectionEnd {
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: String::from_utf8_lossy(client.client_id()).to_string(),
            connection_id: Some(conn_idx.to_string()),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        delay_period: new_connection.delay_period,
        versions: vec![Default::default()],
    };

    client.verify_object(expected, msg.proofs.object_proof)
}

pub fn handle_channel_open_init_and_try<C: Client>(
    client: C,
    channel: IbcChannel,
    channel_args: ChannelArgs,
    envelop: Envelope,
    old_connections: IbcConnections,
    old_connection_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_connection_args: ConnectionArgs,
) -> Result<(), VerifyError> {
    if old_connections.next_channel_number + 1 != new_connections.next_channel_number {
        return Err(VerifyError::WrongConnectionNextChannelNumber);
    }
    if old_connection_args != new_connection_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if old_connection_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if channel_args.client_id != old_connection_args.client_id
        || channel_args.open
        || channel_args.channel_id != channel.number
        || channel_args.port_id != channel.port_id.as_bytes()
    {
        return Err(VerifyError::WrongChannelArgs);
    }

    match envelop.msg_type {
        MsgType::MsgChannelOpenInit => {
            let init_msg = decode::<MsgChannelOpenInit>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_init(client, &new_connections, channel, init_msg)
        }
        MsgType::MsgChannelOpenTry => {
            let open_try_msg = decode::<MsgChannelOpenTry>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_try(client, &new_connections, channel, open_try_msg)
        }
        _ => Err(VerifyError::EventNotMatch),
    }
}

pub fn handle_msg_channel_open_init<C: Client>(
    client: C,
    ibc_connections: &IbcConnections,
    new: IbcChannel,
    _msg: MsgChannelOpenInit,
) -> Result<(), VerifyError> {
    if new.connection_hops.is_empty() {
        return Err(VerifyError::ConnectionsWrong);
    }
    let conn_id = convert_connection_id_to_index(&new.connection_hops[0])?;
    let conn = &ibc_connections.connections[conn_id];

    if convert_string_to_client_id(&conn.client_id)? != client.client_id() {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.state != State::Init {
        return Err(VerifyError::WrongChannelState);
    }

    Ok(())
}

pub fn handle_msg_channel_open_try<C: Client>(
    mut client: C,
    ibc_connections: &IbcConnections,
    new: IbcChannel,
    msg: MsgChannelOpenTry,
) -> Result<(), VerifyError> {
    if new.connection_hops.is_empty() {
        return Err(VerifyError::ConnectionsWrong);
    }
    let conn_id = convert_connection_id_to_index(&new.connection_hops[0])?;
    let conn = &ibc_connections.connections[conn_id];

    if convert_string_to_client_id(&conn.client_id)? != client.client_id() {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.state != State::OpenTry {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        state: State::Init,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.port_id,
            channel_id: new.number.to_string(),
        },
        connection_hops: Vec::new(),
    };

    client.verify_object(object, msg.proof_chan_end_on_a.object_proof)
}

pub fn handle_channel_open_ack_and_confirm<C: Client>(
    client: C,
    envelope: Envelope,
    old_channel: IbcChannel,
    old_args: ChannelArgs,
    new_channel: IbcChannel,
    new_args: ChannelArgs,
) -> Result<(), VerifyError> {
    if old_args.open
        || !new_args.open
        || old_args.client_id != new_args.client_id
        || old_args.channel_id != new_args.channel_id
        || old_args.port_id != new_args.port_id
    {
        return Err(VerifyError::WrongChannelArgs);
    }
    match envelope.msg_type {
        MsgType::MsgChannelOpenAck => {
            let msg = decode::<MsgChannelOpenAck>(&envelope.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_ack(client, old_channel, new_channel, msg)
        }
        MsgType::MsgChannelOpenConfirm => {
            let msg = decode::<MsgChannelOpenConfirm>(&envelope.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_confirm(client, old_channel, new_channel, msg)
        }
        _ => Err(VerifyError::EventNotMatch),
    }
}

pub fn handle_msg_channel_open_ack<C: Client>(
    mut client: C,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenAck,
) -> Result<(), VerifyError> {
    if !new.equal_unless_state_and_counterparty(&old) {
        return Err(VerifyError::WrongChannel);
    }

    if new.counterparty.channel_id.is_empty() || new.counterparty.port_id.is_empty() {
        return Err(VerifyError::WrongChannel);
    }

    if old.state != State::Init || new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        state: State::OpenTry,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.counterparty.port_id,
            channel_id: new.counterparty.channel_id,
        },
        connection_hops: Vec::new(),
    };

    client.verify_object(object, msg.proofs.object_proof)
}

pub fn handle_msg_channel_open_confirm<C: Client>(
    mut client: C,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenConfirm,
) -> Result<(), VerifyError> {
    if !new.equal_unless_state_and_counterparty(&old) {
        return Err(VerifyError::WrongChannel);
    }
    if old.state != State::OpenTry || new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        state: State::Open,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.counterparty.port_id,
            channel_id: new.counterparty.channel_id,
        },
        connection_hops: Vec::new(),
    };

    client.verify_object(object, msg.proofs.object_proof)
}

pub fn handle_msg_send_packet<C: Client>(
    _: C,
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    ibc_packet: IbcPacket,
    packet_args: PacketArgs,
    _: MsgSendPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if packet_args.port_id != ibc_packet.packet.source_port_id.as_bytes()
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.source_channel_id
    {
        return Err(VerifyError::WrongPacketArgs);
    }

    if !old_channel
        .sequence
        .next_send_packet_is(&new_channel.sequence)
    {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.status != PacketStatus::Send {
        return Err(VerifyError::WrongPacketStatus);
    }

    if ibc_packet.packet.sequence != old_channel.sequence.next_sequence_sends {
        return Err(VerifyError::WrongPacketSequence);
    }

    Ok(())
}

pub fn handle_msg_recv_packet<C: Client>(
    mut client: C,
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    ibc_packet: IbcPacket,
    packet_args: PacketArgs,
    msg: MsgRecvPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.status != PacketStatus::Recv {
        return Err(VerifyError::WrongPacketStatus);
    }

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if packet_args.port_id != ibc_packet.packet.source_port_id.as_bytes()
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.source_channel_id
    {
        return Err(VerifyError::WrongPacketArgs);
    }

    let unorder_sequence = if old_channel.order == Ordering::Unordered {
        Some(ibc_packet.packet.sequence)
    } else {
        None
    };
    if !old_channel
        .sequence
        .next_recv_packet_is(&new_channel.sequence, unorder_sequence)
    {
        return Err(VerifyError::WrongChannelSequence);
    }

    if ibc_packet.packet.sequence != new_channel.sequence.next_sequence_recvs {
        return Err(VerifyError::WrongPacketSequence);
    }

    client.verify_object(ibc_packet.packet, msg.proofs.object_proof)
}

pub fn handle_msg_ack_packet<C: Client>(
    mut client: C,
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    old_ibc_packet: IbcPacket,
    old_packet_args: PacketArgs,
    new_ibc_packet: IbcPacket,
    new_packet_args: PacketArgs,
    msg: MsgAckPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if old_ibc_packet.status != PacketStatus::Send && new_ibc_packet.status != PacketStatus::Ack {
        return Err(VerifyError::WrongPacketStatus);
    }

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if old_packet_args != new_packet_args {
        return Err(VerifyError::WrongPacketArgs);
    }

    let is_unorder = old_channel.order == Ordering::Unordered;
    if !old_channel
        .sequence
        .next_recv_ack_is(&new_channel.sequence, is_unorder)
    {
        return Err(VerifyError::WrongChannelSequence);
    }

    if new_ibc_packet.packet.sequence != new_channel.sequence.next_sequence_acks {
        return Err(VerifyError::WrongPacketSequence);
    }

    if old_ibc_packet.packet != new_ibc_packet.packet {
        return Err(VerifyError::WrongPacketContent);
    }

    let obj = PacketAck {
        ack: msg.acknowledgement,
        packet: new_ibc_packet.packet,
    };

    client.verify_object(obj, msg.proofs.object_proof)
}

pub fn handle_msg_ack_outbox_packet(
    old_ibc_packet: IbcPacket,
    old_packet_args: PacketArgs,
    new_ibc_packet: IbcPacket,
    new_packet_args: PacketArgs,
    _: MsgAckOutboxPacket,
) -> Result<(), VerifyError> {
    if old_ibc_packet.status != PacketStatus::Recv
        && new_ibc_packet.status != PacketStatus::OutboxAck
    {
        return Err(VerifyError::WrongPacketStatus);
    }
    if old_ibc_packet.packet != new_ibc_packet.packet {
        return Err(VerifyError::WrongPacketContent);
    }
    if old_packet_args != new_packet_args {
        return Err(VerifyError::WrongPacketArgs);
    }
    Ok(())
}

pub fn handle_msg_ack_inbox_packet(
    old_ibc_packet: IbcPacket,
    _: MsgAckInboxPacket,
) -> Result<(), VerifyError> {
    if old_ibc_packet.status != PacketStatus::Ack {
        return Err(VerifyError::WrongPacketStatus);
    }
    Ok(())
}

pub fn get_channel_id_str(idx: u16) -> String {
    let mut result = String::from(CHANNEL_ID_PREFIX);
    result.push_str(&idx.to_string());
    result
}

#[cfg(test)]
mod tests {

    use crate::{consts, convert_client_id_to_string, object::Proofs};

    use super::*;

    fn index_to_connection_id(index: usize) -> String {
        format!("{}{index}", consts::CONNECTION_ID_PREFIX)
    }

    #[derive(Debug, Default)]
    pub struct TestClient {
        client: [u8; 32],
    }

    impl Client for TestClient {
        fn verify_object<O: Object>(
            &mut self,
            _obj: O,
            _proof: ObjectProof,
        ) -> Result<(), VerifyError> {
            Ok(())
        }

        fn client_id(&self) -> &[u8] {
            self.client.as_slice()
        }
    }

    #[test]
    fn test_handle_msg_connection_open_init() {
        let client = TestClient::default();

        let old_connections = IbcConnections::default();
        let mut new_connections = IbcConnections::default();
        new_connections.connections.push(ConnectionEnd {
            state: State::Init,
            client_id: convert_client_id_to_string([0u8; 32]),
            ..Default::default()
        });
        new_connections.next_connection_number += 1;

        let old_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();
        let new_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();

        let msg = MsgConnectionOpenInit {};
        handle_msg_connection_open_init(
            client,
            old_connections,
            old_args,
            new_connections,
            new_args,
            msg,
        )
        .unwrap();
    }

    #[test]
    fn test_handle_msg_connection_open_try() {
        let client = TestClient::default();

        let old_connections = IbcConnections::default();
        let mut new_connections = IbcConnections::default();
        new_connections.connections.push(ConnectionEnd {
            state: State::OpenTry,
            client_id: convert_client_id_to_string([0u8; 32]),
            counterparty: ConnectionCounterparty {
                client_id: String::from("dummy"),
                connection_id: Some(String::from("dummy")),
                commitment_prefix: COMMITMENT_PREFIX.to_vec(),
            },
            ..Default::default()
        });
        new_connections.next_connection_number += 1;

        let msg = MsgConnectionOpenTry {
            proof: Proofs::default(),
        };
        let old_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();
        let new_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();

        handle_msg_connection_open_try(
            client,
            old_connections,
            old_args,
            new_connections,
            new_args,
            msg,
        )
        .unwrap();
    }

    #[test]
    fn test_handle_msg_connection_open_ack() {
        let client = TestClient::default();

        let msg = MsgConnectionOpenAck {
            conn_id_on_a: 1,
            proof_conn_end_on_b: Default::default(),
        };

        let dummy_connection_end = ConnectionEnd::default();

        let mut old_connection_end = ConnectionEnd::default();
        old_connection_end.state = State::Init;

        let mut new_connection_end = ConnectionEnd::default();
        new_connection_end.state = State::Open;
        new_connection_end.counterparty.connection_id = Some("connection".to_string());

        let mut old_connections = IbcConnections::default();
        old_connections
            .connections
            .push(dummy_connection_end.clone());
        old_connections.connections.push(old_connection_end);
        old_connections
            .connections
            .push(dummy_connection_end.clone());

        let mut new_connections = IbcConnections::default();
        new_connections
            .connections
            .push(dummy_connection_end.clone());
        new_connections.connections.push(new_connection_end);
        new_connections
            .connections
            .push(dummy_connection_end.clone());

        let old_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();
        let new_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();
        handle_msg_connection_open_ack(
            client,
            old_connections,
            old_args,
            new_connections,
            new_args,
            msg,
        )
        .unwrap();
    }

    #[test]
    fn test_handle_msg_connection_open_confirm() {
        let client = TestClient::default();

        let msg = MsgConnectionOpenConfirm {
            conn_id_on_b: 1,
            proofs: Proofs::default(),
        };

        let dummy_connection_end = ConnectionEnd::default();

        let mut old_connection_end = ConnectionEnd::default();
        old_connection_end.state = State::OpenTry;

        let mut new_connection_end = ConnectionEnd::default();
        new_connection_end.state = State::Open;

        let mut old_connections = IbcConnections::default();
        old_connections
            .connections
            .push(dummy_connection_end.clone());
        old_connections.connections.push(old_connection_end);
        old_connections
            .connections
            .push(dummy_connection_end.clone());

        let mut new_connections = IbcConnections::default();
        new_connections
            .connections
            .push(dummy_connection_end.clone());
        new_connections.connections.push(new_connection_end);
        new_connections
            .connections
            .push(dummy_connection_end.clone());
        let old_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();
        let new_args = ConnectionArgs::from_slice(&[0u8; 32]).unwrap();

        handle_msg_connection_open_confirm(
            client,
            old_connections,
            old_args,
            new_connections,
            new_args,
            msg,
        )
        .unwrap();
    }

    #[test]
    fn test_handle_msg_channel_open_init() {
        let client = TestClient::default();

        let mut new_connections = IbcConnections::default();
        new_connections.next_channel_number += 1;

        let mut connection_end = ConnectionEnd::default();
        connection_end.state = State::Open;
        new_connections.connections.push(connection_end);

        let mut channel = IbcChannel::default();
        channel.state = State::Init;
        channel.connection_hops.push(index_to_connection_id(0));

        let msg = MsgChannelOpenInit {};
        handle_msg_channel_open_init(client, &new_connections, channel, msg).unwrap();
    }

    #[test]
    fn test_handle_msg_channel_open_try_success() {
        let client = TestClient::default();

        let mut new_connections = IbcConnections::default();
        new_connections.next_channel_number += 1;

        let mut connection_end = ConnectionEnd::default();
        connection_end.state = State::Open;
        new_connections.connections.push(connection_end);

        let mut channel = IbcChannel::default();
        channel.connection_hops.push(index_to_connection_id(0));
        channel.state = State::OpenTry;

        let msg = MsgChannelOpenTry {
            proof_chan_end_on_a: Proofs::default(),
        };
        handle_msg_channel_open_try(client, &new_connections, channel, msg).unwrap()
    }

    #[test]
    fn test_handle_msg_channel_open_ack_success() {
        let client = TestClient::default();
        let mut old_channel = IbcChannel::default();
        old_channel.state = State::Init;
        old_channel.counterparty.port_id = "portid".to_string();

        let mut new_channel = IbcChannel::default();
        new_channel.state = State::Open;
        new_channel.counterparty.channel_id = "channel-id".to_string();
        new_channel.counterparty.port_id = "portid".to_string();

        let msg = MsgChannelOpenAck {
            proofs: Proofs::default(),
        };

        handle_msg_channel_open_ack(client, old_channel, new_channel, msg).unwrap();
    }

    #[test]
    fn test_handle_msg_channel_open_ack_failed() {
        let client = TestClient::default();
        let old_channel = IbcChannel {
            number: 0,
            port_id: String::from(
                "b6ac779881b4fe05a167e413ff534469b6b5f6c06d95e4c523eb2945d85ed450",
            ),
            state: State::Init,
            order: Ordering::Unordered,
            sequence: Sequence::default(),
            counterparty: ChannelCounterparty {
                port_id: String::from(
                    "54d043fc84623f7a9f7383e1a332c524f0def68608446fc420316c30dfc00f01",
                ),
                channel_id: String::from(""),
            },
            connection_hops: vec![index_to_connection_id(0)],
        };
        let new_channel = IbcChannel {
            number: 0,
            port_id: String::from(
                "b6ac779881b4fe05a167e413ff534469b6b5f6c06d95e4c523eb2945d85ed450",
            ),
            state: State::Open,
            order: Ordering::Unordered,
            sequence: Sequence::default(),
            counterparty: ChannelCounterparty {
                port_id: String::from(
                    "54d043fc84623f7a9f7383e1a332c524f0def68608446fc420316c30dfc00f01",
                ),
                channel_id: String::from("channel-1"),
            },
            connection_hops: vec![index_to_connection_id(0)],
        };

        let old_args = ChannelArgs {
            client_id: [
                59, 202, 83, 204, 94, 60, 251, 53, 29, 14, 91, 232, 113, 191, 94, 227, 72, 206, 76,
                254, 177, 59, 247, 13, 54, 105, 235, 22, 75, 21, 45, 12,
            ],
            open: false,
            channel_id: 0,
            port_id: [
                182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182,
                181, 246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
            ],
        };

        let new_args = ChannelArgs {
            client_id: [
                59, 202, 83, 204, 94, 60, 251, 53, 29, 14, 91, 232, 113, 191, 94, 227, 72, 206, 76,
                254, 177, 59, 247, 13, 54, 105, 235, 22, 75, 21, 45, 12,
            ],
            open: true,
            channel_id: 0,
            port_id: [
                182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182,
                181, 246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
            ],
        };

        let envelope = Envelope {
            msg_type: MsgType::MsgChannelOpenAck,
            content: rlp::encode(&MsgChannelOpenAck {
                proofs: Proofs::default(),
            })
            .to_vec(),
        };
        handle_channel_open_ack_and_confirm(
            client,
            envelope,
            old_channel,
            old_args,
            new_channel,
            new_args,
        )
        .unwrap();
    }

    #[test]
    fn handle_msg_channel_open_confirm_success() {
        let client = TestClient::default();
        let mut old_channel = IbcChannel::default();
        old_channel.state = State::OpenTry;

        let mut new_channel = IbcChannel::default();
        new_channel.state = State::Open;

        let msg = MsgChannelOpenConfirm {
            proofs: Proofs::default(),
        };

        handle_msg_channel_open_confirm(client, old_channel, new_channel, msg).unwrap();
    }

    #[test]
    fn handle_msg_channel_open_confirm_channel_unmatch() {
        let client = TestClient::default();
        let mut old_channel = IbcChannel::default();
        old_channel.state = State::OpenTry;

        let mut new_channel = IbcChannel::default();
        new_channel.state = State::Open;

        new_channel.order = Ordering::Ordered;

        let msg = MsgChannelOpenConfirm {
            proofs: Proofs::default(),
        };

        if let Err(VerifyError::WrongChannel) =
            handle_msg_channel_open_confirm(client, old_channel, new_channel, msg)
        {
        } else {
            panic!()
        }
    }

    #[test]
    fn test_handle_msg_send_packet_success() {
        let client = TestClient::default();

        let mut seq2 = Sequence::default();
        seq2.next_sequence_sends += 1;

        let mut old_channel = IbcChannel::default();
        old_channel.state = State::Open;

        let mut new_channel = IbcChannel::default();
        new_channel.sequence = seq2;
        new_channel.state = State::Open;
        let msg = MsgSendPacket {};

        let packet = Packet::default();

        let ibc_packet = IbcPacket {
            packet,
            tx_hash: None,
            status: PacketStatus::Send,
        };

        let old_channel_args = ChannelArgs::default();
        let new_channel_args = ChannelArgs::default();
        let packet_args = PacketArgs::default();

        handle_msg_send_packet(
            client,
            old_channel,
            old_channel_args,
            new_channel,
            new_channel_args,
            ibc_packet,
            packet_args,
            msg,
        )
        .unwrap();
    }

    #[test]
    fn test_msg_recv_packet_success() {
        let seq1 = Sequence::default();
        let mut seq2 = Sequence::default();
        seq2.next_sequence_recvs += 1;

        let mut old_channel = IbcChannel::default();
        old_channel.sequence = seq1;
        old_channel.state = State::Open;

        let mut new_channel = IbcChannel::default();
        new_channel.sequence = seq2;
        new_channel.state = State::Open;

        let mut packet = Packet::default();
        packet.sequence = 1;

        let ibc_packet = IbcPacket {
            packet,
            tx_hash: None,
            status: PacketStatus::Recv,
        };
        let old_channel_args = ChannelArgs::default();
        let new_channel_args = ChannelArgs::default();
        let mut packet_args = PacketArgs::default();
        packet_args.sequence += 1;

        handle_msg_recv_packet(
            TestClient::default(),
            old_channel,
            old_channel_args,
            new_channel,
            new_channel_args,
            ibc_packet,
            packet_args,
            MsgRecvPacket {
                proofs: Proofs::default(),
            },
        )
        .unwrap();
    }

    #[test]
    fn test_msg_ack_outbox_packet_success() {
        let packet = Packet::default();
        let old_ibc_packet = IbcPacket {
            packet: packet.clone(),
            tx_hash: None,
            status: PacketStatus::Recv,
        };
        let new_ibc_packet = IbcPacket {
            packet,
            tx_hash: None,
            status: PacketStatus::OutboxAck,
        };
        handle_msg_ack_outbox_packet(
            old_ibc_packet,
            PacketArgs::default(),
            new_ibc_packet,
            PacketArgs::default(),
            MsgAckOutboxPacket { ack: Vec::new() },
        )
        .unwrap();
    }

    #[test]
    fn test_msg_ack_outbox_packet_differenct_packet() {
        let old_packet = Packet::default();
        let mut new_packet = old_packet.clone();
        new_packet.sequence = 1;
        let old_ibc_packet = IbcPacket {
            packet: old_packet,
            tx_hash: None,
            status: PacketStatus::Recv,
        };
        let new_ibc_packet = IbcPacket {
            packet: new_packet,
            tx_hash: None,
            status: PacketStatus::OutboxAck,
        };
        if let Err(VerifyError::WrongPacketContent) = handle_msg_ack_outbox_packet(
            old_ibc_packet,
            PacketArgs::default(),
            new_ibc_packet,
            PacketArgs::default(),
            MsgAckOutboxPacket { ack: Vec::new() },
        ) {
        } else {
            panic!()
        }
    }

    #[test]
    fn test_ibc_connection_encode_and_decode() {
        let mut conn = IbcConnections::default();
        conn.connections.push(ConnectionEnd::default());
        let a = rlp::encode(&conn);
        rlp::decode::<IbcConnections>(&a).unwrap();
    }
}

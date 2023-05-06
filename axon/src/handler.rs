// These structs should only be used in CKB contracts.

use crate::message::Envelope;
use crate::message::MsgAckPacket;
use crate::message::MsgChannelOpenAck;
use crate::message::MsgChannelOpenConfirm;
use crate::message::MsgChannelOpenInit;
use crate::message::MsgChannelOpenTry;
use crate::message::MsgConnectionOpenAck;
use crate::message::MsgConnectionOpenConfirm;
use crate::message::MsgConnectionOpenInit;
use crate::message::MsgConnectionOpenTry;
use crate::message::MsgRecvPacket;
use crate::message::MsgSendPacket;
use crate::message::MsgType;
use crate::object::ChannelCounterparty;
use crate::object::ChannelEnd;
use crate::object::ConnectionCounterparty;
use crate::object::Ordering;
use crate::object::Packet;
use crate::object::PacketAck;
use crate::object::State;
use crate::object::VerifyError;
use crate::proof::Block;
use crate::verify_object;

// use axon_protocol::types::Bytes;
use super::Bytes;
use super::Vec;

use alloc::string::ToString;
use cstr_core::CString;
use ethereum_types::H256;
use rlp::{decode, Decodable, Encodable};

use super::object::ConnectionEnd;

pub struct IbcConnections {
    pub connection_prefix: Bytes,
    pub channel_prefix: Bytes,
    pub next_connection_number: u16,
    pub next_channel_number: u16,
    pub connections: Vec<ConnectionEnd>,
}

impl Encodable for IbcConnections {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for IbcConnections {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct IbcChannel {
    pub num: u16,
    pub port_id: CString,
    pub state: State,
    pub order: Ordering,
    pub sequence: Sequence,
    pub counterparty: ChannelCounterparty,
    pub connection_hops: Vec<usize>,
}

impl IbcChannel {
    pub fn equal_unless_state(&self, other: &Self) -> bool {
        if self.num != other.num
            || self.port_id != other.port_id
            || self.order != other.order
            || self.sequence != other.sequence
            || self.counterparty != other.counterparty
        {
            return false;
        }
        return true;
    }

    pub fn equal_unless_sequence(&self, other: &Self) -> bool {
        if self.num != other.num
            || self.port_id != other.port_id
            || self.order != other.order
            || self.state != other.state
            || self.counterparty != other.counterparty
        {
            return false;
        }
        return true;
    }
}

impl Encodable for IbcChannel {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for IbcChannel {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct IbcPacket {
    pub packet: Packet,
    pub tx_hash: H256,
    pub status: PacketStatus,
}

pub enum PacketStatus {}

impl Encodable for IbcPacket {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for IbcPacket {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

#[derive(PartialEq, Eq)]
pub struct Sequence {
    pub next_send_packet: u16,
    pub next_recv_packet: u16,
    pub next_recv_ack: u16,
    pub unorder_recv_packet: Vec<u16>,
    pub unorder_recv_ack: Vec<u16>,
}

impl Sequence {
    pub fn next_send_packet_is(&self, new: &Self) -> bool {
        if self.next_send_packet + 1 != new.next_send_packet {
            return false;
        }

        if self.next_recv_packet != new.next_recv_packet {
            return false;
        }

        if self.next_recv_ack != new.next_recv_ack {
            return false;
        }

        if self.unorder_recv_packet.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for i in 0..self.unorder_recv_packet.len() {
            if self.unorder_recv_packet[i] != new.unorder_recv_packet[i] {
                return false;
            }
        }

        if self.unorder_recv_ack.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for j in 0..self.unorder_recv_ack.len() {
            if self.unorder_recv_ack[j] != new.unorder_recv_ack[j] {
                return false;
            }
        }
        return true;
    }

    pub fn next_recv_packet_is(&self, new: &Self) -> bool {
        if self.next_send_packet != new.next_send_packet {
            return false;
        }

        if self.next_recv_packet + 1 != new.next_recv_packet {
            return false;
        }

        if self.next_recv_ack != new.next_recv_ack {
            return false;
        }

        if self.unorder_recv_packet.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for i in 0..self.unorder_recv_packet.len() {
            if self.unorder_recv_packet[i] != new.unorder_recv_packet[i] {
                return false;
            }
        }

        if self.unorder_recv_ack.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for j in 0..self.unorder_recv_ack.len() {
            if self.unorder_recv_ack[j] != new.unorder_recv_ack[j] {
                return false;
            }
        }
        return true;
    }

    pub fn next_recv_ack_is(&self, new: &Self) -> bool {
        if self.next_send_packet != new.next_send_packet {
            return false;
        }

        if self.next_recv_packet != new.next_recv_packet {
            return false;
        }

        if self.next_recv_ack + 1 != new.next_recv_ack {
            return false;
        }

        if self.unorder_recv_packet.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for i in 0..self.unorder_recv_packet.len() {
            if self.unorder_recv_packet[i] != new.unorder_recv_packet[i] {
                return false;
            }
        }

        if self.unorder_recv_ack.len() != new.unorder_recv_packet.len() {
            return false;
        }

        for j in 0..self.unorder_recv_ack.len() {
            if self.unorder_recv_ack[j] != new.unorder_recv_ack[j] {
                return false;
            }
        }
        return true;
    }
}

impl Encodable for Sequence {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for Sequence {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub struct Client {
    pub id: CString,
}

impl Client {
    pub fn verify_block(&self, block: Block) -> Result<(), VerifyError> {
        todo!()
    }
}

impl Encodable for Client {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        todo!()
    }
}

impl Decodable for Client {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        todo!()
    }
}

pub fn handle_msg_connection_open_init(
    client: Client,
    old_connections: IbcConnections,
    new_connections: IbcConnections,
    msg: MsgConnectionOpenInit,
) -> Result<(), VerifyError> {
    if old_connections.connections.len() + 1 != new_connections.connections.len()
        || old_connections.next_connection_number + 1 != new_connections.next_connection_number
    {
        return Err(VerifyError::WrongConnectionCnt);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if connection.client_id != client.id {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::Init {
        return Err(VerifyError::WrongConnectionState);
    }

    Ok(())
}

pub fn handle_msg_connection_open_try(
    client: Client,
    old_connections: IbcConnections,
    new_connections: IbcConnections,
    msg: MsgConnectionOpenTry,
) -> Result<(), VerifyError> {
    if old_connections.connections.len() + 1 != new_connections.connections.len()
        || old_connections.next_connection_number + 1 != new_connections.next_connection_number
    {
        return Err(VerifyError::WrongConnectionCnt);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if connection.client_id != connection.client_id {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::OpenTry {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected_connection_end_on_counterparty = ConnectionEnd {
        state: State::Init,
        client_id: msg.counterparty.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: client.id.clone(),
            connection_id: None,
        },
        delay_period: msg.delay_period.clone(),
    };

    let object_proof = msg.proof.object_proof;

    verify_object(
        client,
        expected_connection_end_on_counterparty,
        object_proof,
    )
}

pub fn handle_msg_connection_open_ack(
    client: Client,
    old: IbcConnections,
    new: IbcConnections,
    msg: MsgConnectionOpenAck,
) -> Result<(), VerifyError> {
    if old.connections.len() != new.connections.len() {
        return Err(VerifyError::WrongConnectionCnt);
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
        || old_connection.delay_period != old_connection.delay_period
        || old_connection.counterparty != new_connection.counterparty
    {
        return Err(VerifyError::WrongClient);
    }

    if old_connection.state != State::Init || new_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected = ConnectionEnd {
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: client.id.clone(),
            connection_id: Some(CString::new(conn_idx.to_string()).unwrap()),
        },
        delay_period: new_connection.delay_period.clone(),
    };
    verify_object(client, expected, msg.proof_conn_end_on_b.object_proof)
}

pub fn handle_msg_connection_open_confirm(
    client: Client,
    old: IbcConnections,
    new: IbcConnections,
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

    let old_connection = &old.connections[conn_idx];
    let new_connection = &new.connections[conn_idx];

    if old_connection.client_id != new_connection.client_id
        || old_connection.delay_period != old_connection.delay_period
        || old_connection.counterparty != new_connection.counterparty
    {
        return Err(VerifyError::WrongClient);
    }
    if old_connection.state != State::Init || old_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }
    let expected = ConnectionEnd {
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionCounterparty {
            client_id: client.id.clone(),
            connection_id: Some(CString::new(conn_idx.to_string()).unwrap()),
        },
        delay_period: new_connection.delay_period.clone(),
    };

    verify_object(client, expected, msg.proofs.object_proof)
}

pub fn handle_channel_open_init_and_try(
    client: Client,
    channel: IbcChannel,
    envelop: Envelope,
    old_connections: IbcConnections,
    new_connections: IbcConnections,
) -> Result<(), VerifyError> {
    let connection_idx = channel.connection_hops[0];
    if old_connections.next_channel_number + 1 != new_connections.next_channel_number {
        return Err(VerifyError::WrongConnectionNextChannelNumber);
    }
    let new_connection_end = &new_connections.connections[connection_idx];

    match envelop.msg_type {
        MsgType::MsgChannelOpenInit => {
            let init_msg = decode::<MsgChannelOpenInit>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_init(client, new_connection_end, channel, init_msg)
        }
        MsgType::MsgChannelOpenTry => {
            let open_try_msg = decode::<MsgChannelOpenTry>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_try(client, new_connection_end, channel, open_try_msg)
        }
        _ => Err(VerifyError::EventNotMatch),
    }
}

pub fn handle_msg_channel_open_init(
    client: Client,
    conn: &ConnectionEnd,
    new: IbcChannel,
    _msg: MsgChannelOpenInit,
) -> Result<(), VerifyError> {
    if conn.client_id != client.id {
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

pub fn handle_msg_channel_open_try(
    client: Client,
    conn: &ConnectionEnd,
    new: IbcChannel,
    msg: MsgChannelOpenTry,
) -> Result<(), VerifyError> {
    if conn.client_id != client.id {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    let object = ChannelEnd {
        state: State::Init,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
        connection_hops: msg.connection_hops_on_a,
    };

    verify_object(client, object, msg.proof_chan_end_on_a.object_proof)
}

pub fn handle_channel_open_ack_and_confirm(
    client: Client,
    envelope: Envelope,
    old_channel: IbcChannel,
    new_channel: IbcChannel,
) -> Result<(), VerifyError> {
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

pub fn handle_msg_channel_open_ack(
    client: Client,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenAck,
) -> Result<(), VerifyError> {
    if !new.equal_unless_state(&old) {
        return Err(VerifyError::WrongChannel);
    }

    if old.state != State::Init && new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        state: State::OpenTry,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
        connection_hops: msg.connection_hops_on_b,
    };

    verify_object(client, object, msg.proofs.object_proof)
}

pub fn handle_msg_channel_open_confirm(
    client: Client,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenConfirm,
) -> Result<(), VerifyError> {
    if !new.equal_unless_state(&old) {
        return Err(VerifyError::WrongChannel);
    }
    if old.state != State::OpenTry && new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        state: State::OpenTry,
        ordering: new.order,
        remote: ChannelCounterparty {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
        connection_hops: msg.connection_hops_on_b,
    };

    verify_object(client, object, msg.proofs.object_proof)
}

pub fn handle_msg_send_packet(
    _: Client,
    old_channel: IbcChannel,
    new_channel: IbcChannel,
    ibc_packet: IbcPacket,
    _: MsgSendPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if old_channel
        .sequence
        .next_send_packet_is(&new_channel.sequence)
    {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.packet.sequence != new_channel.sequence.next_send_packet {
        return Err(VerifyError::WrongPacketSequence);
    }

    return Ok(());
}

pub fn handle_msg_recv_packet(
    client: Client,
    old_channel: IbcChannel,
    new_channel: IbcChannel,
    ibc_packet: IbcPacket,
    msg: MsgRecvPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if !old_channel
        .sequence
        .next_recv_packet_is(&new_channel.sequence)
    {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.packet.sequence != new_channel.sequence.next_recv_packet {
        return Err(VerifyError::WrongPacketSequence);
    }
    verify_object(client, ibc_packet.packet, msg.proofs.object_proof)
}

pub fn handle_msg_ack_packet(
    client: Client,
    old_channel: IbcChannel,
    new_channel: IbcChannel,
    ibc_packet: IbcPacket,
    msg: MsgAckPacket,
) -> Result<(), VerifyError> {
    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if !old_channel.sequence.next_recv_ack_is(&new_channel.sequence) {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.packet.sequence != new_channel.sequence.next_recv_ack {
        return Err(VerifyError::WrongPacketSequence);
    }

    let obj = PacketAck {
        ack: msg.acknowledgement,
        packet: ibc_packet.packet,
    };

    verify_object(client, obj, msg.proofs.object_proof)
}

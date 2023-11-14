#![allow(clippy::too_many_arguments)]

use alloc::string::ToString;
use prost::Message;
use rlp::decode;

use crate::consts::COMMITMENT_PREFIX;
use crate::message::{
    Envelope, MsgAckPacket, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
    MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgConsumeAckPacket, MsgRecvPacket, MsgSendPacket, MsgType,
    MsgWriteAckPacket,
};
use crate::object::{Ordering, State, VerifyError};
use crate::proto::client::Height;
use crate::{commitment::*, proto};
use crate::{
    convert_byte32_to_hex, convert_connection_id_to_index, convert_hex_to_client_id,
    convert_hex_to_port_id, get_channel_id_str,
};
use crate::{ChannelArgs, ConnectionArgs, PacketArgs};

mod objects;
#[cfg(test)]
mod test;

pub use objects::*;

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

    if old_args != new_args || old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if &convert_hex_to_client_id(&connection.client_id)? != client.client_id() {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::Init {
        return Err(VerifyError::WrongConnectionState);
    }

    Ok(())
}

pub fn handle_msg_connection_open_try<C: Client>(
    client: C,
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

    if old_args != new_args || old_args.client_id.as_slice() != client.client_id() {
        return Err(VerifyError::WrongConnectionArgs);
    }

    for i in 0..old_connections.connections.len() {
        if old_connections.connections[i] != new_connections.connections[i] {
            return Err(VerifyError::ConnectionsWrong);
        }
    }

    let connection = new_connections.connections.last().unwrap();
    if &convert_hex_to_client_id(&connection.client_id)? != client.client_id()
        || connection.counterparty.connection_id.is_none()
    {
        return Err(VerifyError::WrongClient);
    }

    if connection.state != State::OpenTry {
        return Err(VerifyError::WrongConnectionState);
    }

    let counterparty = &connection.counterparty;

    let expected_connection_end_on_counterparty = proto::connection::ConnectionEnd {
        state: proto::connection::State::Init as _,
        client_id: counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            client_id: convert_byte32_to_hex(client.client_id()),
            connection_id: "".to_string(),
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: connection.delay_period,
        // XXX: should be msg_.counterpartyVersions
        versions: vec![Default::default()],
    };

    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_init,
        counterparty.connection_id.as_deref().unwrap(),
        &expected_connection_end_on_counterparty,
    )?;

    // TODO: verify client and consensus.
    Ok(())
}

fn verify_connection_state(
    client: &impl Client,
    proof_height: Height,
    proof: &[u8],
    connection_id: &str,
    connection: &proto::connection::ConnectionEnd,
) -> Result<(), VerifyError> {
    client.verify_membership(
        proof_height,
        proof,
        connection_path(connection_id).as_bytes(),
        &connection.encode_to_vec(),
    )
}

pub fn handle_msg_connection_open_ack<C: Client>(
    client: C,
    old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenAck,
) -> Result<(), VerifyError> {
    if old.connections.len() != new.connections.len() {
        return Err(VerifyError::WrongConnectionCnt);
    }

    if old_args != new_args || &old_args.client_id != client.client_id() {
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

    let expected = proto::connection::ConnectionEnd {
        state: proto::connection::State::Tryopen as _,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            client_id: convert_byte32_to_hex(client.client_id()),
            connection_id: format!("connection-{conn_idx}"),
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: new_connection.delay_period,
        // XXX
        versions: vec![Default::default()],
    };

    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_try,
        new_connection
            .counterparty
            .connection_id
            .as_deref()
            .unwrap(),
        &expected,
    )?;

    // TODO: verify client.
    Ok(())
}

pub fn handle_msg_connection_open_confirm<C: Client>(
    client: C,
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

    if old_args != new_args || &old_args.client_id != client.client_id() {
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
    let expected = proto::connection::ConnectionEnd {
        state: proto::connection::State::Open as _,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            client_id: convert_byte32_to_hex(client.client_id()),
            connection_id: format!("connection-{conn_idx}"),
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: new_connection.delay_period,
        versions: vec![Default::default()],
    };

    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_ack,
        new_connection
            .counterparty
            .connection_id
            .as_deref()
            .unwrap(),
        &expected,
    )?;

    // TODO: verify client.

    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    if old_connection_args != new_connection_args
        || &old_connection_args.client_id != client.client_id()
    {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if channel_args.client_id != old_connection_args.client_id
        || channel_args.open
        || channel_args.channel_id != channel.number
        || channel_args.port_id != convert_hex_to_port_id(&channel.port_id)?
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
    let conn = ibc_connections
        .connections
        .get(conn_id)
        .ok_or(VerifyError::WrongConnectionnNumber)?;

    if &convert_hex_to_client_id(&conn.client_id)? != client.client_id() {
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
    client: C,
    ibc_connections: &IbcConnections,
    new: IbcChannel,
    msg: MsgChannelOpenTry,
) -> Result<(), VerifyError> {
    if new.connection_hops.is_empty() {
        return Err(VerifyError::ConnectionsWrong);
    }
    let conn_id = convert_connection_id_to_index(&new.connection_hops[0])?;
    let conn = ibc_connections
        .connections
        .get(conn_id)
        .ok_or(VerifyError::WrongConnectionnNumber)?;

    if &convert_hex_to_client_id(&conn.client_id)? != client.client_id() {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.state != State::OpenTry {
        return Err(VerifyError::WrongChannelState);
    }

    let expected = proto::channel::Channel {
        state: proto::channel::State::Init as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        // TODO.
        connection_hops: vec![],
        version: "TODO".into(),
        counterparty: Some(proto::channel::Counterparty {
            channel_id: "".into(),
            port_id: new.port_id,
        }),
    };

    verify_channel_state(
        &client,
        msg.proof_height,
        &msg.proof_init[..],
        &new.counterparty.port_id,
        &new.counterparty.channel_id,
        &expected,
    )?;

    Ok(())
}

fn verify_channel_state(
    client: &impl Client,
    proof_height: Height,
    proof: &[u8],
    port_id: &str,
    channel_id: &str,
    expected: &proto::channel::Channel,
) -> Result<(), VerifyError> {
    client.verify_membership(
        proof_height,
        proof,
        channel_path(port_id, channel_id).as_bytes(),
        &expected.encode_to_vec(),
    )
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
    client: C,
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

    let expected = proto::channel::Channel {
        state: proto::channel::State::Tryopen as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        // TODO.
        connection_hops: vec![],
        version: "TODO".into(),
        counterparty: Some(proto::channel::Counterparty {
            channel_id: get_channel_id_str(new.number),
            port_id: new.port_id,
        }),
    };

    verify_channel_state(
        &client,
        msg.proof_height,
        &msg.proof_try,
        &new.counterparty.port_id,
        &new.counterparty.channel_id,
        &expected,
    )?;

    Ok(())
}

pub fn handle_msg_channel_open_confirm<C: Client>(
    client: C,
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

    let expected = proto::channel::Channel {
        state: proto::channel::State::Open as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        // TODO.
        connection_hops: vec![],
        version: "TODO".into(),
        counterparty: Some(proto::channel::Counterparty {
            channel_id: get_channel_id_str(new.number),
            port_id: new.port_id,
        }),
    };

    verify_channel_state(
        &client,
        msg.proof_height,
        &msg.proof_ack,
        &new.counterparty.port_id,
        &new.counterparty.channel_id,
        &expected,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
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

    if packet_args.port_id != convert_hex_to_port_id(&ibc_packet.packet.source_port_id)?
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.source_channel_id
    {
        return Err(VerifyError::WrongPacketArgs);
    }

    if ibc_packet.packet.destination_channel_id != old_channel.counterparty.channel_id
        || ibc_packet.packet.destination_port_id != old_channel.counterparty.port_id
    {
        return Err(VerifyError::WrongPacketContent);
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

    if ibc_packet.ack.is_some() {
        return Err(VerifyError::WrongPacketAck);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_msg_recv_packet<C: Client>(
    client: C,
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    useless_ibc_packet: Option<IbcPacket>,
    ibc_packet: IbcPacket,
    packet_args: PacketArgs,
    msg: MsgRecvPacket,
) -> Result<(), VerifyError> {
    // A write_ack packet can be consumed.
    if let Some(ibc_packed) = useless_ibc_packet {
        if ibc_packed.status != PacketStatus::WriteAck {
            return Err(VerifyError::WrongUnusedPacket);
        }
    }

    if !old_channel.equal_unless_sequence(&new_channel) {
        return Err(VerifyError::WrongChannel);
    }

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.status != PacketStatus::Recv {
        return Err(VerifyError::WrongPacketStatus);
    }

    if ibc_packet.ack.is_some() {
        return Err(VerifyError::WrongPacketAck);
    }

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if packet_args.port_id != convert_hex_to_port_id(&ibc_packet.packet.destination_port_id)?
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.destination_channel_id
    {
        return Err(VerifyError::WrongPacketArgs);
    }

    let unorder_sequence = if old_channel.order == Ordering::Unordered {
        Some(ibc_packet.packet.sequence)
    } else {
        if ibc_packet.packet.sequence != old_channel.sequence.next_sequence_recvs {
            return Err(VerifyError::WrongPacketSequence);
        }
        None
    };

    if !old_channel
        .sequence
        .next_recv_packet_is(&new_channel.sequence, unorder_sequence)
    {
        return Err(VerifyError::WrongChannelSequence);
    }

    client.verify_membership(
        msg.proof_height,
        &msg.proof_commitment,
        packet_commitment_path(
            &ibc_packet.packet.source_port_id,
            &ibc_packet.packet.source_channel_id,
            ibc_packet.packet.sequence.into(),
        )
        .as_bytes(),
        &sha256(&[
            &ibc_packet.packet.timeout_timestamp.to_le_bytes(),
            // Revision number
            &0u64.to_le_bytes(),
            &ibc_packet.packet.timeout_height.to_le_bytes(),
            &sha256(&[&ibc_packet.packet.data]),
        ]),
    )
}

fn sha256(msgs: &[&[u8]]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for m in msgs {
        hasher.update(m);
    }
    hasher.finalize().into()
}

#[allow(clippy::too_many_arguments)]
pub fn handle_msg_ack_packet<C: Client>(
    client: C,
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

    if !old_ibc_packet
        .packet
        .equal_unless_sequence(&new_ibc_packet.packet)
    {
        return Err(VerifyError::WrongPacketContent);
    }

    let is_unorder = if old_channel.order == Ordering::Unordered {
        true
    } else {
        if new_ibc_packet.packet.sequence != old_channel.sequence.next_sequence_acks {
            return Err(VerifyError::WrongPacketSequence);
        }
        false
    };

    if !old_channel
        .sequence
        .next_recv_ack_is(&new_channel.sequence, is_unorder)
    {
        return Err(VerifyError::WrongChannelSequence);
    }

    if old_ibc_packet.ack.is_some() || new_ibc_packet.ack.is_none() {
        return Err(VerifyError::WrongPacketAck);
    }

    client.verify_membership(
        msg.proof_height,
        &msg.proof_acked,
        packet_acknowledgement_commitment_path(
            &new_ibc_packet.packet.destination_port_id,
            &new_ibc_packet.packet.destination_channel_id,
            new_ibc_packet.packet.sequence.into(),
        )
        .as_bytes(),
        &sha256(&[&new_ibc_packet.ack.unwrap()]),
    )
}

pub fn handle_msg_write_ack_packet(
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    old_ibc_packet: IbcPacket,
    old_packet_args: PacketArgs,
    new_ibc_packet: IbcPacket,
    new_packet_args: PacketArgs,
    _: MsgWriteAckPacket,
) -> Result<(), VerifyError> {
    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if old_channel.state != State::Open || old_channel != new_channel {
        return Err(VerifyError::WrongChannelState);
    }

    if old_ibc_packet.status != PacketStatus::Recv
        && new_ibc_packet.status != PacketStatus::WriteAck
    {
        return Err(VerifyError::WrongPacketStatus);
    }

    if old_packet_args != new_packet_args {
        return Err(VerifyError::WrongPacketArgs);
    }

    if old_ibc_packet.packet != new_ibc_packet.packet {
        return Err(VerifyError::WrongPacketContent);
    }

    if old_ibc_packet.ack.is_some() || new_ibc_packet.ack.is_none() {
        return Err(VerifyError::WrongPacketAck);
    }

    Ok(())
}

pub fn handle_msg_consume_ack_packet(
    old_ibc_packet: IbcPacket,
    _: PacketArgs,
    _: MsgConsumeAckPacket,
) -> Result<(), VerifyError> {
    if old_ibc_packet.status != PacketStatus::Ack {
        return Err(VerifyError::WrongPacketStatus);
    }

    Ok(())
}

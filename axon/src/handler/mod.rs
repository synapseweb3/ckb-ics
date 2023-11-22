#![allow(clippy::too_many_arguments)]

use alloc::string::ToString;
use prost::Message;
use rlp::decode;

use crate::consts::COMMITMENT_PREFIX;
use crate::get_channel_id_str;
use crate::message::{
    Envelope, MsgAckPacket, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck,
    MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry, MsgConsumeAckPacket,
    MsgRecvPacket, MsgSendPacket, MsgType, MsgWriteAckPacket,
};
use crate::object::{ConnectionEnd, Ordering, State, VerifyError, Version};
use crate::proto::client::Height;
use crate::{commitment::*, connection_id, proto};
use crate::{ChannelArgs, ConnectionArgs, PacketArgs};

mod objects;
#[cfg(test)]
mod test;

pub use objects::*;

pub fn handle_msg_connection_open_init<C: Client>(
    _client: C,
    mut old_connections: IbcConnections,
    old_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_args: ConnectionArgs,
    _: MsgConnectionOpenInit,
) -> Result<(), VerifyError> {
    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    let new = new_connections
        .connections
        .last()
        .ok_or(VerifyError::WrongConnectionState)?;

    if !new.counterparty.connection_id.is_empty() {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.versions != [Version::version_1()] {
        return Err(VerifyError::WrongConnectionState);
    }

    old_connections.connections.push(ConnectionEnd {
        state: State::Init,
        counterparty: new.counterparty.clone(),
        versions: new.versions.clone(),
        delay_period: new.delay_period,
    });

    if old_connections != new_connections {
        return Err(VerifyError::WrongConnectionState);
    }

    Ok(())
}

pub fn handle_msg_connection_open_try<C: Client>(
    client: C,
    mut old_connections: IbcConnections,
    old_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenTry,
) -> Result<(), VerifyError> {
    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    let connection = new_connections
        .connections
        .last()
        .ok_or(VerifyError::WrongConnectionState)?;

    let counterparty = &connection.counterparty;

    old_connections.connections.push(ConnectionEnd {
        state: State::OpenTry,
        counterparty: counterparty.clone(),
        versions: vec![Version::version_1()],
        delay_period: connection.delay_period,
    });

    if old_connections != new_connections {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected_connection_end_on_counterparty = proto::connection::ConnectionEnd {
        state: proto::connection::State::Init as _,
        client_id: counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            client_id: new_args.client_id(),
            connection_id: "".to_string(),
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: connection.delay_period,
        versions: vec![Version::version_1().into()],
    };

    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_init,
        &counterparty.connection_id,
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
    mut old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenAck,
) -> Result<(), VerifyError> {
    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    // Verify connection state transition.
    let conn_idx = msg.conn_id_on_a;
    let old_connection = &mut old.connections[conn_idx];
    let new_connection = &new.connections[conn_idx];
    if old_connection.state != State::Init {
        return Err(VerifyError::WrongConnectionState);
    }
    old_connection.state = State::Open;
    old_connection.counterparty.connection_id = new_connection.counterparty.connection_id.clone();
    if old != new {
        return Err(VerifyError::WrongConnectionState);
    }

    // Verify counterparty connection state.
    let client_id = new_args.client_id();
    let expected = proto::connection::ConnectionEnd {
        state: proto::connection::State::Tryopen as _,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            connection_id: connection_id(&client_id, conn_idx),
            client_id,
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: new_connection.delay_period,
        versions: vec![Version::version_1().into()],
    };
    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_try,
        &new_connection.counterparty.connection_id,
        &expected,
    )?;

    // TODO: verify client.
    Ok(())
}

pub fn handle_msg_connection_open_confirm<C: Client>(
    client: C,
    mut old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    msg: MsgConnectionOpenConfirm,
) -> Result<(), VerifyError> {
    if old_args != new_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    // Verify state transition.
    let conn_idx = msg.conn_id_on_b;
    let old_connection = &mut old.connections[conn_idx];
    let new_connection = &new.connections[conn_idx];
    if old_connection.state != State::OpenTry {
        return Err(VerifyError::WrongConnectionState);
    }
    old_connection.state = State::Open;
    if old != new {
        return Err(VerifyError::WrongConnectionState);
    }

    let client_id = new_args.client_id();

    // Verify counterparty state.
    let expected = proto::connection::ConnectionEnd {
        state: proto::connection::State::Open as _,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: Some(proto::connection::Counterparty {
            connection_id: connection_id(&client_id, conn_idx),
            client_id,
            prefix: Some(proto::commitment::MerklePrefix {
                key_prefix: COMMITMENT_PREFIX.to_vec(),
            }),
        }),
        delay_period: new_connection.delay_period,
        versions: vec![Version::version_1().into()],
    };

    verify_connection_state(
        &client,
        msg.proof_height,
        &msg.proof_ack,
        &new_connection.counterparty.connection_id,
        &expected,
    )?;

    Ok(())
}

pub fn handle_channel_open_init_and_try<C: Client>(
    client: C,
    channel: IbcChannel,
    channel_args: ChannelArgs,
    envelop: Envelope,
    mut old_connections: IbcConnections,
    old_connection_args: ConnectionArgs,
    new_connections: IbcConnections,
    new_connection_args: ConnectionArgs,
) -> Result<(), VerifyError> {
    if channel.number != old_connections.next_channel_number {
        return Err(VerifyError::WrongChannel);
    }
    old_connections.next_channel_number += 1;
    if old_connections != new_connections {
        return Err(VerifyError::WrongConnectionState);
    }

    if old_connection_args != new_connection_args {
        return Err(VerifyError::WrongConnectionArgs);
    }

    if channel_args.connection() != old_connection_args
        || channel_args.open
        || channel_args.channel_id != channel.number
        || hex::encode(channel_args.port_id) != channel.port_id
    {
        return Err(VerifyError::WrongChannelArgs);
    }

    match envelop.msg_type {
        MsgType::MsgChannelOpenInit => {
            let init_msg = decode::<MsgChannelOpenInit>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_init(
                &new_connection_args.client_id(),
                &new_connections,
                channel,
                init_msg,
            )
        }
        MsgType::MsgChannelOpenTry => {
            let open_try_msg = decode::<MsgChannelOpenTry>(&envelop.content)
                .map_err(|_| VerifyError::SerdeError)?;
            handle_msg_channel_open_try(
                client,
                &new_connection_args.client_id(),
                &new_connections,
                channel,
                open_try_msg,
            )
        }
        _ => Err(VerifyError::EventNotMatch),
    }
}

pub fn handle_msg_channel_open_init(
    client_id: &str,
    ibc_connections: &IbcConnections,
    new: IbcChannel,
    _msg: MsgChannelOpenInit,
) -> Result<(), VerifyError> {
    if new.connection_hops.len() != 1 {
        return Err(VerifyError::ConnectionsWrong);
    }
    let conn = ibc_connections
        .get_by_id(client_id, &new.connection_hops[0])
        .ok_or(VerifyError::WrongConnectionId)?;

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.state != State::Init || !new.counterparty.channel_id.is_empty() {
        return Err(VerifyError::WrongChannelState);
    }

    if new.sequence != Sequence::default() {
        return Err(VerifyError::WrongPacketSequence);
    }

    if new.counterparty.connection_id != conn.counterparty.connection_id {
        return Err(VerifyError::WrongConnectionCounterparty);
    }

    Ok(())
}

pub fn handle_msg_channel_open_try<C: Client>(
    client: C,
    client_id: &str,
    ibc_connections: &IbcConnections,
    new: IbcChannel,
    msg: MsgChannelOpenTry,
) -> Result<(), VerifyError> {
    if new.connection_hops.len() != 1 {
        return Err(VerifyError::ConnectionsWrong);
    }

    let conn = ibc_connections
        .get_by_id(client_id, &new.connection_hops[0])
        .ok_or(VerifyError::WrongConnectionId)?;

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if new.state != State::OpenTry {
        return Err(VerifyError::WrongChannelState);
    }

    if new.sequence != Sequence::default() {
        return Err(VerifyError::WrongPacketSequence);
    }

    if new.counterparty.connection_id != conn.counterparty.connection_id {
        return Err(VerifyError::WrongConnectionCounterparty);
    }

    let expected = proto::channel::Channel {
        state: proto::channel::State::Init as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        connection_hops: vec![conn.counterparty.connection_id.clone()],
        // We don't have version negotiation, so we assume that new channel
        // version is counterparty version.
        version: new.version,
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
        || old_args.metadata_type_id != new_args.metadata_type_id
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
    mut old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenAck,
) -> Result<(), VerifyError> {
    if old.state != State::Init {
        return Err(VerifyError::WrongChannelState);
    }
    old.state = State::Open;
    // We don't have version negotiation, so we'll just accept counterparty
    // version.
    old.version = new.version.clone();
    old.counterparty.channel_id = new.counterparty.channel_id.clone();
    if old != new {
        return Err(VerifyError::WrongChannel);
    }

    let expected = proto::channel::Channel {
        state: proto::channel::State::Tryopen as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        connection_hops: vec![new.counterparty.connection_id],
        version: new.version,
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
    mut old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenConfirm,
) -> Result<(), VerifyError> {
    if old.state != State::OpenTry {
        return Err(VerifyError::WrongChannelState);
    }
    old.state = State::Open;
    if old != new {
        return Err(VerifyError::WrongChannel);
    }

    let expected = proto::channel::Channel {
        state: proto::channel::State::Open as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        connection_hops: vec![new.counterparty.connection_id],
        version: new.version,
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

pub fn handle_msg_channel_close_init<C: Client>(
    _: C,
    mut old: IbcChannel,
    mut old_args: ChannelArgs,
    new: IbcChannel,
    new_args: ChannelArgs,
    _: MsgChannelCloseInit,
) -> Result<(), VerifyError> {
    if old.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }
    if !old_args.open {
        return Err(VerifyError::WrongChannelArgs);
    }

    old.state = State::Closed;
    old_args.open = false;
    if old != new {
        return Err(VerifyError::WrongChannel);
    }
    if old_args != new_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    Ok(())
}

pub fn handle_msg_channel_close_confirm<C: Client>(
    client: C,
    mut old: IbcChannel,
    mut old_args: ChannelArgs,
    new: IbcChannel,
    new_args: ChannelArgs,
    msg: MsgChannelCloseConfirm,
) -> Result<(), VerifyError> {
    if old.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }
    if !old_args.open {
        return Err(VerifyError::WrongChannelArgs);
    }

    old.state = State::Closed;
    old_args.open = false;
    if old != new {
        return Err(VerifyError::WrongChannel);
    }
    if old_args != new_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    let expected = proto::channel::Channel {
        state: proto::channel::State::Closed as i32,
        ordering: proto::channel::Order::from(new.order) as i32,
        connection_hops: vec![new.counterparty.connection_id],
        version: "TODO".into(),
        counterparty: Some(proto::channel::Counterparty {
            channel_id: get_channel_id_str(new.number),
            port_id: new.port_id,
        }),
    };

    verify_channel_state(
        &client,
        msg.proof_height,
        &msg.proof_init,
        &new.counterparty.port_id,
        &new.counterparty.channel_id,
        &expected,
    )
}

pub fn handle_msg_send_packet<C: Client>(
    _: C,
    mut old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    ibc_packet: IbcPacket,
    packet_args: PacketArgs,
    _: MsgSendPacket,
) -> Result<(), VerifyError> {
    if ibc_packet.packet.sequence != old_channel.sequence.next_sequence_sends {
        return Err(VerifyError::WrongPacketSequence);
    }

    old_channel.sequence.next_sequence_sends += 1;
    if old_channel != new_channel {
        return Err(VerifyError::WrongChannel);
    }

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if hex::encode(packet_args.port_id) != ibc_packet.packet.source_port_id
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

    if new_channel.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    if ibc_packet.status != PacketStatus::Send {
        return Err(VerifyError::WrongPacketStatus);
    }

    if ibc_packet.ack.is_some() {
        return Err(VerifyError::WrongPacketAck);
    }

    Ok(())
}

pub fn handle_msg_recv_packet<C: Client>(
    client: C,
    mut old_channel: IbcChannel,
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

    if old_channel.order == Ordering::Unordered {
        old_channel
            .sequence
            .unorder_receive(ibc_packet.packet.sequence)?;
    } else {
        if old_channel.sequence.next_sequence_recvs != ibc_packet.packet.sequence {
            return Err(VerifyError::WrongPacketSequence);
        }
        old_channel.sequence.next_sequence_recvs += 1;
    }

    if old_channel != new_channel {
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

    if hex::encode(packet_args.port_id) != ibc_packet.packet.destination_port_id
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.destination_channel_id
    {
        return Err(VerifyError::WrongPacketArgs);
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

pub fn handle_msg_ack_packet<C: Client>(
    client: C,
    mut old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    mut old_ibc_packet: IbcPacket,
    old_packet_args: PacketArgs,
    new_ibc_packet: IbcPacket,
    new_packet_args: PacketArgs,
    msg: MsgAckPacket,
) -> Result<(), VerifyError> {
    if old_ibc_packet.status != PacketStatus::Send {
        return Err(VerifyError::WrongPacketStatus);
    }
    old_ibc_packet.status = PacketStatus::Ack;

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if old_packet_args != new_packet_args {
        return Err(VerifyError::WrongPacketArgs);
    }

    if old_ibc_packet.ack.is_some() || new_ibc_packet.ack.is_none() {
        return Err(VerifyError::WrongPacketAck);
    }
    old_ibc_packet.ack = new_ibc_packet.ack.clone();

    if old_ibc_packet != new_ibc_packet {
        return Err(VerifyError::WrongPacketContent);
    }

    if old_channel.order != Ordering::Unordered {
        if new_ibc_packet.packet.sequence != old_channel.sequence.next_sequence_acks {
            return Err(VerifyError::WrongPacketSequence);
        }
        old_channel.sequence.next_sequence_acks += 1;
    }

    if old_channel != new_channel {
        return Err(VerifyError::WrongChannel);
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

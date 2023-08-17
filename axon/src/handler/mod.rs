use alloc::string::ToString;
use alloc::vec::Vec;
use rlp::decode;

use crate::consts::COMMITMENT_PREFIX;
use crate::message::{
    Envelope, MsgAckPacket, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
    MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgRecvPacket, MsgSendPacket, MsgType, MsgWriteAckPacket,
};
use crate::object::{
    ChannelCounterparty, ChannelEnd, ConnectionCounterparty, ConnectionEnd, Ordering, PacketAck,
    State, VerifyError,
};
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
    if &convert_hex_to_client_id(&connection.client_id)? != client.client_id() {
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
    if &convert_hex_to_client_id(&connection.client_id)? != client.client_id() {
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
            client_id: convert_byte32_to_hex(client.client_id()),
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
            client_id: convert_byte32_to_hex(client.client_id()),
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
            client_id: convert_byte32_to_hex(client.client_id()),
            connection_id: Some(conn_idx.to_string()),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        delay_period: new_connection.delay_period,
        versions: vec![Default::default()],
    };

    client.verify_object(expected, msg.proofs.object_proof)
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

    if &convert_hex_to_client_id(&conn.client_id)? != client.client_id() {
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

#[allow(clippy::too_many_arguments)]
pub fn handle_msg_recv_packet<C: Client>(
    mut client: C,
    old_channel: IbcChannel,
    old_channel_args: ChannelArgs,
    new_channel: IbcChannel,
    new_channel_args: ChannelArgs,
    useless_ibc_packet: Option<IbcPacket>,
    ibc_packet: IbcPacket,
    packet_args: PacketArgs,
    msg: MsgRecvPacket,
) -> Result<(), VerifyError> {
    if let Some(ibc_packed) = useless_ibc_packet {
        if ibc_packed.status != PacketStatus::WriteAck
            || ibc_packed.packet.sequence + 1 >= old_channel.sequence.next_sequence_recvs
        {
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

    if old_channel_args != new_channel_args {
        return Err(VerifyError::WrongChannelArgs);
    }

    if packet_args.port_id != convert_hex_to_port_id(&ibc_packet.packet.source_port_id)?
        || packet_args.sequence != ibc_packet.packet.sequence
        || get_channel_id_str(packet_args.channel_id) != ibc_packet.packet.source_channel_id
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

    client.verify_object(ibc_packet.packet, msg.proofs.object_proof)
}

#[allow(clippy::too_many_arguments)]
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

    if !old_ibc_packet
        .packet
        .equal_unless_sequence(&new_ibc_packet.packet)
    {
        return Err(VerifyError::WrongPacketContent);
    }

    let is_unorder = if old_channel.order == Ordering::Unordered {
        true
    } else {
        if new_ibc_packet.packet.sequence != old_channel.sequence.next_sequence_acks
            || old_ibc_packet.packet.sequence != old_channel.sequence.next_sequence_sends
        {
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

    let object = PacketAck {
        ack: msg.acknowledgement,
        packet: new_ibc_packet.packet,
    };

    client.verify_object(object, msg.proofs.object_proof)
}

pub fn handle_msg_write_ack_packet(
    old_ibc_packet: IbcPacket,
    old_packet_args: PacketArgs,
    new_ibc_packet: IbcPacket,
    new_packet_args: PacketArgs,
    _: MsgWriteAckPacket,
) -> Result<(), VerifyError> {
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

    Ok(())
}

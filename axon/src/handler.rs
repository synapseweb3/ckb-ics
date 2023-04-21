// These structs should only be used in CKB contracts.

use crate::message::MsgChannelOpenAck;
use crate::message::MsgChannelOpenConfirm;
use crate::message::MsgChannelOpenInit;
use crate::message::MsgChannelOpenTry;
use crate::message::MsgConnectionOpenAck;
use crate::message::MsgConnectionOpenInit;
use crate::message::MsgConnectionOpenTry;
use crate::object::ChannelEnd;
use crate::object::ChannelId;
use crate::object::ConnectionId;
use crate::object::Ordering;
use crate::object::State;
use crate::object::VerifyError;
use crate::proof::Block;
use crate::verify_object;

// use axon_protocol::types::Bytes;
use super::Bytes;
use super::Vec;

use alloc::string::ToString;
use cstr_core::CString;
use rlp::{Decodable, Encodable};

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
    pub counterparty: ChannelId,
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

#[derive(PartialEq, Eq)]
pub struct Sequence {
    pub next_send_packet: u16,
    pub next_recv_packet: u16,
    pub next_recv_ack: u16,
    pub unorder_recv_packet: Vec<u16>,
    pub unorder_recv_ack: Vec<u16>,
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

    if connection.counterparty.client_id != msg.counterparty.client_id {
        return Err(VerifyError::WrongConnectionCounterparty);
    }

    if connection.client_id != msg.client_id {
        return Err(VerifyError::WrongConnectionClient);
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
        connection_id: ConnectionId {
            client_id: msg.counterparty.client_id.clone(),
            connection_id: msg.counterparty.connection_id.clone(),
        },
        state: State::Init,
        client_id: msg.counterparty.client_id.clone(),
        counterparty: ConnectionId {
            client_id: msg.client_id.clone(),
            connection_id: Some(msg.previous_connection_id.clone()),
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
    for i in 0..old.connections.len() - 1 {
        if old.connections[i] != new.connections[i] {
            return Err(VerifyError::WrongClient);
        }
    }

    let old_connection = &old.connections[old.connections.len() - 1];
    let new_connection = &new.connections[new.connections.len() - 1];

    if old_connection.client_id != new_connection.client_id
        || old_connection.connection_id != new_connection.connection_id
        || old_connection.delay_period != old_connection.delay_period
        || old_connection.counterparty != new_connection.counterparty
    {
        return Err(VerifyError::WrongClient);
    }

    let connection_id = new_connection
        .connection_id
        .connection_id
        .as_ref()
        .ok_or(VerifyError::ConnectionsWrong)?;

    // TODO: Check message
    if &msg.connection_id != connection_id || &msg.counterparty_connection_id != connection_id {
        return Err(VerifyError::ConnectionsWrong);
    }

    if old_connection.state != State::Init || old_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    let expected = ConnectionEnd {
        connection_id: ConnectionId {
            client_id: new_connection.counterparty.client_id.clone(),
            connection_id: new_connection.counterparty.connection_id.clone(),
        },
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionId {
            client_id: client.id.clone(),
            connection_id: Some(connection_id.clone()),
        },
        delay_period: new_connection.delay_period.clone(),
    };
    verify_object(client, expected, msg.proofs.object_proof)
}

pub fn handle_msg_connection_open_confirm(
    client: Client,
    old: IbcConnections,
    new: IbcConnections,
    msg: MsgConnectionOpenAck,
) -> Result<(), VerifyError> {
    if old.connections.len() != new.connections.len() {
        return Err(VerifyError::WrongConnectionCnt);
    }
    for i in 0..old.connections.len() - 1 {
        if old.connections[i] != new.connections[i] {
            return Err(VerifyError::WrongClient);
        }
    }

    let old_connection = &old.connections[old.connections.len() - 1];
    let new_connection = &new.connections[new.connections.len() - 1];

    if old_connection.client_id != new_connection.client_id
        || old_connection.connection_id != new_connection.connection_id
        || old_connection.delay_period != old_connection.delay_period
        || old_connection.counterparty != new_connection.counterparty
    {
        return Err(VerifyError::WrongClient);
    }

    let connection_id = new_connection
        .connection_id
        .connection_id
        .as_ref()
        .ok_or(VerifyError::ConnectionsWrong)?;

    // TODO: Check message
    if &msg.connection_id != connection_id || &msg.counterparty_connection_id != connection_id {
        return Err(VerifyError::ConnectionsWrong);
    }

    if old_connection.state != State::Init || old_connection.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }
    let expected = ConnectionEnd {
        connection_id: ConnectionId {
            client_id: new_connection.counterparty.client_id.clone(),
            connection_id: new_connection.counterparty.connection_id.clone(),
        },
        state: State::Open,
        client_id: new_connection.counterparty.client_id.clone(),
        counterparty: ConnectionId {
            client_id: client.id.clone(),
            connection_id: Some(connection_id.clone()),
        },
        delay_period: new_connection.delay_period.clone(),
    };

    verify_object(client, expected, msg.proofs.object_proof)
}

pub fn handle_msg_channel_open_init(
    client: Client,
    conn: ConnectionEnd,
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
    conn: ConnectionEnd,
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
        channel_id: ChannelId {
            port_id: new.counterparty.port_id,
            channel_id: new.counterparty.channel_id,
        },
        state: State::Init,
        ordering: new.order,
        remote: ChannelId {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
    };

    verify_object(client, object, msg.proofs.object_proof)
}

pub fn handle_msg_channel_open_ack(
    client: Client,
    conn: ConnectionEnd,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenAck,
) -> Result<(), VerifyError> {
    if conn.client_id != client.id {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if !new.equal_unless_state(&old) {
        return Err(VerifyError::WrongChannel);
    }

    if old.state != State::Init && new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        channel_id: ChannelId {
            port_id: new.counterparty.port_id,
            channel_id: new.counterparty.channel_id,
        },
        state: State::OpenTry,
        ordering: new.order,
        remote: ChannelId {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
    };

    verify_object(client, object, msg.proofs.object_proof)
}

pub fn handle_msg_channel_open_confirm(
    client: Client,
    conn: ConnectionEnd,
    old: IbcChannel,
    new: IbcChannel,
    msg: MsgChannelOpenConfirm,
) -> Result<(), VerifyError> {
    if conn.client_id != client.id {
        return Err(VerifyError::WrongConnectionClient);
    }

    if conn.state != State::Open {
        return Err(VerifyError::WrongConnectionState);
    }

    if !new.equal_unless_state(&old) {
        return Err(VerifyError::WrongChannel);
    }
    if old.state != State::OpenTry && new.state != State::Open {
        return Err(VerifyError::WrongChannelState);
    }

    let object = ChannelEnd {
        channel_id: ChannelId {
            port_id: new.counterparty.port_id,
            channel_id: new.counterparty.channel_id,
        },
        state: State::OpenTry,
        ordering: new.order,
        remote: ChannelId {
            port_id: new.port_id,
            channel_id: CString::new(new.num.to_string()).unwrap(),
        },
    };

    verify_object(client, object, msg.proofs.object_proof)
}

// These structs should only be used in CKB contracts.

use crate::message::MsgConnectionOpenInit;
use crate::message::MsgConnectionOpenTry;
use crate::object::ConnectionId;
use crate::object::State;
use crate::object::VerifyError;
use crate::proof::Block;
use crate::verify_object;

// use axon_protocol::types::Bytes;
use super::Bytes;
use super::Vec;

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

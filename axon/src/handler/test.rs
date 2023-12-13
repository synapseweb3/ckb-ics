use alloc::string::String;

use crate::handler::*;
use crate::message::CommitmentKV;
use crate::object::ChannelCounterparty;
use crate::object::ConnectionCounterparty;
use crate::object::ConnectionEnd;
use crate::object::Packet;
use crate::proto::client::Height;

#[derive(Debug, Default)]
pub struct TestClient {}

impl Client for TestClient {
    fn verify_membership(
        &self,
        _height: Height,
        _proof: &[u8],
        _path: &[u8],
        _value: &[u8],
    ) -> Result<(), VerifyError> {
        Ok(())
    }
}

#[test]
fn test_handle_msg_connection_open_init() {
    let new_connection_end = ConnectionEnd {
        state: State::Init,
        ..Default::default()
    };

    let old_connections = IbcConnections::default();
    let new_connections = IbcConnections {
        connections: vec![new_connection_end],
        ..Default::default()
    };

    let old_args = ConnectionArgs::default();
    let new_args = ConnectionArgs::default();
    handle_msg_connection_open_init(
        old_connections,
        old_args,
        new_connections,
        new_args,
        &mut Vec::new(),
    )
    .unwrap();
}

#[test]
fn test_handle_msg_connection_open_try() {
    let client = TestClient::default();

    let new_connection_end = ConnectionEnd {
        state: State::OpenTry,
        counterparty: ConnectionCounterparty {
            client_id: String::from("dummy"),
            connection_id: "dummy".into(),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        ..Default::default()
    };

    let old_connections = IbcConnections::default();
    let new_connections = IbcConnections {
        connections: vec![new_connection_end],
        ..Default::default()
    };

    let msg = MsgConnectionOpenTry {
        proof_height: Height {
            revision_height: 0,
            revision_number: 0,
        },
        proof_init: vec![],
    };
    let old_args = ConnectionArgs::default();
    let new_args = ConnectionArgs::default();

    handle_msg_connection_open_try(
        client,
        old_connections,
        old_args,
        new_connections,
        new_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn test_handle_msg_connection_open_ack() {
    let client = TestClient::default();

    let msg = MsgConnectionOpenAck {
        conn_id_on_a: 1,
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_try: vec![],
    };

    let old_connection_end = ConnectionEnd {
        state: State::Init,
        ..Default::default()
    };

    let new_connection_end = ConnectionEnd {
        state: State::Open,
        counterparty: ConnectionCounterparty {
            connection_id: "connection".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let old_connections = IbcConnections {
        connections: vec![
            ConnectionEnd::default(),
            old_connection_end,
            ConnectionEnd::default(),
        ],
        ..Default::default()
    };

    let new_connections = IbcConnections {
        connections: vec![
            ConnectionEnd::default(),
            new_connection_end,
            ConnectionEnd::default(),
        ],
        ..Default::default()
    };

    let old_args = ConnectionArgs::default();
    let new_args = ConnectionArgs::default();
    handle_msg_connection_open_ack(
        client,
        old_connections,
        old_args,
        new_connections,
        new_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn test_handle_msg_connection_open_confirm() {
    let client = TestClient::default();

    let msg = MsgConnectionOpenConfirm {
        conn_id_on_b: 1,
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_ack: vec![],
    };

    let old_connection_end = ConnectionEnd {
        state: State::OpenTry,
        counterparty: ConnectionCounterparty {
            connection_id: "connection-1".into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let new_connection_end = ConnectionEnd {
        state: State::Open,
        counterparty: ConnectionCounterparty {
            connection_id: "connection-1".into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let old_connections = IbcConnections {
        connections: vec![
            ConnectionEnd::default(),
            old_connection_end,
            ConnectionEnd::default(),
        ],
        ..Default::default()
    };

    let new_connections = IbcConnections {
        connections: vec![
            ConnectionEnd::default(),
            new_connection_end,
            ConnectionEnd::default(),
        ],
        ..Default::default()
    };

    let old_args = ConnectionArgs::default();
    let new_args = ConnectionArgs::default();
    handle_msg_connection_open_confirm(
        client,
        old_connections,
        old_args,
        new_connections,
        new_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn test_handle_msg_channel_open_init() {
    let connection_end = ConnectionEnd {
        state: State::Open,
        ..Default::default()
    };
    let connection_args = ConnectionArgs::default();
    let channel_args = ChannelArgs::default();

    let old_connections = IbcConnections {
        next_channel_number: 0,
        connections: vec![connection_end.clone()],
    };

    let new_connections = IbcConnections {
        next_channel_number: 1,
        connections: vec![connection_end],
    };

    let channel = IbcChannel {
        state: State::Init,
        connection_hops: vec![connection_id(&connection_args.client_id(), 0)],
        ..Default::default()
    };

    handle_msg_channel_open_init(
        old_connections,
        connection_args,
        new_connections,
        connection_args,
        channel,
        channel_args,
        &mut Vec::new(),
    )
    .unwrap();
}

#[test]
fn test_handle_msg_channel_open_try_success() {
    let client = TestClient::default();

    let connection_end = ConnectionEnd {
        state: State::Open,
        ..Default::default()
    };

    let connection_args = ConnectionArgs::default();
    let channel_args = ChannelArgs::default();

    let old_connections = IbcConnections {
        next_channel_number: 0,
        connections: vec![connection_end.clone()],
    };

    let new_connections = IbcConnections {
        next_channel_number: 1,
        connections: vec![connection_end],
    };

    let channel = IbcChannel {
        state: State::OpenTry,
        connection_hops: vec![connection_id(&connection_args.client_id(), 0)],
        ..Default::default()
    };

    let msg = MsgChannelOpenTry {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_init: vec![],
    };

    handle_msg_channel_open_try(
        client,
        old_connections,
        connection_args,
        new_connections,
        connection_args,
        channel,
        channel_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap()
}

#[test]
fn test_handle_msg_channel_open_ack_success() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::Init,
        counterparty: ChannelCounterparty {
            channel_id: "".to_string(),
            port_id: "portid".to_string(),
            connection_id: "connection-2".to_string(),
        },
        ..Default::default()
    };

    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open,
        counterparty: ChannelCounterparty {
            channel_id: "channel-id".to_string(),
            port_id: "portid".to_string(),
            connection_id: "connection-2".to_string(),
        },
        ..Default::default()
    };

    let msg = MsgChannelOpenAck {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_try: vec![],
    };

    handle_msg_channel_open_ack(
        client,
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn handle_msg_channel_open_confirm_success() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::OpenTry,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let msg = MsgChannelOpenConfirm {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_ack: vec![],
    };

    handle_msg_channel_open_confirm(
        client,
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn handle_msg_channel_close_init_success() {
    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let old_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Closed,
        ..Default::default()
    };

    let new_args = ChannelArgs {
        open: false,
        ..Default::default()
    };

    handle_msg_channel_close_init(
        old_channel,
        old_args,
        new_channel,
        new_args,
        &mut Vec::new(),
    )
    .unwrap();
}

#[test]
fn handle_msg_channel_close_init_failure() {
    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let old_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Closed,
        ..Default::default()
    };

    let new_args = ChannelArgs {
        open: true, // wrong open state
        ..Default::default()
    };

    if let Err(VerifyError::WrongChannelArgs) = handle_msg_channel_close_init(
        old_channel,
        old_args,
        new_channel,
        new_args,
        &mut Vec::new(),
    ) {
    } else {
        panic!()
    }
}

#[test]
fn handle_msg_channel_close_confirm_success() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let old_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Closed,
        ..Default::default()
    };

    let new_args = ChannelArgs {
        open: false,
        ..Default::default()
    };

    let msg = MsgChannelCloseConfirm {
        proof_height: Height::default(),
        proof_init: vec![],
    };

    handle_msg_channel_close_confirm(
        client,
        old_channel,
        old_args,
        new_channel,
        new_args,
        &mut Vec::new(),
        msg,
    )
    .unwrap();
}

#[test]
fn handle_msg_channel_close_confirm_failure() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let old_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open, // wrong channel state
        ..Default::default()
    };

    let new_args = ChannelArgs {
        open: false,
        ..Default::default()
    };

    let msg = MsgChannelCloseConfirm {
        proof_height: Default::default(),
        proof_init: vec![],
    };

    if let Err(VerifyError::WrongChannel) = handle_msg_channel_close_confirm(
        client,
        old_channel,
        old_args,
        new_channel,
        new_args,
        &mut Vec::new(),
        msg,
    ) {
    } else {
        panic!()
    }
}

#[test]
fn handle_msg_channel_open_confirm_channel_unmatch() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::OpenTry,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open,
        order: Ordering::Ordered,
        ..Default::default()
    };

    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs {
        open: true,
        ..Default::default()
    };

    let msg = MsgChannelOpenConfirm {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_ack: vec![],
    };

    if let Err(VerifyError::WrongChannel) = handle_msg_channel_open_confirm(
        client,
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        &mut Vec::new(),
        msg,
    ) {
    } else {
        panic!()
    }
}

#[test]
fn test_handle_msg_send_packet_success() {
    let mut seq2 = Sequence::default();
    seq2.next_sequence_sends += 1;

    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };
    let new_channel = IbcChannel {
        sequence: seq2,
        state: State::Open,
        ..Default::default()
    };

    let ibc_packet = IbcPacket {
        packet: Packet {
            destination_channel_id: old_channel.counterparty.channel_id.clone(),
            destination_port_id: old_channel.counterparty.port_id.clone(),
            sequence: 1,
            ..Default::default()
        },
        status: PacketStatus::Send,
        ack: None,
    };

    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs::default();
    let packet_args = PacketArgs {
        sequence: 1,
        ..Default::default()
    };

    handle_msg_send_packet(
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        ibc_packet,
        packet_args,
        &mut Vec::new(),
    )
    .unwrap();
}

#[test]
fn test_msg_recv_packet_success() {
    let seq1 = Sequence::default();
    let mut seq2 = Sequence::default();
    seq2.next_sequence_recvs += 1;

    let old_channel = IbcChannel {
        sequence: seq1,
        state: State::Open,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        sequence: seq2,
        state: State::Open,
        ..Default::default()
    };

    let ibc_packet = IbcPacket {
        packet: Packet {
            sequence: 1,
            ..Packet::default()
        },
        status: PacketStatus::Recv,
        ack: None,
    };
    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs::default();
    let packet_args = PacketArgs {
        sequence: 1,
        ..Default::default()
    };

    handle_msg_recv_packet(
        TestClient::default(),
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        None,
        ibc_packet,
        packet_args,
        &mut Vec::new(),
        MsgRecvPacket {
            proof_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            proof_commitment: vec![],
        },
    )
    .unwrap();
}

#[test]
fn test_msg_ack_outbox_packet_success() {
    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };
    let old_channel_args = ChannelArgs::default();
    let new_channel = old_channel.clone();
    let new_channel_args = ChannelArgs::default();

    let packet = Packet::default();
    let old_ibc_packet = IbcPacket {
        packet: packet.clone(),
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet,
        status: PacketStatus::WriteAck,
        ack: Some(vec![1]),
    };
    handle_msg_write_ack_packet(
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        old_ibc_packet,
        PacketArgs::default(),
        new_ibc_packet,
        PacketArgs::default(),
        &mut Vec::new(),
    )
    .unwrap();
}

#[test]
fn test_msg_write_ack_packet_channel_state_error() {
    let old_channel = IbcChannel {
        state: State::Init,
        ..Default::default()
    };
    let old_channel_args = ChannelArgs::default();
    let new_channel = old_channel.clone();
    let new_channel_args = ChannelArgs::default();

    let old_packet = Packet::default();
    let new_packet = old_packet.clone();
    let old_ibc_packet = IbcPacket {
        packet: old_packet,
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet: new_packet,
        status: PacketStatus::WriteAck,
        ack: Some(vec![1]),
    };
    if let Err(VerifyError::WrongChannelState) = handle_msg_write_ack_packet(
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        old_ibc_packet,
        PacketArgs::default(),
        new_ibc_packet,
        PacketArgs::default(),
        &mut Vec::new(),
    ) {
    } else {
        panic!()
    }
}

#[test]
fn test_msg_ack_outbox_packet_differenct_packet() {
    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };
    let old_channel_args = ChannelArgs::default();
    let new_channel = old_channel.clone();
    let new_channel_args = ChannelArgs::default();

    let old_packet = Packet::default();
    let mut new_packet = old_packet.clone();
    new_packet.sequence = 1;
    let old_ibc_packet = IbcPacket {
        packet: old_packet,
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet: new_packet,
        status: PacketStatus::WriteAck,
        ack: None,
    };
    if let Err(VerifyError::WrongPacketContent) = handle_msg_write_ack_packet(
        old_channel,
        old_channel_args,
        new_channel,
        new_channel_args,
        old_ibc_packet,
        PacketArgs::default(),
        new_ibc_packet,
        PacketArgs::default(),
        &mut Vec::new(),
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

struct ClientWithCommitments {
    commitments: Vec<CommitmentKV>,
}

fn client_with_commitments(commitments: Vec<CommitmentKV>) -> ClientWithCommitments {
    ClientWithCommitments { commitments }
}

impl Client for ClientWithCommitments {
    fn verify_membership(
        &self,
        _height: Height,
        _proof: &[u8],
        path: &[u8],
        value: &[u8],
    ) -> Result<(), VerifyError> {
        let expected = CommitmentKV::hash(path, value);
        if self.commitments.iter().any(|c| *c == expected) {
            Ok(())
        } else {
            Err(VerifyError::Mpt)
        }
    }
}

#[test]
fn test_connection_commitment_ping_pong() {
    let a_connections_before_init = IbcConnections::default();
    let a_args = ConnectionArgs::default();
    let b_args = ConnectionArgs {
        metadata_type_id: [3; 32],
        ibc_handler_address: [4; 20],
    };
    let a_connections_after_init = IbcConnections {
        connections: vec![ConnectionEnd {
            state: State::Init,
            counterparty: ConnectionCounterparty {
                client_id: b_args.client_id(),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut init_commitments = Vec::new();
    handle_msg_connection_open_init(
        a_connections_before_init.clone(),
        a_args,
        a_connections_after_init.clone(),
        a_args,
        &mut init_commitments,
    )
    .unwrap();

    let mut try_commitments = Vec::new();
    let b_connections_after_try = IbcConnections {
        connections: vec![ConnectionEnd {
            state: State::OpenTry,
            counterparty: ConnectionCounterparty {
                client_id: a_args.client_id(),
                connection_id: connection_id(&a_args.client_id(), 0),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    handle_msg_connection_open_try(
        client_with_commitments(init_commitments),
        a_connections_before_init.clone(),
        b_args,
        b_connections_after_try.clone(),
        b_args,
        &mut try_commitments,
        MsgConnectionOpenTry {
            proof_height: Height::default(),
            proof_init: vec![],
        },
    )
    .unwrap();

    let mut ack_commitments = Vec::new();
    let a_connection_after_ack = IbcConnections {
        connections: vec![ConnectionEnd {
            state: State::Open,
            counterparty: ConnectionCounterparty {
                client_id: b_args.client_id(),
                connection_id: connection_id(&b_args.client_id(), 0),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    handle_msg_connection_open_ack(
        client_with_commitments(try_commitments),
        a_connections_after_init.clone(),
        a_args,
        a_connection_after_ack,
        a_args,
        &mut ack_commitments,
        MsgConnectionOpenAck {
            conn_id_on_a: 0,
            proof_height: Height::default(),
            proof_try: vec![],
        },
    )
    .unwrap();

    let b_connections_after_confirm = IbcConnections {
        connections: vec![ConnectionEnd {
            state: State::Open,
            counterparty: ConnectionCounterparty {
                client_id: a_args.client_id(),
                connection_id: connection_id(&a_args.client_id(), 0),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    handle_msg_connection_open_confirm(
        client_with_commitments(ack_commitments),
        b_connections_after_try.clone(),
        b_args,
        b_connections_after_confirm,
        b_args,
        &mut Vec::new(),
        MsgConnectionOpenConfirm {
            conn_id_on_b: 0,
            proof_height: Height::default(),
            proof_ack: vec![],
        },
    )
    .unwrap();
}

#[test]
fn test_channel_packet_commitment_ping_pong() {
    let a_conn_args = ConnectionArgs::default();
    let b_conn_args = ConnectionArgs {
        metadata_type_id: [3; 32],
        ibc_handler_address: [4; 20],
    };
    let b_conn_id = connection_id(&b_conn_args.client_id(), 1);
    let a_conn_id = connection_id(&a_conn_args.client_id(), 0);
    let a_conns = IbcConnections {
        connections: vec![ConnectionEnd {
            state: State::Open,
            counterparty: ConnectionCounterparty {
                client_id: b_conn_args.client_id(),
                connection_id: b_conn_id.clone(),
                ..Default::default()
            },
            ..Default::default()
        }],
        next_channel_number: 0,
    };
    let b_conns = IbcConnections {
        connections: vec![
            ConnectionEnd::default(),
            ConnectionEnd {
                state: State::Open,
                counterparty: ConnectionCounterparty {
                    client_id: a_conn_args.client_id(),
                    connection_id: a_conn_id.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ],
        next_channel_number: 1,
    };

    let mut a_conns_after_init = a_conns.clone();
    a_conns_after_init.next_channel_number += 1;
    let a_channel_args = ChannelArgs {
        metadata_type_id: a_conn_args.metadata_type_id,
        ibc_handler_address: a_conn_args.ibc_handler_address,
        channel_id: 0,
        open: false,
        port_id: [7; 32],
    };
    let b_channel_args = ChannelArgs {
        metadata_type_id: b_conn_args.metadata_type_id,
        ibc_handler_address: b_conn_args.ibc_handler_address,
        channel_id: 1,
        open: false,
        port_id: [9; 32],
    };
    let a_channel_init = IbcChannel {
        state: State::Init,
        number: 0,
        order: Ordering::Unordered,
        port_id: a_channel_args.port_id_str(),
        connection_hops: vec![a_conn_id.clone()],
        counterparty: ChannelCounterparty {
            channel_id: "".into(),
            port_id: b_channel_args.port_id_str(),
            connection_id: b_conn_id.clone(),
        },
        version: "a-version".into(),
        sequence: Sequence::default(),
    };
    let mut commitments = Vec::new();
    handle_msg_channel_open_init(
        a_conns.clone(),
        a_conn_args,
        a_conns_after_init.clone(),
        a_conn_args,
        a_channel_init.clone(),
        a_channel_args,
        &mut commitments,
    )
    .unwrap();

    let mut b_conns_after_try = b_conns.clone();
    b_conns_after_try.next_channel_number += 1;
    let b_channel = IbcChannel {
        state: State::OpenTry,
        number: 1,
        order: Ordering::Unordered,
        port_id: b_channel_args.port_id_str(),
        connection_hops: vec![b_conn_id.clone()],
        counterparty: ChannelCounterparty {
            channel_id: a_channel_args.channel_id_str(),
            port_id: a_channel_args.port_id_str(),
            connection_id: a_conn_id,
        },
        version: "a-version".into(),
        sequence: Sequence::default(),
    };
    let mut try_commitments = Vec::new();
    handle_msg_channel_open_try(
        client_with_commitments(commitments),
        b_conns,
        b_conn_args,
        b_conns_after_try,
        b_conn_args,
        b_channel.clone(),
        b_channel_args,
        &mut try_commitments,
        MsgChannelOpenTry {
            proof_height: Height::default(),
            proof_init: vec![],
        },
    )
    .unwrap();

    let mut a_channel_args_open = a_channel_args;
    a_channel_args_open.open = true;
    let mut a_channel_ack = a_channel_init.clone();
    a_channel_ack.state = State::Open;
    a_channel_ack.counterparty.channel_id = b_channel_args.channel_id_str();
    let mut ack_commitments = Vec::new();
    handle_msg_channel_open_ack(
        client_with_commitments(try_commitments),
        a_channel_init,
        a_channel_args,
        a_channel_ack.clone(),
        a_channel_args_open,
        &mut ack_commitments,
        MsgChannelOpenAck {
            proof_height: Height::default(),
            proof_try: vec![],
        },
    )
    .unwrap();

    let mut b_channel_args_open = b_channel_args;
    b_channel_args_open.open = true;
    let mut b_channel_confirm = b_channel.clone();
    b_channel_confirm.state = State::Open;
    handle_msg_channel_open_confirm(
        client_with_commitments(ack_commitments),
        b_channel,
        b_channel_args,
        b_channel_confirm.clone(),
        b_channel_args_open,
        &mut Vec::new(),
        MsgChannelOpenConfirm {
            proof_height: Height::default(),
            proof_ack: vec![],
        },
    )
    .unwrap();

    let packet = IbcPacket {
        packet: Packet {
            sequence: 1,
            source_port_id: a_channel_args.port_id_str(),
            source_channel_id: a_channel_args.channel_id_str(),
            destination_port_id: b_channel_args.port_id_str(),
            destination_channel_id: b_channel_args.channel_id_str(),
            data: vec![73; 8],
            timeout_height: 0,
            timeout_timestamp: 0,
        },
        status: PacketStatus::Send,
        ack: None,
    };
    let a_packet_args = PacketArgs {
        ibc_handler_address: a_channel_args.ibc_handler_address,
        channel_id: a_channel_args.channel_id,
        port_id: a_channel_args.port_id,
        sequence: 1,
    };

    let mut a_channel_sent = a_channel_ack.clone();
    a_channel_sent.sequence.next_sequence_sends += 1;
    let mut send_commitments = Vec::new();
    handle_msg_send_packet(
        a_channel_ack,
        a_channel_args_open,
        a_channel_sent.clone(),
        a_channel_args_open,
        packet.clone(),
        a_packet_args,
        &mut send_commitments,
    )
    .unwrap();

    let mut b_channel_recv = b_channel_confirm.clone();
    b_channel_recv.sequence.unorder_receive(1).unwrap();
    let mut b_packet = packet.clone();
    b_packet.status = PacketStatus::Recv;
    let b_packet_args = PacketArgs {
        ibc_handler_address: b_channel_args.ibc_handler_address,
        channel_id: b_channel_args.channel_id,
        port_id: b_channel_args.port_id,
        sequence: 1,
    };
    handle_msg_recv_packet(
        client_with_commitments(send_commitments),
        b_channel_confirm,
        b_channel_args_open,
        b_channel_recv.clone(),
        b_channel_args_open,
        None,
        b_packet.clone(),
        b_packet_args,
        &mut Vec::new(),
        MsgRecvPacket {
            proof_height: Height::default(),
            proof_commitment: vec![],
        },
    )
    .unwrap();

    let mut b_packet_ack = b_packet.clone();
    b_packet_ack.status = PacketStatus::WriteAck;
    b_packet_ack.ack = Some("ack".into());

    let mut ack_commitments = Vec::new();
    handle_msg_write_ack_packet(
        b_channel_recv.clone(),
        b_channel_args_open,
        b_channel_recv.clone(),
        b_channel_args_open,
        b_packet,
        b_packet_args,
        b_packet_ack,
        b_packet_args,
        &mut ack_commitments,
    )
    .unwrap();

    let mut a_packet_acked = packet.clone();
    a_packet_acked.status = PacketStatus::Ack;
    a_packet_acked.ack = Some("ack".into());
    let a_channel_acked = a_channel_sent.clone();
    handle_msg_ack_packet(
        client_with_commitments(ack_commitments),
        a_channel_sent,
        a_channel_args_open,
        a_channel_acked.clone(),
        a_channel_args_open,
        packet,
        a_packet_args,
        a_packet_acked,
        a_packet_args,
        &mut Vec::new(),
        MsgAckPacket {
            proof_height: Height::default(),
            proof_acked: vec![],
        },
    )
    .unwrap();

    let mut a_channel_closed = a_channel_acked.clone();
    a_channel_closed.state = State::Closed;
    let mut close_commitments = Vec::new();
    handle_msg_channel_close_init(
        a_channel_acked.clone(),
        a_channel_args_open,
        a_channel_closed,
        a_channel_args,
        &mut close_commitments,
    )
    .unwrap();

    let mut b_channel_close = b_channel_recv.clone();
    b_channel_close.state = State::Closed;
    handle_msg_channel_close_confirm(
        client_with_commitments(close_commitments),
        b_channel_recv,
        b_channel_args_open,
        b_channel_close,
        b_channel_args,
        &mut Vec::new(),
        MsgChannelCloseConfirm {
            proof_height: Height::default(),
            proof_init: vec![],
        },
    )
    .unwrap();
}

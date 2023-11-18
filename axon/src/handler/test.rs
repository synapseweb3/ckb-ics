use alloc::string::String;

use crate::consts;
use crate::handler::*;
use crate::object::ChannelCounterparty;
use crate::object::ConnectionCounterparty;
use crate::object::ConnectionEnd;
use crate::object::Packet;
use crate::proto::client::Height;

fn index_to_connection_id(index: usize) -> String {
    format!("{}{index}", consts::CONNECTION_ID_PREFIX)
}

#[derive(Debug, Default)]
pub struct TestClient {
    client: [u8; 32],
}

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

    fn client_id(&self) -> &[u8; 32] {
        &self.client
    }
}

#[test]
fn test_handle_msg_connection_open_init() {
    let client = TestClient::default();

    let new_connection_end = ConnectionEnd {
        state: State::Init,
        client_id: convert_byte32_to_hex(&[0u8; 32]),
        ..Default::default()
    };

    let old_connections = IbcConnections::default();
    let new_connections = IbcConnections {
        connections: vec![new_connection_end],
        ..Default::default()
    };

    let old_args = ConnectionArgs::default();
    let new_args = ConnectionArgs::default();

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

    let new_connection_end = ConnectionEnd {
        state: State::OpenTry,
        client_id: convert_byte32_to_hex(&[0u8; 32]),
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
        msg,
    )
    .unwrap();
}

#[test]
fn test_handle_msg_channel_open_init() {
    let client = TestClient::default();

    let connection_end = ConnectionEnd {
        state: State::Open,
        ..Default::default()
    };

    let new_connections = IbcConnections {
        next_channel_number: 1,
        connections: vec![connection_end],
    };

    let channel = IbcChannel {
        state: State::Init,
        connection_hops: vec![index_to_connection_id(0)],
        ..Default::default()
    };

    let msg = MsgChannelOpenInit {};
    handle_msg_channel_open_init(client, &new_connections, channel, msg).unwrap();
}

#[test]
fn test_handle_msg_channel_open_try_success() {
    let client = TestClient::default();

    let connection_end = ConnectionEnd {
        state: State::Open,
        ..Default::default()
    };

    let new_connections = IbcConnections {
        next_channel_number: 1,
        connections: vec![connection_end],
    };

    let channel = IbcChannel {
        state: State::OpenTry,
        connection_hops: vec![index_to_connection_id(0)],
        ..Default::default()
    };

    let msg = MsgChannelOpenTry {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_init: vec![],
    };

    handle_msg_channel_open_try(client, &new_connections, channel, msg).unwrap()
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

    handle_msg_channel_open_ack(client, old_channel, new_channel, msg).unwrap();
}

#[test]
fn test_handle_msg_channel_open_ack_failed() {
    let client = TestClient::default();
    let old_channel = IbcChannel {
        number: 0,
        port_id: String::from("b6ac779881b4fe05a167e413ff534469b6b5f6c06d95e4c523eb2945d85ed450"),
        state: State::Init,
        order: Ordering::Unordered,
        sequence: Sequence::default(),
        counterparty: ChannelCounterparty {
            port_id: String::from(
                "54d043fc84623f7a9f7383e1a332c524f0def68608446fc420316c30dfc00f01",
            ),
            channel_id: String::from(""),
            connection_id: "connection-2".into(),
        },
        connection_hops: vec![index_to_connection_id(0)],
        version: "".into(),
    };
    let new_channel = IbcChannel {
        number: 0,
        port_id: String::from("b6ac779881b4fe05a167e413ff534469b6b5f6c06d95e4c523eb2945d85ed450"),
        state: State::Open,
        order: Ordering::Unordered,
        sequence: Sequence::default(),
        counterparty: ChannelCounterparty {
            port_id: String::from(
                "54d043fc84623f7a9f7383e1a332c524f0def68608446fc420316c30dfc00f01",
            ),
            channel_id: String::from("channel-1"),
            connection_id: "connection-2".into(),
        },
        connection_hops: vec![index_to_connection_id(0)],
        version: "".into(),
    };

    let old_args = ChannelArgs {
        client_id: [
            59, 202, 83, 204, 94, 60, 251, 53, 29, 14, 91, 232, 113, 191, 94, 227, 72, 206, 76,
            254, 177, 59, 247, 13, 54, 105, 235, 22, 75, 21, 45, 12,
        ],
        ibc_handler_address: [7; 20],
        open: false,
        channel_id: 0,
        port_id: [
            182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182, 181,
            246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
        ],
    };

    let new_args = ChannelArgs {
        client_id: [
            59, 202, 83, 204, 94, 60, 251, 53, 29, 14, 91, 232, 113, 191, 94, 227, 72, 206, 76,
            254, 177, 59, 247, 13, 54, 105, 235, 22, 75, 21, 45, 12,
        ],
        ibc_handler_address: [7; 20],
        open: true,
        channel_id: 0,
        port_id: [
            182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182, 181,
            246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
        ],
    };

    let envelope = Envelope {
        msg_type: MsgType::MsgChannelOpenAck,
        content: rlp::encode(&MsgChannelOpenAck {
            proof_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            proof_try: vec![],
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

    let old_channel = IbcChannel {
        state: State::OpenTry,
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };

    let msg = MsgChannelOpenConfirm {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_ack: vec![],
    };

    handle_msg_channel_open_confirm(client, old_channel, new_channel, msg).unwrap();
}

#[test]
fn handle_msg_channel_close_init_success() {
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

    let msg = MsgChannelCloseInit {};

    handle_msg_channel_close_init(client, old_channel, old_args, new_channel, new_args, msg)
        .unwrap();
}

#[test]
fn handle_msg_channel_close_init_failure() {
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
        open: true, // wrong open state
        ..Default::default()
    };

    let msg = MsgChannelCloseInit {};

    if let Err(VerifyError::WrongChannelArgs) =
        handle_msg_channel_close_init(client, old_channel, old_args, new_channel, new_args, msg)
    {
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

    handle_msg_channel_close_confirm(client, old_channel, old_args, new_channel, new_args, msg)
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

    if let Err(VerifyError::WrongChannel) =
        handle_msg_channel_close_confirm(client, old_channel, old_args, new_channel, new_args, msg)
    {
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

    let msg = MsgChannelOpenConfirm {
        proof_height: Height {
            revision_number: 0,
            revision_height: 0,
        },
        proof_ack: vec![],
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

    let old_channel = IbcChannel {
        state: State::Open,
        ..Default::default()
    };
    let new_channel = IbcChannel {
        sequence: seq2,
        state: State::Open,
        ..Default::default()
    };
    let msg = MsgSendPacket {};

    let ibc_packet = IbcPacket {
        packet: Packet {
            destination_channel_id: old_channel.counterparty.channel_id.clone(),
            destination_port_id: old_channel.counterparty.port_id.clone(),
            sequence: 1,
            ..Default::default()
        },
        tx_hash: None,
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
        tx_hash: None,
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
        tx_hash: None,
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet,
        tx_hash: None,
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
        MsgWriteAckPacket {},
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
        tx_hash: None,
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet: new_packet,
        tx_hash: None,
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
        MsgWriteAckPacket {},
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
        tx_hash: None,
        status: PacketStatus::Recv,
        ack: None,
    };
    let new_ibc_packet = IbcPacket {
        packet: new_packet,
        tx_hash: None,
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
        MsgWriteAckPacket {},
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

use alloc::string::String;

use crate::consts;
use crate::handler::*;
use crate::object::{Object, Packet};
use crate::proof::ObjectProof;

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
        next_connection_number: 1,
        connections: vec![new_connection_end],
        ..Default::default()
    };

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

    let new_connection_end = ConnectionEnd {
        state: State::OpenTry,
        client_id: convert_byte32_to_hex(&[0u8; 32]),
        counterparty: ConnectionCounterparty {
            client_id: String::from("dummy"),
            connection_id: Some(String::from("dummy")),
            commitment_prefix: COMMITMENT_PREFIX.to_vec(),
        },
        ..Default::default()
    };

    let old_connections = IbcConnections::default();
    let new_connections = IbcConnections {
        next_connection_number: 1,
        connections: vec![new_connection_end],
        ..Default::default()
    };

    let msg = MsgConnectionOpenTry {
        proof: Default::default(),
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

    let old_connection_end = ConnectionEnd {
        state: State::Init,
        ..Default::default()
    };

    let new_connection_end = ConnectionEnd {
        state: State::Open,
        counterparty: ConnectionCounterparty {
            connection_id: Some("connection".to_string()),
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
        proofs: Default::default(),
    };

    let old_connection_end = ConnectionEnd {
        state: State::OpenTry,
        ..Default::default()
    };

    let new_connection_end = ConnectionEnd {
        state: State::Open,
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

    let connection_end = ConnectionEnd {
        state: State::Open,
        ..Default::default()
    };

    let new_connections = IbcConnections {
        next_channel_number: 1,
        connections: vec![connection_end],
        ..Default::default()
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
        ..Default::default()
    };

    let channel = IbcChannel {
        state: State::OpenTry,
        connection_hops: vec![index_to_connection_id(0)],
        ..Default::default()
    };

    let msg = MsgChannelOpenTry {
        proof_chan_end_on_a: Default::default(),
    };

    handle_msg_channel_open_try(client, &new_connections, channel, msg).unwrap()
}

#[test]
fn test_handle_msg_channel_open_ack_success() {
    let client = TestClient::default();

    let old_channel = IbcChannel {
        state: State::Init,
        counterparty: ChannelCounterparty {
            port_id: "portid".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let new_channel = IbcChannel {
        state: State::Open,
        counterparty: ChannelCounterparty {
            channel_id: "channel-id".to_string(),
            port_id: "portid".to_string(),
        },
        ..Default::default()
    };

    let msg = MsgChannelOpenAck {
        proofs: Default::default(),
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
        },
        connection_hops: vec![index_to_connection_id(0)],
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
            182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182, 181,
            246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
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
            182, 172, 119, 152, 129, 180, 254, 5, 161, 103, 228, 19, 255, 83, 68, 105, 182, 181,
            246, 192, 109, 149, 228, 197, 35, 235, 41, 69, 216, 94, 212, 80,
        ],
    };

    let envelope = Envelope {
        msg_type: MsgType::MsgChannelOpenAck,
        content: rlp::encode(&MsgChannelOpenAck {
            proofs: Default::default(),
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
        proofs: Default::default(),
    };

    handle_msg_channel_open_confirm(client, old_channel, new_channel, msg).unwrap();
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
        proofs: Default::default(),
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
            ..Default::default()
        },
        tx_hash: None,
        status: PacketStatus::Send,
        ack: None,
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
        packet: Packet::default(),
        tx_hash: None,
        status: PacketStatus::Recv,
        ack: None,
    };
    let old_channel_args = ChannelArgs::default();
    let new_channel_args = ChannelArgs::default();
    let packet_args = PacketArgs::default();

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
            proofs: Default::default(),
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

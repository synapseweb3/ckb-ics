// These structs should only be used in CKB contracts.

// use axon_protocol::types::Bytes;
use super::Bytes;
use super::Vec;

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

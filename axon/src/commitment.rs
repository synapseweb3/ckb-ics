//! Functions for commitment paths.

use alloc::string::String;

pub fn connection_path(connection_id: &str) -> String {
    format!("connections/{connection_id}")
}

pub fn packet_commitment_path(port_id: &str, channel_id: &str, sequence: u64) -> String {
    format!("commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")
}

pub fn packet_acknowledgement_commitment_path(
    port_id: &str,
    channel_id: &str,
    sequence: u64,
) -> String {
    format!("acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")
}

pub fn channel_path(port_id: &str, channel_id: &str) -> String {
    format!("channelEnds/ports/{port_id}/channels/{channel_id}")
}

#![cfg_attr(not(test), no_std)]
#![allow(clippy::result_unit_err)]

#[macro_use]
extern crate alloc;

use alloc::string::String;
pub use alloc::vec::Vec;

/// Implement rlp::{Encodable, Decodable} for enum.
///
/// Encoding will be the same as the repr type.
///
/// The second parameter must be the same as the repr type.
macro_rules! impl_enum_rlp {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }, $type:ty) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl rlp::Encodable for $name {
            fn rlp_append(&self, s: &mut rlp::RlpStream) {
                (*self as $type).rlp_append(s);
            }
        }

        impl rlp::Decodable for $name {
            fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
                let v: $type = rlp::Decodable::decode(rlp)?;
                match v {
                    $(x if x == $name::$vname as $type => Ok($name::$vname),)*
                    _ => Err(rlp::DecoderError::Custom(concat!("invalid value for ", stringify!($name)))),
                }
            }
        }
    }
}

pub mod axon_client;
pub mod commitment;
pub mod consts;
pub mod handler;
pub mod message;
pub mod object;
pub mod proto;

use axon_tools::keccak_256;
use consts::CHANNEL_ID_PREFIX;
use object::VerifyError;

pub type U256 = Vec<u8>;
pub type Bytes = Vec<u8>;

macro_rules! try_read {
    ($buf:ident, $len:literal) => {{
        let x: &[u8; $len] = $buf.get(..$len).ok_or(())?.try_into().unwrap();
        $buf = &$buf[$len..];
        x
    }};
}

macro_rules! try_read_last {
    ($buf:ident, $len:literal) => {{
        let x: &[u8; $len] = $buf.get(..$len).ok_or(())?.try_into().unwrap();
        $buf = &$buf[$len..];
        if !$buf.is_empty() {
            return Err(());
        }
        x
    }};
}

// The args of the connection cell's script
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConnectionArgs {
    pub metadata_type_id: [u8; 32],
    pub ibc_handler_address: [u8; 20],
}

impl ConnectionArgs {
    pub fn from_slice(mut slice: &[u8]) -> Result<Self, ()> {
        Ok(Self {
            metadata_type_id: *try_read!(slice, 32),
            ibc_handler_address: *try_read_last!(slice, 20),
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        [&self.metadata_type_id[..], &self.ibc_handler_address].concat()
    }

    pub fn client_id(&self) -> String {
        hex::encode(&keccak_256(&self.encode())[..20])
    }
}

// The args of the channel cell's script
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ChannelArgs {
    pub metadata_type_id: [u8; 32],
    pub ibc_handler_address: [u8; 20],
    // For the sake of convenience, we use a bool here to describe
    // whether this channel is open. Relayer search the the unopen channel cell
    // frequently.
    pub open: bool,
    // Relayer will search the specified channel by channel id and port id
    pub channel_id: u16,
    pub port_id: [u8; 32],
}

impl ChannelArgs {
    pub fn connection(&self) -> ConnectionArgs {
        ConnectionArgs {
            metadata_type_id: self.metadata_type_id,
            ibc_handler_address: self.ibc_handler_address,
        }
    }

    pub fn from_slice(mut slice: &[u8]) -> Result<Self, ()> {
        Ok(Self {
            metadata_type_id: *try_read!(slice, 32),
            ibc_handler_address: *try_read!(slice, 20),
            open: try_read!(slice, 1) != &[0],
            channel_id: u16::from_le_bytes(*try_read!(slice, 2)),
            port_id: *try_read_last!(slice, 32),
        })
    }

    pub fn get_prefix_for_searching_unopen(&self) -> Vec<u8> {
        [
            &self.metadata_type_id[..],
            &self.ibc_handler_address,
            &if self.open { [1] } else { [0] },
        ]
        .concat()
    }

    pub fn get_prefix_for_all(&self) -> Vec<u8> {
        [&self.metadata_type_id[..], &self.ibc_handler_address].concat()
    }

    pub fn is_open(data: Vec<u8>) -> Result<bool, ()> {
        let open_byte = data.get(32 + 20 + 1).ok_or(())?;
        if *open_byte == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn to_args(self) -> Vec<u8> {
        [
            &self.metadata_type_id[..],
            &self.ibc_handler_address,
            &if self.open { [1] } else { [0] },
            &self.channel_id.to_le_bytes(),
            &self.port_id,
        ]
        .concat()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PacketArgs {
    pub channel_id: u16,
    pub port_id: [u8; 32], // mark as owner_lockhash
    pub sequence: u16,
}

impl PacketArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, VerifyError> {
        if slice.len() != 36 {
            return Err(VerifyError::WrongPacketArgs);
        }
        let channel_id = u16::from_le_bytes(slice[0..2].try_into().unwrap());
        let port_id = slice[2..34].try_into().unwrap();
        let sequence = u16::from_le_bytes(slice[34..36].try_into().unwrap());
        Ok(PacketArgs {
            channel_id,
            port_id,
            sequence,
        })
    }

    pub fn get_search_args(self, search_all: bool) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.channel_id.to_le_bytes());
        result.extend(self.port_id);
        if !search_all {
            result.extend(self.sequence.to_le_bytes());
        }
        result
    }

    pub fn get_owner(&self) -> [u8; 32] {
        self.port_id
    }

    pub fn to_args(self) -> Vec<u8> {
        self.get_search_args(false)
    }
}

pub fn connection_id(client_id: &str, connection_idx: usize) -> String {
    format!(
        "{}-{}{}",
        &client_id[..6],
        consts::CONNECTION_ID_PREFIX,
        connection_idx
    )
}

pub fn get_channel_id_str(idx: u16) -> String {
    format!("{CHANNEL_ID_PREFIX}{}", idx)
}

#[cfg(test)]
mod tests {
    use crate::ChannelArgs;

    #[test]
    fn channel_args_conversion() {
        let channel_args = ChannelArgs {
            metadata_type_id: [1; 32],
            ibc_handler_address: [7; 20],
            open: true,
            channel_id: 23,
            port_id: [2; 32],
        };
        let slice = channel_args.clone().to_args();
        let actual = ChannelArgs::from_slice(&slice).unwrap();
        assert_eq!(channel_args, actual);
    }
}

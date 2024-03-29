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
pub use axon_tools;

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
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ChannelArgs {
    pub metadata_type_id: [u8; 32],
    pub ibc_handler_address: [u8; 20],
    // For the sake of convenience, we use a bool here to describe
    // whether this channel is open. Relayer search the the unopen channel cell
    // frequently.
    pub open: bool,
    // Relayer will search the specified channel by channel id and port id
    pub channel_id: u64,
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
            channel_id: u64::from_le_bytes(*try_read!(slice, 8)),
            port_id: *try_read_last!(slice, 32),
        })
    }

    pub fn channel_id_str(&self) -> String {
        format!("{CHANNEL_ID_PREFIX}{}", self.channel_id)
    }

    pub fn port_id_str(&self) -> String {
        hex::encode(self.port_id)
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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct PacketArgs {
    pub ibc_handler_address: [u8; 20], // distinguish different packet cells with same channel and port
    pub channel_id: u64,
    pub port_id: [u8; 32], // mark as owner_lockhash
    pub sequence: u64,
}

impl PacketArgs {
    pub fn is_channel(&self, channel: &ChannelArgs) -> Result<(), VerifyError> {
        if channel.channel_id != self.channel_id || channel.port_id != self.port_id {
            Err(VerifyError::WrongPacketArgs)
        } else {
            Ok(())
        }
    }

    pub fn from_slice(mut slice: &[u8]) -> Result<Self, ()> {
        Ok(Self {
            ibc_handler_address: *try_read!(slice, 20),
            channel_id: u64::from_le_bytes(*try_read!(slice, 8)),
            port_id: *try_read!(slice, 32),
            sequence: u64::from_le_bytes(*try_read_last!(slice, 8)),
        })
    }

    pub fn get_prefix_for_all(self) -> Vec<u8> {
        self.ibc_handler_address.to_vec()
    }

    pub fn get_search_args(self, search_all: bool) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.ibc_handler_address);
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

pub trait WriteOrVerifyCommitments {
    fn write_no_commitment(&mut self) -> Result<(), VerifyError> {
        self.write_commitments::<String, Vec<u8>>([])
    }

    fn write_commitments<K, V>(
        &mut self,
        kvs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<(), VerifyError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;
}

impl<T> WriteOrVerifyCommitments for &mut T
where
    T: WriteOrVerifyCommitments,
{
    fn write_commitments<K, V>(
        &mut self,
        kvs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<(), VerifyError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        T::write_commitments(self, kvs)
    }
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
        let slice = channel_args.to_args();
        let actual = ChannelArgs::from_slice(&slice).unwrap();
        assert_eq!(channel_args, actual);
    }
}

#![cfg_attr(not(test), no_std)]
#![allow(clippy::result_unit_err)]

#[macro_use]
extern crate alloc;

use core::str::FromStr;

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

pub mod commitment;
pub mod consts;
pub mod handler;
pub mod message;
pub mod object;
pub mod proto;
pub mod verify_mpt;

use consts::CHANNEL_ID_PREFIX;
use ethereum_types::H256;
use object::VerifyError;
use rlp::{Encodable, RlpStream};

pub type U256 = Vec<u8>;
pub type Bytes = Vec<u8>;

// The args of the connection cell's script
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConnectionArgs {
    pub client_id: [u8; 32],
}

impl ConnectionArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 32 {
            return Err(());
        }
        Ok(ConnectionArgs {
            client_id: slice[0..32].try_into().unwrap(),
        })
    }

    pub fn get_client_id(slice: &[u8]) -> &[u8] {
        &slice[0..32]
    }
}

// The args of the channel cell's script
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ChannelArgs {
    pub client_id: [u8; 32],
    // For the sake of convenience, we use a bool here to describe
    // whether this channel is open. Relayer search the the unopen channel cell
    // frequently.
    pub open: bool,
    // Relayer will search the specified channel by channel id and port id
    pub channel_id: u16,
    pub port_id: [u8; 32],
}

impl ChannelArgs {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 67 {
            return Err(());
        }
        let client_id: [u8; 32] = slice[0..32].try_into().unwrap();
        let open = slice.get(32).unwrap() > &0;
        let channel_id = u16::from_le_bytes(slice[33..35].try_into().unwrap());
        let port_id: [u8; 32] = slice[35..67].try_into().unwrap();
        Ok(ChannelArgs {
            client_id,
            open,
            channel_id,
            port_id,
        })
    }

    pub fn get_prefix_for_searching_unopen(&self) -> Vec<u8> {
        let mut result = self.client_id.to_vec();
        let open: u8 = if self.open { 1 } else { 0 };
        result.push(open);
        result
    }

    pub fn get_prefix_for_all(&self) -> Vec<u8> {
        self.client_id.to_vec()
    }

    pub fn is_open(data: Vec<u8>) -> Result<bool, ()> {
        let open_byte = data.get(33).ok_or(())?;
        if *open_byte == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn to_args(self) -> Vec<u8> {
        let mut result = self.get_prefix_for_searching_unopen();
        result.extend(self.channel_id.to_le_bytes());
        result.extend(self.port_id);
        result
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

pub fn rlp_opt<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

pub fn rlp_opt_list<T: Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        // Choice of `u8` type here is arbitrary as all empty lists are encoded the same.
        rlp.append_list::<u8, u8>(&[]);
    }
}

pub fn convert_byte32_to_hex(bytes32: &[u8; 32]) -> String {
    format!("{:x}", H256::from(bytes32))
}

pub fn convert_hex_to_client_id(s: &str) -> Result<[u8; 32], VerifyError> {
    Ok(H256::from_str(s)
        .map_err(|_| VerifyError::WrongClient)?
        .into())
}

pub fn convert_hex_to_port_id(s: &str) -> Result<[u8; 32], VerifyError> {
    Ok(H256::from_str(s)
        .map_err(|_| VerifyError::WrongPortId)?
        .into())
}

// ConnectionId example: xxxxxx-connection-0, `xxxxxx` is the prefix of hex encoded ClientId
pub fn convert_connection_id_to_index(connection_id: &str) -> Result<usize, VerifyError> {
    let index_str = connection_id
        .split('-')
        .last()
        .ok_or(VerifyError::WrongConnectionId)?;
    let index = usize::from_str(index_str).map_err(|_| VerifyError::WrongConnectionId)?;
    Ok(index)
}

pub fn get_channel_id_str(idx: u16) -> String {
    format!("{CHANNEL_ID_PREFIX}{}", idx)
}

#[cfg(test)]
mod tests {
    use crate::ChannelArgs;

    use super::{convert_byte32_to_hex, convert_hex_to_client_id};

    #[test]
    fn client_id_to_string() {
        let actual = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let s = convert_byte32_to_hex(&actual);
        let r = convert_hex_to_client_id(&s).unwrap();
        assert_eq!(actual, r);
    }

    #[test]
    fn channel_args_conversion() {
        let channel_args = ChannelArgs {
            client_id: [1; 32],
            open: true,
            channel_id: 23,
            port_id: [2; 32],
        };
        let slice = channel_args.clone().to_args();
        let actual = ChannelArgs::from_slice(&slice).unwrap();
        assert_eq!(channel_args, actual);
    }
}

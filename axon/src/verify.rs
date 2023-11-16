use alloc::{borrow::ToOwned as _, vec::Vec};
use core::cmp;
use ethereum_types::{H256, U256};
use rlp_derive::{RlpDecodable, RlpEncodable};

use rlp::decode_list;
use tiny_keccak::{Hasher, Keccak};

use crate::object::VerifyError;

pub fn keccak256(slice: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(slice);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

/// Verify MPT proof.
///
/// There's some limitations to this function: it won't handle “inner nodes” or
/// values at branch node. But it should work for state and storage tries.
fn verify_mpt(
    proof: &[Vec<u8>],
    root: &[u8],
    path: &[u8],
    value: &[u8],
) -> Result<(), VerifyError> {
    let mut expected_hash = root.to_owned();
    let mut path_offset = 0;

    for (i, node) in proof.iter().enumerate() {
        if expected_hash != keccak256(node).to_vec() {
            return Err(VerifyError::Mpt);
        }

        let node_list: Vec<Vec<u8>> = decode_list(node);

        if node_list.len() == 17 {
            let nibble = get_nibble(path, path_offset);
            expected_hash = node_list[nibble as usize].clone();

            // Limitation: the case that node_list[nibble] is already a node is not handled.

            path_offset += 1;
        } else if node_list.len() == 2 {
            if i == proof.len() - 1 {
                // will use extension node value as value?
                if node_list[1] == value {
                    return if paths_match(
                        &node_list[0],
                        skip_length(&node_list[0]),
                        path,
                        path_offset,
                    ) {
                        Ok(())
                    } else {
                        Err(VerifyError::Mpt)
                    };
                }
            } else {
                let node_path = &node_list[0];
                let prefix_length = shared_prefix_length(path, path_offset, node_path);
                if prefix_length < node_path.len() * 2 - skip_length(node_path) {
                    return Err(VerifyError::Mpt);
                }
                path_offset += prefix_length;
                expected_hash = node_list[1].clone();
            }
        } else {
            return Err(VerifyError::Mpt);
        }
    }

    Err(VerifyError::Mpt)
}

#[derive(RlpDecodable, RlpEncodable)]
struct Account {
    pub nonce: U256,
    pub balance: U256,
    pub storage_root: H256,
    pub code_hash: H256,
}

#[allow(clippy::too_many_arguments)]
pub fn verify_commitment(
    state_root: &[u8],
    address: &[u8],
    account_bytes: &[u8],
    account_proof: &[Vec<u8>],
    path: &[u8],
    value: &[u8],
    storage_proof: &[Vec<u8>],
    secure_trie: bool,
) -> Result<(), VerifyError> {
    let address_hash = keccak256(address);
    verify_mpt(
        account_proof,
        state_root,
        if secure_trie { &address_hash } else { address },
        account_bytes,
    )?;
    let account: Account = rlp::decode(account_bytes).map_err(|_| VerifyError::Mpt)?;

    // Commitment mapping key is keccak256(commitment path);
    let commitment_key = keccak256(path);
    let slot = {
        let mut h = Keccak::v256();
        h.update(&commitment_key);
        h.update(&[0u8; 32]);
        let mut o = [0u8; 32];
        h.finalize(&mut o);
        o
    };
    let slot_hash = keccak256(&slot);
    // Slot value (commitment mapping value) is keccak256(commitment value)
    // Trie value is rlp(slot value)
    let trie_value = rlp::encode(&keccak256(value).to_vec());
    verify_mpt(
        storage_proof,
        account.storage_root.as_bytes(),
        if secure_trie { &slot_hash } else { &slot },
        &trie_value,
    )?;

    Ok(())
}

fn paths_match(p1: &[u8], s1: usize, p2: &[u8], s2: usize) -> bool {
    let len1 = p1.len() * 2 - s1;
    let len2 = p2.len() * 2 - s2;
    if len1 != len2 {
        return false;
    }
    for offset in 0..len1 {
        let n1 = get_nibble(p1, s1 + offset);
        let n2 = get_nibble(p2, s2 + offset);
        if n1 != n2 {
            return false;
        }
    }
    true
}

fn shared_prefix_length(path: &[u8], path_offset: usize, node_path: &[u8]) -> usize {
    let skip_length = skip_length(node_path);

    let len = cmp::min(
        node_path.len() * 2 - skip_length,
        path.len() * 2 - path_offset,
    );
    let mut prefix_len = 0;

    for i in 0..len {
        let path_nibble = get_nibble(path, i + path_offset);
        let node_path_nibble = get_nibble(node_path, i + skip_length);

        if path_nibble == node_path_nibble {
            prefix_len += 1;
        } else {
            break;
        }
    }

    prefix_len
}

fn skip_length(node: &[u8]) -> usize {
    if node.is_empty() {
        return 0;
    }

    let nibble = get_nibble(node, 0);
    match nibble {
        0 => 2,
        1 => 1,
        2 => 2,
        3 => 1,
        _ => 0,
    }
}

fn get_nibble(path: &[u8], offset: usize) -> u8 {
    let byte = path[offset / 2];
    if offset % 2 == 0 {
        byte >> 4
    } else {
        byte & 0xF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_mpt() {
        use std::sync::Arc;

        use hasher::HasherKeccak;

        use cita_trie::MemoryDB;
        use cita_trie::{PatriciaTrie, Trie};

        let memdb = Arc::new(MemoryDB::new(false));
        let hasher = Arc::new(HasherKeccak::new());

        let key = keccak256(b"test-key").to_vec();
        let value = keccak256(b"test-value").to_vec();

        let key1 = vec![0xff, 32];
        let value1 = keccak256(b"test-value1").to_vec();

        let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));
        trie.insert(key.clone(), value.clone()).unwrap();
        let root = trie.root().unwrap();

        let proof = trie.get_proof(&key).unwrap();
        verify_mpt(&proof, &root, &key, &value).unwrap();

        trie.insert(key1.clone(), value1.clone()).unwrap();
        let root = trie.root().unwrap();

        let proof = trie.get_proof(&key1).unwrap();
        verify_mpt(&proof, &root, &key1, &value1).unwrap();
    }

    #[test]
    fn test_verify_commitment() {
        // Test with eth_getProof result from ganache.
        let account_bytes = rlp::encode(&Account {
            balance: U256::zero(),
            code_hash: "0xc09715ef7e413bd06144c8c6dd476b1901eb2e29c6826f3c7a2b2e1834887c0a"
                .parse()
                .unwrap(),
            storage_root: "0x6eefedf8b895defe8b8b32522a7746b9c388b67cc710ec0aaa45c2305fb9cedf"
                .parse()
                .unwrap(),
            nonce: U256::one(),
        });
        let account_proof = ["0xf90211a021ff4e4e9ef3e4206823799dc4181bef914f590200c1ba58d8b517ec2ec902e9a0fa22b58ff107979c4ecb0d18dcc4c9c4a21d82a5f49e18bc84a6dcc1f43c6cffa05f401c8ccf28c88c795c8dc3194c10de0364f121365be2f81c89ae9a4466ac4aa07d55b967ed900e13b3dd0794dd7284d6a84b6aeb0da2d0c22ae3c1d46206e51ba0438cfa73f409f90f93f88859ba2a249b158341547c715c9ea05863f25e0d872fa02245741cb87ce55bef07c0dded7d132b406a961a3876d3126a5c770902097551a04c5a9fe5ecc0f2400e2ea6eeae97bbbed17e40e2c95bc60044f4f8bd7d502049a07f0773a09800a67a39a15b889e2f777caffab9cbd7d44e7749f92ea78b4ab188a050e2407b752610686f21766779ef5184561d1280387ad40e190429fccc9dc1bca0ea55baaf73e67e8d7bf88847ef7ed8d11cfc1731174555a9fb9092e704e4b9d6a05ee161597380346a6cd11d71f0fa58d5ddb480a528e85e70bdb55904af8253b3a051e8cb9a583463217423146de2503fcba6be0c21cd624456bf830a6f8789e93ea02d2049a1a43b4c1409793f8fd21181fadc3f5d645909998876227f7f3d4f8fa6a0a9dc17c0c91876c9183b348321bdf025e2f6f0e087c6d1b7941635f1db314226a0941655b6277d7ae3573ad038f87bd135fb7c385ab2e07b214d3e6d6e261c8b65a0bc634e3ad0d3010f8dfbfacd2e10198e7c814d30d40e07987b24c36aea3c428f80","0xf8b1a0df5900ec8abdb023b4ededf5ca973bb8fdffeaf4fff45bdee6821e2177fb9be3a0996dbe53744140b7f467c72ef93d107539d783922fc78c3e9dc0ec1bd05788db8080a0dc3910d1aea67675f479f2cd95f6f15bb02e8935805f1cce951bbc9134901f4580a0cf56a435fe6b8cc75faf566d7e9767d219650723dad3eb7aa3f4743feaf5e4b880a0d78ebfe5f7c2ea4bb7a89bf465c7a308386a474fc3176b10ef039ab52747cc728080808080808080","0xf8518080808080808080808080a04d046e6057422dde202a8394ed7f71b4c92b776c2eb51d976ca71ecf41db1b7e808080a036698dc604cca461696b339fabf922f3e5898571f81bf3bfe96d897e21f8a99880","0xf8689f364b9c7b69139bea764e6a6ed3394a2fb0c3affd66fe531a68eaeca9cfe297b846f8440180a06eefedf8b895defe8b8b32522a7746b9c388b67cc710ec0aaa45c2305fb9cedfa0c09715ef7e413bd06144c8c6dd476b1901eb2e29c6826f3c7a2b2e1834887c0a"];
        verify_commitment(
            &hex::decode("b05361ee4e2433d107e7bbd512d906b0b9cb9b7122636dff7fdb74f78c16f551").unwrap(),
            &hex::decode("1C6e2aAcAf61711A2dD74d18363766482d93CF84").unwrap(),
            &account_bytes,
            &Vec::from_iter(account_proof.iter().map(|p| hex::decode(&p[2..]).unwrap())),
            b"abc",
            b"def",
            &[hex::decode("f844a1201663f081233a2f6d2dc07d9801a0a4bd2608df182782575baee276e196bad7aea1a034607c9bbfeb9c23509680f04363f298fdb0b5f9abe327304ecd1daca08cda9c").unwrap()],
            true,
        )
        .unwrap();
    }
}

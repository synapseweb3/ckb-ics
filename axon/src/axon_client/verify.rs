use bytes::Bytes;
use ethereum_types::U256;
use rlp::Rlp;

use super::keccak256;
use crate::object::VerifyError;

// Root hash of an empty trie.
// 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421
const EMPTY_ROOT: [u8; 32] = [
    86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153,
    108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33,
];

/// Get value by path from MPT root and proof nodes.
///
/// Returns an empty slice if the path can be proven not exist.
pub fn verify_mpt<'b, T: AsRef<[u8]>>(
    root: &[u8],
    path: &[u8],
    proof: &'b [T],
) -> Result<&'b [u8], VerifyError> {
    if root == EMPTY_ROOT {
        return Ok(&[]);
    }

    let mut proof_iter = proof.iter();

    let mut get_node_by_hash = |h: &[u8]| {
        let next = proof_iter.next().ok_or(VerifyError::Mpt)?;
        if h == keccak256(next.as_ref()) {
            Ok(Rlp::new(next.as_ref()))
        } else {
            Err(VerifyError::Mpt)
        }
    };

    let path_nibbles = path.len() * 2;
    let mut node = get_node_by_hash(root)?;
    let mut path_offset = 0;

    // Loop invariant: we have traversed path_nibbles[..path_offset], we are at
    // MPT node `node`.
    loop {
        let node_list_len = node.item_count()?;
        if node_list_len == 17 {
            if path_offset == path_nibbles {
                // Return branch value.
                return Ok(node.at(16)?.data()?);
            }

            // Branch by next nibble.
            let nibble = get_nibble(path, path_offset) as usize;
            let next = node.at(nibble)?;
            path_offset += 1;
            if next.is_empty() {
                return Ok(&[]);
            } else if next.is_data() {
                // Get next node by hash.
                node = get_node_by_hash(next.data()?)?;
            } else if next.is_list() {
                // Nested node.
                node = next;
            } else {
                // Invalid next node.
                return Err(VerifyError::Mpt);
            }
        } else if node_list_len == 2 {
            let node_path = node.at(0)?.data()?;
            let node_value = node.at(1)?;
            let (skip, is_leaf) = skip_length(node_path)?;
            let node_nibbles = node_path.len() * 2 - skip;
            let common_prefix_nibbles = nibbles(path, path_offset)
                .zip(nibbles(node_path, skip))
                .take_while(|(n1, n2)| n1 == n2)
                .count();
            path_offset += common_prefix_nibbles;
            if common_prefix_nibbles < node_nibbles {
                // Path doesn't exist in MPT.
                return Ok(&[]);
            } else if is_leaf {
                if path_offset == path_nibbles {
                    // Leaf exact match.
                    return Ok(node_value.data()?);
                } else {
                    // Path doesn't exist in MPT.
                    return Ok(&[]);
                }
            } else {
                // Follow extension node.
                node = if node_value.is_data() {
                    // Next node by hash.
                    get_node_by_hash(node_value.data()?)?
                } else {
                    // Nested node.
                    node_value
                };
            }
        } else {
            return Err(VerifyError::Mpt);
        }
    }
}

// Returns (nibble skip length, is_leaf).
fn skip_length(node: &[u8]) -> Result<(usize, bool), VerifyError> {
    if node.is_empty() {
        return Err(VerifyError::Mpt);
    }

    let nibble = get_nibble(node, 0);
    match nibble {
        0 => Ok((2, false)),
        1 => Ok((1, false)),
        2 => Ok((2, true)),
        3 => Ok((1, true)),
        _ => Err(VerifyError::Mpt),
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

fn nibbles(buf: &[u8], offset: usize) -> impl Iterator<Item = u8> + '_ {
    (offset..buf.len() * 2).map(|o| get_nibble(buf, o))
}

#[allow(clippy::too_many_arguments)]
pub fn verify_account_and_storage<T1: AsRef<[u8]>, T2: AsRef<[u8]>>(
    state_root: &[u8],
    address: &[u8],
    account_proof: &[T1],
    slot: [u8; 32],
    slot_value: [u8; 32],
    storage_proof: &[T2],
) -> Result<(), VerifyError> {
    let address_hash = keccak256(address);
    let account_bytes = verify_mpt(state_root, &address_hash, account_proof)?;
    // Account is (nonce, balance, storage_root, code_hash), so storage root is at index 2.
    let storage_root = Rlp::new(account_bytes).at(2)?.data()?;

    let slot_hash = keccak256(&slot);
    let expected_trie_value = if slot_value != [0; 32] {
        rlp::encode(&U256::from(&slot_value)).freeze()
    } else {
        Bytes::new()
    };
    let trie_value = verify_mpt(storage_root, &slot_hash, storage_proof)?;

    if trie_value != expected_trie_value {
        return Err(VerifyError::Mpt);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::axon_client::commitment_slot;

    fn test_verify_mpt(kvs: Vec<(Vec<u8>, Vec<u8>)>, pks: Vec<Vec<u8>>) {
        use std::sync::Arc;

        use hasher::HasherKeccak;

        use cita_trie::MemoryDB;
        use cita_trie::{PatriciaTrie, Trie};

        let memdb = Arc::new(MemoryDB::new(false));
        let hasher = Arc::new(HasherKeccak::new());

        let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));

        for (k, v) in kvs {
            trie.insert(k, v.to_vec()).unwrap();
        }
        let root = trie.root().unwrap();

        for pk in pks {
            let proof = trie.get_proof(&pk).unwrap();
            let v = verify_mpt(&root, &pk, &proof).unwrap();
            assert_eq!(v, trie.get(&pk).unwrap().unwrap_or_default());
        }
    }

    proptest!(
        #[test]
        fn prop_test_verify_mpt(kvs: Vec<(Vec<u8>, Vec<u8>)>, pks: Vec<Vec<u8>>) {
            test_verify_mpt(kvs, pks)
        }
    );

    #[test]
    fn test_verify_commitment() {
        // Test with eth_getProof result from ganache.
        let account_proof = ["0xf90211a021ff4e4e9ef3e4206823799dc4181bef914f590200c1ba58d8b517ec2ec902e9a0fa22b58ff107979c4ecb0d18dcc4c9c4a21d82a5f49e18bc84a6dcc1f43c6cffa05f401c8ccf28c88c795c8dc3194c10de0364f121365be2f81c89ae9a4466ac4aa07d55b967ed900e13b3dd0794dd7284d6a84b6aeb0da2d0c22ae3c1d46206e51ba0438cfa73f409f90f93f88859ba2a249b158341547c715c9ea05863f25e0d872fa02245741cb87ce55bef07c0dded7d132b406a961a3876d3126a5c770902097551a04c5a9fe5ecc0f2400e2ea6eeae97bbbed17e40e2c95bc60044f4f8bd7d502049a07f0773a09800a67a39a15b889e2f777caffab9cbd7d44e7749f92ea78b4ab188a050e2407b752610686f21766779ef5184561d1280387ad40e190429fccc9dc1bca0ea55baaf73e67e8d7bf88847ef7ed8d11cfc1731174555a9fb9092e704e4b9d6a05ee161597380346a6cd11d71f0fa58d5ddb480a528e85e70bdb55904af8253b3a051e8cb9a583463217423146de2503fcba6be0c21cd624456bf830a6f8789e93ea02d2049a1a43b4c1409793f8fd21181fadc3f5d645909998876227f7f3d4f8fa6a0a9dc17c0c91876c9183b348321bdf025e2f6f0e087c6d1b7941635f1db314226a0941655b6277d7ae3573ad038f87bd135fb7c385ab2e07b214d3e6d6e261c8b65a0bc634e3ad0d3010f8dfbfacd2e10198e7c814d30d40e07987b24c36aea3c428f80","0xf8b1a0df5900ec8abdb023b4ededf5ca973bb8fdffeaf4fff45bdee6821e2177fb9be3a0996dbe53744140b7f467c72ef93d107539d783922fc78c3e9dc0ec1bd05788db8080a0dc3910d1aea67675f479f2cd95f6f15bb02e8935805f1cce951bbc9134901f4580a0cf56a435fe6b8cc75faf566d7e9767d219650723dad3eb7aa3f4743feaf5e4b880a0d78ebfe5f7c2ea4bb7a89bf465c7a308386a474fc3176b10ef039ab52747cc728080808080808080","0xf8518080808080808080808080a04d046e6057422dde202a8394ed7f71b4c92b776c2eb51d976ca71ecf41db1b7e808080a036698dc604cca461696b339fabf922f3e5898571f81bf3bfe96d897e21f8a99880","0xf8689f364b9c7b69139bea764e6a6ed3394a2fb0c3affd66fe531a68eaeca9cfe297b846f8440180a06eefedf8b895defe8b8b32522a7746b9c388b67cc710ec0aaa45c2305fb9cedfa0c09715ef7e413bd06144c8c6dd476b1901eb2e29c6826f3c7a2b2e1834887c0a"];
        verify_account_and_storage(
            &hex::decode("b05361ee4e2433d107e7bbd512d906b0b9cb9b7122636dff7fdb74f78c16f551").unwrap(),
            &hex::decode("1C6e2aAcAf61711A2dD74d18363766482d93CF84").unwrap(),
            &Vec::from_iter(account_proof.iter().map(|p| hex::decode(&p[2..]).unwrap())),
            commitment_slot(b"abc"),
            keccak256(b"def"),
            &[hex::decode("f844a1201663f081233a2f6d2dc07d9801a0a4bd2608df182782575baee276e196bad7aea1a034607c9bbfeb9c23509680f04363f298fdb0b5f9abe327304ecd1daca08cda9c").unwrap()],
        )
        .unwrap();
    }
}

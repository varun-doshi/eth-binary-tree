use alloy_primitives::{aliases::B256, Address};
use blake3::Hasher;
use std::convert::TryInto;

type Address32 = B256;
type Bytes32 = B256;

pub const BASIC_DATA_LEAF_KEY: u8 = 0;
pub const CODE_HASH_LEAF_KEY: u8 = 1;
pub const HEADER_STORAGE_OFFSET: u64 = 64;
pub const CODE_OFFSET: u64 = 128;
pub const STEM_SUBTREE_WIDTH: u64 = 256;
pub const MAIN_STORAGE_OFFSET: u64 = 256;

pub const PUSH_OFFSET: u8 = 95;
pub const PUSH1: u8 = PUSH_OFFSET + 1;
pub const PUSH32: u8 = PUSH_OFFSET + 32;

pub fn old_style_address_to_address32(address: &Address) -> Address32 {
    let mut address32 = [0u8; 32];
    address32[12..].copy_from_slice(address.as_slice());
    Address32::from(address32)
}

pub fn tree_hash(inp: &[u8]) -> Bytes32 {
    let mut hasher = Hasher::new();
    hasher.update(inp);
    hasher
        .finalize()
        .as_bytes()
        .try_into()
        .expect("Hash should be 32 bytes")
}

pub fn get_tree_key(address: &Address32, tree_index: u64, sub_index: u8) -> [u8; 32] {
    let mut key_input = vec![];
    key_input.extend_from_slice(address.as_slice());
    key_input.extend_from_slice(&tree_index.to_le_bytes());
    let hash = tree_hash(&key_input);

    let mut key = [0u8; 32];
    key[..31].copy_from_slice(&hash[..31]);
    key[31] = sub_index;
    key
}

pub fn get_tree_key_for_basic_data(address: &Address32) -> [u8; 32] {
    get_tree_key(address, 0, BASIC_DATA_LEAF_KEY)
}

pub fn get_tree_key_for_code_hash(address: &Address32) -> [u8; 32] {
    get_tree_key(address, 0, CODE_HASH_LEAF_KEY)
}

pub fn get_tree_key_for_storage_slot(address: &Address32, storage_key: u64) -> [u8; 32] {
    let pos = if storage_key < (CODE_OFFSET - HEADER_STORAGE_OFFSET) {
        HEADER_STORAGE_OFFSET + storage_key
    } else {
        MAIN_STORAGE_OFFSET + storage_key
    };

    get_tree_key(
        address,
        pos / STEM_SUBTREE_WIDTH,
        (pos % STEM_SUBTREE_WIDTH) as u8,
    )
}

pub fn get_tree_key_for_code_chunk(address: &Address32, chunk_id: u64) -> [u8; 32] {
    get_tree_key(
        address,
        (CODE_OFFSET + chunk_id) / STEM_SUBTREE_WIDTH,
        ((CODE_OFFSET + chunk_id) % STEM_SUBTREE_WIDTH) as u8,
    )
}

pub fn chunkify_code(code: &[u8]) -> Vec<Bytes32> {
    let mut padded_code = code.to_vec();
    if code.len() % 31 != 0 {
        padded_code.extend(vec![0; 31 - (code.len() % 31)]);
    }

    let mut bytes_to_exec_data = vec![0u8; padded_code.len() + 32];
    let mut pos = 0;

    while pos < padded_code.len() {
        let pushdata_bytes = if (PUSH1..=PUSH32).contains(&padded_code[pos]) {
            padded_code[pos] - PUSH_OFFSET
        } else {
            0
        };

        pos += 1;
        for x in 0..pushdata_bytes {
            if let Some(val) = bytes_to_exec_data.get_mut(pos + x as usize) {
                *val = pushdata_bytes - x;
            }
        }
        pos += pushdata_bytes as usize;
    }

    padded_code
        .chunks(31)
        .enumerate()
        .map(|(i, chunk)| {
            let exec_data_byte = bytes_to_exec_data[i * 31] as u8;
            let mut array = [0u8; 32];
            array[0] = exec_data_byte;
            array[1..(1 + chunk.len())].copy_from_slice(chunk);

            B256::from(array)
        })
        .collect()
}

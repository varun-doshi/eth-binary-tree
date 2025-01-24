#[cfg(test)]
mod tests {
    use alloy_primitives::B256;
    use eth_binary_tree::embedding::*;

    fn address32_example() -> B256 {
        B256::from([0x42; 32])
    }

    #[test]
    fn test_get_tree_key_for_basic_data() {
        let address = address32_example();
        let result = get_tree_key_for_basic_data(&address);
        assert_eq!(result.len(), 32);
        assert_eq!(result[31], BASIC_DATA_LEAF_KEY);
    }

    #[test]
    fn test_get_tree_key_for_code_hash() {
        let address = address32_example();
        let result = get_tree_key_for_code_hash(&address);
        assert_eq!(result.len(), 32);
        assert_eq!(result[31], CODE_HASH_LEAF_KEY);
    }

    #[test]
    fn test_get_tree_key_for_storage_slot_below_threshold() {
        let address = address32_example();
        let header_keys: Vec<B256> = (0..HEADER_STORAGE_OFFSET)
            .map(|storage_key| {
                let key = get_tree_key_for_storage_slot(&address, storage_key);
                B256::from(key)
            })
            .collect();

        let stems: Vec<_> = header_keys.iter().map(|key| &key[..31]).collect();
        let unique_stems: std::collections::HashSet<_> = stems.iter().collect();
        assert_eq!(unique_stems.len(), 1);

        for (i, key) in header_keys.iter().enumerate() {
            assert_eq!(key[31], i as u8 + 64);
        }

        let storage_slot = 64;
        let outside_key = get_tree_key_for_storage_slot(&address, storage_slot);
        assert_ne!(header_keys[0], outside_key);
    }

    #[test]
    fn test_get_tree_key_for_code_chunk() {
        let address = address32_example();
        let code_keys: Vec<B256> = (0..128)
            .map(|chunk_id| {
                let key = get_tree_key_for_code_chunk(&address, chunk_id);
                B256::from(key)
            })
            .collect();

        let stems: Vec<_> = code_keys.iter().map(|key| &key[..31]).collect();
        let unique_stems: std::collections::HashSet<_> = stems.iter().collect();
        assert_eq!(unique_stems.len(), 1);

        for (i, key) in code_keys.iter().enumerate() {
            assert_eq!(key[31], i as u8 + 128);
        }

        let chunk_id = 256;
        let outside_key = get_tree_key_for_code_chunk(&address, chunk_id);
        assert_ne!(code_keys[0], outside_key);
    }
}

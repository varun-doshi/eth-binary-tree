#[cfg(test)]
mod tests {
    use eth_binary_tree::tree::{BinaryTree, InternalNode, TreeNode};
    use hex;

    fn get_height(node: Option<&TreeNode>) -> usize {
        match node {
            None => 0,
            Some(TreeNode::Stem(_)) => 1,
            Some(TreeNode::Internal(internal)) => {
                1 + usize::max(
                    get_height(internal.left.as_deref()),
                    get_height(internal.right.as_deref()),
                )
            }
        }
    }

    #[test]
    fn test_single_entry() {
        let mut tree = BinaryTree::new();
        tree.insert([0u8; 32], [1u8; 32]);
        assert_eq!(get_height(tree.root.as_ref()), 1);
        assert_eq!(
            hex::encode(tree.merkelize()),
            "694545468677064fd833cddc8455762fe6b21c6cabe2fc172529e0f573181cd5"
        );
    }

    #[test]
    fn test_two_entries_diff_first_bit() {
        let mut tree = BinaryTree::new();
        tree.insert([0u8; 32], [1u8; 32]);
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[0] = 0x80;
                key
            },
            [2u8; 32],
        );
        assert_eq!(get_height(tree.root.as_ref()), 2);
        assert_eq!(
            hex::encode(tree.merkelize()),
            "85fc622076752a6fcda2c886c18058d639066a83473d9684704b5a29455ed2ed"
        );
    }

    #[test]
    fn test_one_stem_colocated_values() {
        let mut tree = BinaryTree::new();
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 3;
                key
            },
            [1u8; 32],
        );
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 4;
                key
            },
            [2u8; 32],
        );
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 9;
                key
            },
            [3u8; 32],
        );
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 255;
                key
            },
            [4u8; 32],
        );

        assert_eq!(get_height(tree.root.as_ref()), 1);
    }

    #[test]
    fn test_two_stem_colocated_values() {
        let mut tree = BinaryTree::new();
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 3;
                key
            },
            [1u8; 32],
        );
        tree.insert(
            {
                let mut key = [0u8; 32];
                key[31] = 4;
                key
            },
            [2u8; 32],
        );
        tree.insert(
            {
                let mut key = [0x80u8; 32];
                key[31] = 3;
                key
            },
            [1u8; 32],
        );
        tree.insert(
            {
                let mut key = [0x80u8; 32];
                key[31] = 4;
                key
            },
            [2u8; 32],
        );

        assert_eq!(get_height(tree.root.as_ref()), 2);
    }

    #[test]
    fn test_two_keys_match_first_42_bits() {
        let mut tree = BinaryTree::new();
        let mut key1 = [0u8; 32];
        key1[5..32].copy_from_slice(&[0xC0u8; 27]);

        let mut key2 = [0u8; 32];
        key2[5] = 0xE0;
        key2[6..32].copy_from_slice(&[0u8; 26]);
        tree.insert(key1, [1u8; 32]);
        tree.insert(key2, [2u8; 32]);
        assert_eq!(get_height(tree.root.as_ref()), 1 + 42 + 1);
    }

    #[test]
    fn test_insert_duplicate_key() {
        let mut tree = BinaryTree::new();
        tree.insert([1u8; 32], [1u8; 32]);
        tree.insert([1u8; 32], [2u8; 32]);
        assert_eq!(get_height(tree.root.as_ref()), 1);
        if let Some(TreeNode::Stem(stem)) = tree.root.as_ref() {
            assert_eq!(stem.values[1], Some([2u8; 32].to_vec()));
        }
    }

    #[test]
    fn test_large_number_of_entries() {
        let mut tree = BinaryTree::new();
        for i in 0..(1 << 8) {
            let mut key = [0u8; 32];
            key[0] = i as u8;
            tree.insert(key, [0xFFu8; 32]);
        }
        assert_eq!(get_height(tree.root.as_ref()), 1 + 8);
    }

    #[test]
    fn test_merkleize_multiple_entries() {
        let mut tree = BinaryTree::new();

        let keys = vec![
            [0u8; 32],
            {
                let mut key = [0u8; 32];
                key[0] = 0x80;
                key
            },
            {
                let mut key = [0u8; 32];
                key[0] = 0x01;
                key
            },
            {
                let mut key = [0u8; 32];
                key[0] = 0x81;
                key
            },
        ];

        for (i, key) in keys.iter().enumerate() {
            let mut value = [0u8; 32];
            value[0] = (i + 1) as u8;
            tree.insert(*key, value);
        }
        let got = tree.merkelize();

        let expected = "e93c209026b8b00d76062638102ece415028bd104e1d892d5399375a323f2218";

        assert_eq!(hex::encode(got), expected);
    }
}

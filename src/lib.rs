use blake3::Hasher;

#[derive(Debug, Clone)]
pub struct StemNode {
    stem: [u8; 31],
    pub values: [Option<Vec<u8>>; 256],
}

impl StemNode {
    pub fn new(stem: [u8; 31]) -> Self {
        Self {
            stem,
            values: [const { None }; 256],
        }
    }

    pub fn set_value(&mut self, index: usize, value: Vec<u8>) {
        self.values[index] = Some(value);
    }
}

#[derive(Debug, Clone)]
pub struct InternalNode {
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl InternalNode {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TreeNode {
    Stem(StemNode),
    Internal(InternalNode),
}

#[derive(Debug)]
pub struct BinaryTree {
    pub root: Option<TreeNode>,
}

impl BinaryTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, key: [u8; 32], value: [u8; 32]) {
        let stem: [u8; 31] = key[..31]
            .try_into()
            .expect("Failed to convert slice to array of size 31");
        let subindex = key[31] as usize;

        if self.root.is_none() {
            let mut node = StemNode::new(stem.try_into().unwrap());
            node.set_value(subindex, value.to_vec());
            self.root = Some(TreeNode::Stem(node));
            return;
        }

        let root = self.root.take();
        self.root = Some(self.insert_rec(root, stem, subindex, value.to_vec(), 0));
    }

    fn insert_rec(
        &self,
        node: Option<TreeNode>,
        stem: [u8; 31],
        subindex: usize,
        value: Vec<u8>,
        depth: usize,
    ) -> TreeNode {
        assert!(depth < 248, "depth must be less than 248");

        if node.is_none() {
            let mut new_node = StemNode::new(stem);
            new_node.set_value(subindex, value);
            return TreeNode::Stem(new_node);
        }

        match node.unwrap() {
            TreeNode::Stem(mut leaf) => {
                if leaf.stem == stem {
                    leaf.set_value(subindex, value);
                    return TreeNode::Stem(leaf);
                }
                let existing_stem_bits = self.bytes_to_bits(&leaf.stem);
                self.split_leaf(leaf, stem, subindex, value, existing_stem_bits, depth)
            }
            TreeNode::Internal(mut internal) => {
                let stem_bits = self.bytes_to_bits(&stem);
                let bit = stem_bits[depth];
                if bit == 0 {
                    internal.left = Some(Box::new(self.insert_rec(
                        internal.left.map(|node| *node),
                        stem,
                        subindex,
                        value,
                        depth + 1,
                    )));
                } else {
                    internal.right = Some(Box::new(self.insert_rec(
                        internal.right.map(|node| *node),
                        stem,
                        subindex,
                        value,
                        depth + 1,
                    )));
                }
                TreeNode::Internal(internal)
            }
        }
    }

    fn split_leaf(
        &self,
        leaf: StemNode,
        stem: [u8; 31],
        subindex: usize,
        value: Vec<u8>,
        existing_stem_bits: Vec<u8>,
        depth: usize,
    ) -> TreeNode {
        let stem_bits = self.bytes_to_bits(&stem);
        if stem_bits[depth] == existing_stem_bits[depth] {
            let mut internal = InternalNode::new();
            let bit = stem_bits[depth];
            if bit == 0 {
                internal.left = Some(Box::new(self.split_leaf(
                    leaf,
                    stem,
                    subindex,
                    value,
                    existing_stem_bits,
                    depth + 1,
                )));
            } else {
                internal.right = Some(Box::new(self.split_leaf(
                    leaf,
                    stem,
                    subindex,
                    value,
                    existing_stem_bits,
                    depth + 1,
                )));
            }
            TreeNode::Internal(internal)
        } else {
            let mut internal = InternalNode::new();
            let bit = stem_bits[depth];
            let mut new_stem_node = StemNode::new(stem);
            new_stem_node.set_value(subindex, value);
            if bit == 0 {
                internal.left = Some(Box::new(TreeNode::Stem(new_stem_node)));
                internal.right = Some(Box::new(TreeNode::Stem(leaf)));
            } else {
                internal.right = Some(Box::new(TreeNode::Stem(new_stem_node)));
                internal.left = Some(Box::new(TreeNode::Stem(leaf)));
            }
            TreeNode::Internal(internal)
        }
    }

    fn bytes_to_bits(&self, bytes: &[u8]) -> Vec<u8> {
        bytes
            .iter()
            .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
            .collect()
    }

    fn hash(data: &[u8]) -> [u8; 32] {
        if data.is_empty() || data == [0; 64] {
            [0; 32]
        } else {
            let mut hasher = Hasher::new();
            hasher.update(data);
            *hasher.finalize().as_bytes()
        }
    }

    pub fn merkelize(&self) -> [u8; 32] {
        fn _merkelize(tree: &Option<TreeNode>, hash_fn: &dyn Fn(&[u8]) -> [u8; 32]) -> [u8; 32] {
            match tree {
                None => [0; 32],
                Some(TreeNode::Internal(node)) => {
                    let left_hash = _merkelize(&node.left.as_ref().map(|l| *l.clone()), hash_fn);
                    let right_hash = _merkelize(&node.right.as_ref().map(|r| *r.clone()), hash_fn);
                    let mut combined = Vec::with_capacity(64); // 32 bytes for each hash
                    combined.extend_from_slice(&left_hash);
                    combined.extend_from_slice(&right_hash);
                    hash_fn(&combined)
                }
                Some(TreeNode::Stem(node)) => {
                    let mut level: Vec<[u8; 32]> = node
                        .values
                        .iter()
                        .map(|opt| hash_fn(opt.as_deref().unwrap_or(&[0; 64])))
                        .collect();

                    while level.len() > 1 {
                        let mut new_level = Vec::new();
                        for pair in level.chunks(2) {
                            let combined = match pair {
                                [a, b] => [&a[..], &b[..]].concat(),
                                [a] => a.to_vec(),
                                _ => vec![],
                            };
                            new_level.push(hash_fn(&combined));
                        }
                        level = new_level;
                    }

                    let mut buffer = Vec::with_capacity(31 + 1 + 32);
                    buffer.extend_from_slice(&node.stem);
                    buffer.push(0);
                    buffer.extend_from_slice(&level[0]);
                    let stem_hash = hash_fn(&buffer);
                    stem_hash
                }
            }
        }

        _merkelize(&self.root, &Self::hash)
    }
}

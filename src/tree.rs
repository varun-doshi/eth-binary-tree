use blake3::Hasher;
use rand::{Rng, RngCore};

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
pub struct MerkleProof {
    pub path: Vec<[u8; 32]>, // sibling hashes from root to leaf
    pub value: Option<Vec<u8>>,
    pub subindex: usize,
    pub stem: [u8; 31],
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

    pub fn get_proof(&self, key: [u8; 32]) -> Option<MerkleProof> {
        let stem: [u8; 31] = key[..31].try_into().ok()?;
        let subindex = key[31] as usize;
        let stem_bits = self.bytes_to_bits(&stem);

        let mut path = Vec::new();
        let mut current = self.root.as_ref()?;
        let mut depth = 0;

        while let TreeNode::Internal(node) = current {
            let bit = stem_bits.get(depth)?;
            let (next_node, sibling) = if *bit == 0 {
                (&node.left, &node.right)
            } else {
                (&node.right, &node.left)
            };

            let sibling_hash = Self::hash_node(sibling.as_deref());
            path.push(sibling_hash);

            current = next_node.as_deref()?;
            depth += 1;
        }

        if let TreeNode::Stem(stem_node) = current {
            if stem_node.stem != stem {
                return None; // mismatched stem
            }
            let value = stem_node.values[subindex].clone();
            Some(MerkleProof {
                path,
                value,
                subindex,
                stem,
            })
        } else {
            None
        }
    }

    fn hash_node(node: Option<&TreeNode>) -> [u8; 32] {
        match node {
            None => [0; 32],
            Some(TreeNode::Internal(internal)) => {
                let left = Self::hash_node(internal.left.as_deref());
                let right = Self::hash_node(internal.right.as_deref());
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&left);
                combined.extend_from_slice(&right);
                Self::hash(&combined)
            }
            Some(TreeNode::Stem(node)) => {
                let mut level: Vec<[u8; 32]> = node
                    .values
                    .iter()
                    .map(|opt| Self::hash(opt.as_deref().unwrap_or(&[0; 64])))
                    .collect();

                while level.len() > 1 {
                    let mut new_level = Vec::new();
                    for pair in level.chunks(2) {
                        let combined = match pair {
                            [a, b] => [&a[..], &b[..]].concat(),
                            [a] => a.to_vec(),
                            _ => vec![],
                        };
                        new_level.push(Self::hash(&combined));
                    }
                    level = new_level;
                }

                let mut buffer = Vec::with_capacity(31 + 1 + 32);
                buffer.extend_from_slice(&node.stem);
                buffer.push(0);
                buffer.extend_from_slice(&level[0]);
                Self::hash(&buffer)
            }
        }
    }

    fn bytes_to_bits_static(bytes: &[u8]) -> Vec<u8> {
        bytes
            .iter()
            .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
            .collect()
    }
}

pub fn verify_proof(proof: &MerkleProof, root_hash: [u8; 32], key: [u8; 32]) -> bool {
    let MerkleProof {
        path,
        value,
        subindex,
        stem,
    } = proof;

    let mut leaf_hash = {
        let mut level: Vec<[u8; 32]> = (0..256)
            .map(|i| {
                if i == *subindex {
                    BinaryTree::hash(value.as_deref().unwrap_or(&[0; 64]))
                } else {
                    BinaryTree::hash(&[0; 64])
                }
            })
            .collect();

        while level.len() > 1 {
            let mut new_level = Vec::new();
            for pair in level.chunks(2) {
                let combined = match pair {
                    [a, b] => [&a[..], &b[..]].concat(),
                    [a] => a.to_vec(),
                    _ => vec![],
                };
                new_level.push(BinaryTree::hash(&combined));
            }
            level = new_level;
        }

        let mut buffer = Vec::with_capacity(31 + 1 + 32);
        buffer.extend_from_slice(stem);
        buffer.push(0);
        buffer.extend_from_slice(&level[0]);
        BinaryTree::hash(&buffer)
    };

    let stem_bits = BinaryTree::bytes_to_bits_static(stem);

    for (depth, sibling_hash) in path.iter().rev().enumerate() {
        let bit = *stem_bits.get(path.len() - 1 - depth).unwrap_or(&0);

        let (left, right) = if bit == 0 {
            (leaf_hash, *sibling_hash)
        } else {
            (*sibling_hash, leaf_hash)
        };

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&left);
        combined.extend_from_slice(&right);
        leaf_hash = BinaryTree::hash(&combined);
    }

    leaf_hash == root_hash
}

pub fn random_key() -> [u8; 32] {
    let mut rng = rand::rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

/// Generate a random 32-byte value
pub fn random_value() -> [u8; 32] {
    let mut rng = rand::rng();
    let mut value = [0u8; 32];
    rng.fill_bytes(&mut value);
    value
}

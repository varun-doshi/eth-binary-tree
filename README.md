# Rust Implementation of Ethereum Binary Tree EIP-7864

This EIP proposes a new binary state tree to replace the current hexary Patricia trees in Ethereum. The new structure merges account and storage tries into a single tree with 32-byte keys, aiming to improve the efficiency and simplicity of state proofs. The proposal seeks to enhance Ethereum's ability to support validity proofs, reduce the size of Merkle proofs, and improve the overall performance of the network. The binary tree structure is designed to be more SNARK-friendly and post-quantum secure, with a focus on using the BLAKE3 hash function for merkelization

Check out actual proposal [EIP-7864](https://eips.ethereum.org/EIPS/eip-7864)

Longer Explainer thread on [Ethereum Magicians](https://ethereum-magicians.org/t/eip-7864-ethereum-state-using-a-unified-binary-tree/22611)

Shoutout to:</br>
 -Vitalik Buterin </br>
 -Guillaume Ballet</br>
 -Dankrad Feist</br>
 -Ignacio Hagopian</br>
 -Kevaundray Wedderburn</br>
 -Tanishq Jasoria</br>
 -Gajinder Singh</br>
 -Danno Ferrin</br>
 -Piper Merriam</br>
 -Gottfried Herold</br>

For the original proposal


Now contains the implementation of both the `tree` and `embedding`.

### Disclaimer: This is just a minimal conversion of the python specs from the original proposal created as a POC. Do not use in production.
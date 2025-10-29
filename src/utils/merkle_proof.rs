use sha2::{Digest, Sha256};

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let r = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}

pub fn verify_merkle_proof(leaf_hash: [u8; 32], proof: &Vec<[u8; 32]>, root: [u8; 32]) -> bool {
    let mut computed = leaf_hash;
    for sibling in proof {
        // consistent ordering
        let pair = if computed <= *sibling {
            [computed.as_ref(), sibling.as_ref()].concat()
        } else {
            [sibling.as_ref(), computed.as_ref()].concat()
        };
        let mut hasher = Sha256::new();
        hasher.update(&pair);
        let r = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&r);
        computed = out;
    }
    computed == root
}

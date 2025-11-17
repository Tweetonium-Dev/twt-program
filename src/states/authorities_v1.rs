#[derive(Debug)]
pub struct NftAuthorityV1;

impl NftAuthorityV1 {
    pub const SEED: &[u8; 16] = b"nft_authority_v1";
}

#[derive(Debug)]
pub struct TraitAuthorityV1;

impl TraitAuthorityV1 {
    pub const SEED: &[u8; 18] = b"trait_authority_v1";
}

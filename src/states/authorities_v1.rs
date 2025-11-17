#[derive(Debug)]
pub struct NftAuthorityV1;

impl NftAuthorityV1 {
    pub const SEED: &[u8; 16] = b"nft_authority_v1";
}

#[derive(Debug)]
pub struct TraitAuthority;

impl TraitAuthority {
    pub const SEED: &[u8; 15] = b"trait_authority";
}

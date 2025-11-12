#[derive(Debug)]
pub struct NftAuthority;

impl NftAuthority {
    pub const SEED: &[u8; 13] = b"nft_authority";
}

#[derive(Debug)]
pub struct TraitAuthority;

impl TraitAuthority {
    pub const SEED: &[u8; 15] = b"trait_authority";
}

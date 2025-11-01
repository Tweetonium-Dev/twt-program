#[derive(Debug)]
pub struct NftAuthority;

impl NftAuthority {
    pub const SEED: &[u8; 13] = b"nft_authority";
}

#[derive(Debug)]
pub struct VaultAuthority;

impl VaultAuthority {
    pub const SEED: &[u8; 15] = b"vault_authority";
}

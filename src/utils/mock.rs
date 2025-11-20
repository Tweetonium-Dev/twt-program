use borsh::BorshSerialize;
use mpl_core::{
    accounts::BaseAssetV1,
    types::{Key, UpdateAuthority},
};
use solana_program::{
    account_info::AccountInfo, clock::Epoch, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::utils::{MINT_2022_MIN_LEN, MINT_LEN, TOKEN_ACCOUNT_2022_MIN_LEN, TOKEN_ACCOUNT_LEN};

pub fn noop_processor(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _ix_data: &[u8],
) -> ProgramResult {
    Ok(())
}

pub fn mock_u16s<const N: usize>(value: u16) -> [u16; N] {
    [value; N]
}

pub fn mock_u64s<const N: usize>(value: u64) -> [u64; N] {
    [value; N]
}

pub fn default_pubkeys<const N: usize>() -> [Pubkey; N] {
    [Pubkey::default(); N]
}

pub fn mock_pubkeys<const N: usize>() -> [Pubkey; N] {
    let mut arr: [Pubkey; N] = [Pubkey::default(); N];
    for key in arr.iter_mut().take(N) {
        *key = Pubkey::new_unique();
    }
    arr
}

pub fn mock_account(
    key: Pubkey,
    is_signer: bool,
    is_writable: bool,
    lamports: u64,
    data_len: usize,
    owner: Pubkey,
) -> AccountInfo<'static> {
    let lamports = Box::new(lamports);
    let data = vec![0u8; data_len].into_boxed_slice();
    let owner = Box::new(owner);

    AccountInfo::new(
        Box::leak(Box::new(key)),
        is_signer,
        is_writable,
        Box::leak(lamports),
        Box::leak(data),
        Box::leak(owner),
        false,
        Epoch::default(),
    )
}

pub fn mock_account_with_data(
    key: Pubkey,
    is_signer: bool,
    is_writable: bool,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
) -> AccountInfo<'static> {
    let lamports = Box::new(lamports);
    let data = data.into_boxed_slice();
    let owner = Box::new(owner);

    AccountInfo::new(
        Box::leak(Box::new(key)),
        is_signer,
        is_writable,
        Box::leak(lamports),
        Box::leak(data),
        Box::leak(owner),
        false,
        Epoch::default(),
    )
}

pub fn mock_mint(decimals: u8, mint_authority: Pubkey) -> Vec<u8> {
    let mut data = vec![0u8; MINT_LEN];

    // mint_authority = Some
    data[0..4].copy_from_slice(&1u32.to_le_bytes());
    data[4..36].copy_from_slice(mint_authority.as_ref());

    // supply = 0
    data[36..44].copy_from_slice(&0u64.to_le_bytes());

    // decimals
    data[44] = decimals;

    // is_initialized = true
    data[45] = 1u8;

    // freeze_authority = None
    data[46..50].copy_from_slice(&0u32.to_le_bytes());
    // (remaining 32 bytes zero)

    data
}

pub fn mock_mint_2022(decimals: u8, mint_authority: Pubkey) -> Vec<u8> {
    let mut data = vec![0u8; MINT_2022_MIN_LEN]; // 90? Use 82 + TLV header (8 bytes)

    // Offset 0..4: mint_authority COption tag (1 = Some)
    data[0..4].copy_from_slice(&1u32.to_le_bytes());

    // Offset 4..36: mint_authority Pubkey
    data[4..36].copy_from_slice(&mint_authority.to_bytes());

    // Offset 36..44: supply (u64 LE)
    data[36..44].copy_from_slice(&0u64.to_le_bytes());

    // Offset 44: decimals (u8)
    data[44] = decimals;

    // Offset 45: is_initialized (u8 = 1)
    data[45] = 1;

    // Offset 46..50: freeze_authority COption tag (0 = None)
    data[46..50].copy_from_slice(&0u32.to_le_bytes());

    // Offset 50..82: freeze_authority Pubkey (zeroed, ignored)

    // Offset 82..90: TLV header (zero extensions)
    data[82..84].copy_from_slice(&0u16.to_le_bytes()); // type_count
    data[84..86].copy_from_slice(&0u16.to_le_bytes()); // length
    data[86..90].copy_from_slice(&0u32.to_le_bytes()); // reserved

    data
}

pub fn mock_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut data = vec![0u8; TOKEN_ACCOUNT_LEN];

    // mint
    data[0..32].copy_from_slice(mint.as_ref());

    // owner
    data[32..64].copy_from_slice(owner.as_ref());

    // amount
    data[64..72].copy_from_slice(&amount.to_le_bytes());

    // delegate = None
    data[72..76].copy_from_slice(&0u32.to_le_bytes());

    // state = Initialized (1)
    data[108] = 1u8;

    // is_native = None
    data[109..113].copy_from_slice(&0u32.to_le_bytes());

    // delegated_amount = 0
    data[121..129].copy_from_slice(&0u64.to_le_bytes());

    // close_authority = None
    data[129..133].copy_from_slice(&0u32.to_le_bytes());

    data
}

pub fn mock_token_account_2022(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut data = vec![0u8; TOKEN_ACCOUNT_2022_MIN_LEN]; // 165 + TLV min (2 bytes) = 167

    // Offset 0..32: mint Pubkey
    data[0..32].copy_from_slice(mint.as_ref());

    // Offset 32..64: owner Pubkey
    data[32..64].copy_from_slice(owner.as_ref());

    // Offset 64..72: amount (u64 LE)
    data[64..72].copy_from_slice(&amount.to_le_bytes());

    // Offset 72..76: delegate COption tag (0 = None)
    data[72..76].copy_from_slice(&0u32.to_le_bytes());

    // Offset 76..108: delegate Pubkey (zeroed, ignored)

    // Offset 108: state (u8 = 1 = Initialized)
    data[108] = 1;

    // Offset 109..113: is_native COption tag (0 = None)
    data[109..113].copy_from_slice(&0u32.to_le_bytes());

    // Offset 113..121: is_native amount (zeroed, ignored)

    // Offset 121..129: delegated_amount (u64 LE = 0)
    data[121..129].copy_from_slice(&0u64.to_le_bytes());

    // Offset 129..133: close_authority COption tag (0 = None)
    data[129..133].copy_from_slice(&0u32.to_le_bytes());

    // === TLV: one zero-length extension (makes total 167) ===
    data[165] = 0; // ExtensionType (0 = unused/reserved)
    data[166] = 0; // length = 0

    data
}

pub fn mock_base_asset(owner: Pubkey, name: &str, uri: &str) -> Vec<u8> {
    let base = BaseAssetV1 {
        key: Key::AssetV1,
        owner,
        update_authority: UpdateAuthority::Collection(Pubkey::new_unique()),
        name: name.to_string(),
        uri: uri.to_string(),
        seq: None,
    };

    base.try_to_vec().expect("serialize BaseAssetV1")
}

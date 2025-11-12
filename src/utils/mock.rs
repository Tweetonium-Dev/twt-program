use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey};

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

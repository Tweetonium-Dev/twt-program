use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

pub fn new_test_account(
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
    )
}

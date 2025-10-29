use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use solana_sdk_ids::system_program;

pub trait ToOptionalAccount<'a, 'info> {
    fn to_optional(self) -> Option<&'a AccountInfo<'info>>;
}

impl<'a, 'info> ToOptionalAccount<'a, 'info> for &'a AccountInfo<'info> {
    fn to_optional(self) -> Option<&'a AccountInfo<'info>> {
        if self.key == &system_program::ID || self.key == &Pubkey::default() {
            None
        } else {
            Some(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::*;

    #[test]
    fn test_optional_account_exist() {
        let acc = new_test_account(
            Pubkey::new_unique(),
            false,
            false,
            10u64,
            0,
            Pubkey::new_unique(),
        );
        assert!(acc.to_optional().is_some());
    }

    #[test]
    fn test_optional_account_sentinel_account() {
        let acc = new_test_account(
            Pubkey::default(),
            false,
            false,
            10u64,
            0,
            Pubkey::new_unique(),
        );
        assert!(acc.to_optional().is_none());
    }
}

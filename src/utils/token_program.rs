use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
};

pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const MINT_LEN: usize = 82;
pub const MINT_2022_MIN_LEN: usize = 90;

pub const TOKEN_ACCOUNT_LEN: usize = 165;
pub const TOKEN_ACCOUNT_2022_MIN_LEN: usize = 167;

#[derive(Debug)]
pub enum TokenProgram {
    Token,
    Token2022,
}

impl TokenProgram {
    pub fn detect_token_program(account: &AccountInfo) -> Result<Self, ProgramError> {
        if account.key == &TOKEN_PROGRAM_ID {
            Ok(Self::Token)
        } else if account.key == &TOKEN_2022_PROGRAM_ID {
            Ok(Self::Token2022)
        } else {
            msg!("Invalid token program {}", account.key);
            Err(ProgramError::InvalidAccountOwner)
        }
    }

    pub fn get_decimal<'info>(mint: &AccountInfo<'info>) -> Result<u8, ProgramError> {
        const DECIMALS_OFFSET: usize = 44;

        let data = mint.try_borrow_data()?;
        if data.len() <= DECIMALS_OFFSET {
            msg!("Invalid mint data {}", mint.key);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data[DECIMALS_OFFSET])
    }

    pub fn get_balance<'info>(
        token_account: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<u64, ProgramError> {
        let data = token_account.try_borrow_data()?;
        let balance_offset = match Self::detect_token_program(token_program)? {
            Self::Token => 64, // SplTokenAccount::amount at byte 64
            Self::Token2022 => {
                let mut offset = 64;
                if data.len() < 72 {
                    let header_candidate = &data[..8];
                    let likely_tlv = header_candidate.iter().any(|&b| b != 0);
                    if likely_tlv {
                        offset += 8;
                    }
                }
                offset
            }
        };

        if data.len() < balance_offset + 8 {
            msg!("Invalid token data {}", token_account.key);
            return Err(ProgramError::InvalidAccountData);
        }

        let balance_bytes: [u8; 8] = data[balance_offset..balance_offset + 8]
            .try_into()
            .inspect_err(|_| msg!("Balance bytes not found"))
            .map_err(|_| ProgramError::Custom(4))?;

        Ok(u64::from_le_bytes(balance_bytes))
    }

    pub fn transfer<'a, 'info>(
        accounts: TokenTransferAccounts<'a, 'info>,
        args: TokenTransferArgs,
    ) -> ProgramResult {
        Self::transfer_signed(accounts, args, &[])
    }

    pub fn transfer_signed<'a, 'info>(
        accounts: TokenTransferAccounts<'a, 'info>,
        args: TokenTransferArgs,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        match Self::detect_token_program(accounts.token_program)? {
            Self::Token => {
                let ix = Self::token_transfer_checked_ix(
                    *accounts.source.key,
                    *accounts.mint.key,
                    *accounts.destination.key,
                    *accounts.authority.key,
                    args.amount,
                    args.decimals,
                );

                invoke_signed(
                    &ix,
                    &[
                        accounts.source.clone(),
                        accounts.mint.clone(),
                        accounts.destination.clone(),
                        accounts.authority.clone(),
                        accounts.token_program.clone(),
                    ],
                    signers_seeds,
                )
            }
            Self::Token2022 => {
                let ix = Self::token_2022_transfer_checked_ix(
                    *accounts.source.key,
                    *accounts.mint.key,
                    *accounts.destination.key,
                    *accounts.authority.key,
                    args.amount,
                    args.decimals,
                );

                msg!("invoke tf 2022 instruction");

                invoke_signed(
                    &ix,
                    &[
                        accounts.source.clone(),
                        accounts.mint.clone(),
                        accounts.destination.clone(),
                        accounts.authority.clone(),
                        accounts.token_program.clone(),
                    ],
                    signers_seeds,
                )
            }
        }
    }

    fn token_transfer_checked_ix(
        source: Pubkey,
        mint: Pubkey,
        destination: Pubkey,
        authority: Pubkey,
        amount: u64,
        decimals: u8,
    ) -> Instruction {
        let mut data = Vec::with_capacity(10);
        data.push(12);
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(decimals);

        let accounts = vec![
            AccountMeta::new(source, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(authority, true),
        ];

        Instruction {
            program_id: TOKEN_PROGRAM_ID,
            accounts,
            data,
        }
    }

    fn token_2022_transfer_checked_ix(
        source: Pubkey,
        mint: Pubkey,
        destination: Pubkey,
        authority: Pubkey,
        amount: u64,
        decimals: u8,
    ) -> Instruction {
        // Instruction discriminator for TransferChecked = 12
        let mut data = Vec::with_capacity(10);
        data.push(12);
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(decimals);

        let accounts = vec![
            AccountMeta::new(source, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(authority, true),
        ];

        Instruction {
            program_id: TOKEN_2022_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

pub struct TokenTransferAccounts<'a, 'info> {
    pub source: &'a AccountInfo<'info>,
    pub destination: &'a AccountInfo<'info>,
    pub authority: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
}

pub struct TokenTransferArgs {
    pub amount: u64,
    pub decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    // --- Test Helpers ---

    fn mock_account_info(key: Pubkey, data: Vec<u8>) -> AccountInfo<'static> {
        crate::utils::mock::mock_account_with_data(key, false, true, 0, data, Pubkey::new_unique())
    }

    // --- Test Cases ---

    #[test]
    fn test_detect_token_program_valid() {
        let token_program_info = mock_account_info(TOKEN_PROGRAM_ID, vec![]);
        let token_2022_info = mock_account_info(TOKEN_2022_PROGRAM_ID, vec![]);

        assert!(matches!(
            TokenProgram::detect_token_program(&token_program_info).unwrap(),
            TokenProgram::Token
        ));

        assert!(matches!(
            TokenProgram::detect_token_program(&token_2022_info).unwrap(),
            TokenProgram::Token2022
        ));
    }

    #[test]
    fn test_detect_token_program_invalid() {
        let random = mock_account_info(Pubkey::new_unique(), vec![]);
        let err = TokenProgram::detect_token_program(&random).unwrap_err();
        assert_eq!(err, ProgramError::InvalidAccountOwner);
    }

    #[test]
    fn test_get_decimal_valid() {
        let mut data = vec![0u8; 82];
        data[44] = 9; // decimals at offset 44
        let mint_info = mock_account_info(Pubkey::new_unique(), data);

        let result = TokenProgram::get_decimal(&mint_info).unwrap();
        assert_eq!(result, 9);
    }

    #[test]
    fn test_get_decimal_invalid_data_len() {
        let data = vec![0u8; 20]; // too short
        let mint_info = mock_account_info(Pubkey::new_unique(), data);

        let err = TokenProgram::get_decimal(&mint_info).unwrap_err();
        assert_eq!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_get_balance_token_valid() {
        // SPL Token account: balance at offset 64
        let mut data = vec![0u8; 80];
        let balance: u64 = 123_456_789;
        data[64..72].copy_from_slice(&balance.to_le_bytes());

        let token_account_info = mock_account_info(Pubkey::new_unique(), data);
        let token_program_info = mock_account_info(TOKEN_PROGRAM_ID, vec![]);

        let result = TokenProgram::get_balance(&token_account_info, &token_program_info).unwrap();
        assert_eq!(result, 123_456_789);
    }

    #[test]
    fn test_get_balance_invalid_len() {
        let data = vec![0u8; 10]; // too short for balance
        let token_account_info = mock_account_info(Pubkey::new_unique(), data);
        let token_program_info = mock_account_info(TOKEN_PROGRAM_ID, vec![]);

        let err = TokenProgram::get_balance(&token_account_info, &token_program_info).unwrap_err();
        assert_eq!(err, ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_token_2022_transfer_checked_ix_structure() {
        let src = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let dst = Pubkey::new_unique();
        let auth = Pubkey::new_unique();

        let ix = TokenProgram::token_2022_transfer_checked_ix(src, mint, dst, auth, 9999, 6);

        // Check program id
        assert_eq!(ix.program_id, TOKEN_2022_PROGRAM_ID);
        // Instruction discriminator (TransferChecked = 12)
        assert_eq!(ix.data[0], 12);
        // Amount LE
        assert_eq!(u64::from_le_bytes(ix.data[1..9].try_into().unwrap()), 9999);
        // Decimals
        assert_eq!(ix.data[9], 6);

        // Accounts
        assert_eq!(ix.accounts[0].pubkey, src);
        assert_eq!(ix.accounts[1].pubkey, mint);
        assert_eq!(ix.accounts[2].pubkey, dst);
        assert_eq!(ix.accounts[3].pubkey, auth);
    }
}

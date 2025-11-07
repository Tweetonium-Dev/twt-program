use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const MINT_LEN: usize = 82;
pub const MINT_2022_MIN_LEN: usize = 90;

pub const TOKEN_ACCOUNT_LEN: usize = 165;
pub const TOKEN_ACCOUNT_2022_MIN_LEN: usize = 167;

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

    pub fn get_decimal<'info>(
        mint: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<u8, ProgramError> {
        let data = mint.try_borrow_data()?;
        let decimals_offset = match Self::detect_token_program(token_program)? {
            Self::Token => 44,
            Self::Token2022 => 8 + 44, // Extension header (8 bytes) + Mint::decimals
        };

        if data.len() < decimals_offset + 1 {
            msg!("Invalid mint data {}", mint.key);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data[decimals_offset])
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
            .map_err(|_| ProgramError::Custom(6))?;

        Ok(u64::from_le_bytes(balance_bytes))
    }

    pub fn transfer(args: TokenTransferArgs) -> ProgramResult {
        Self::transfer_signed(args, &[])
    }

    pub fn transfer_signed(args: TokenTransferArgs, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        match Self::detect_token_program(args.token_program)? {
            Self::Token => {
                let ix = transfer(
                    &TOKEN_PROGRAM_ID,
                    args.source.key,
                    args.destination.key,
                    args.authority.key,
                    args.signer_pubkeys,
                    args.amount,
                )?;

                invoke(
                    &ix,
                    &[
                        args.source.clone(),
                        args.destination.clone(),
                        args.authority.clone(),
                        args.token_program.clone(),
                    ],
                )?;
            }
            Self::Token2022 => {
                let ix = Self::token_2022_transfer_checked_ix(
                    *args.source.key,
                    *args.mint.key,
                    *args.destination.key,
                    *args.authority.key,
                    args.signer_pubkeys,
                    args.amount,
                    args.decimals,
                );

                invoke_signed(
                    &ix,
                    &[
                        args.source.clone(),
                        args.mint.clone(),
                        args.destination.clone(),
                        args.authority.clone(),
                        args.token_program.clone(),
                    ],
                    signers_seeds,
                )?;
            }
        };

        Ok(())
    }

    fn token_2022_transfer_checked_ix(
        source: Pubkey,
        mint: Pubkey,
        destination: Pubkey,
        authority: Pubkey,
        signer_pubkeys: &[&Pubkey],
        amount: u64,
        decimals: u8,
    ) -> Instruction {
        // Instruction discriminator for TransferChecked = 12
        let mut data = Vec::with_capacity(10);
        data.push(12);
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(decimals);

        let mut accounts = vec![
            AccountMeta::new(source, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(authority, true),
        ];
        for signer in signer_pubkeys {
            accounts.push(AccountMeta::new_readonly(**signer, true));
        }

        Instruction {
            program_id: TOKEN_2022_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(Clone)]
pub struct TokenTransferArgs<'a, 'info> {
    pub source: &'a AccountInfo<'info>,
    pub destination: &'a AccountInfo<'info>,
    pub authority: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub signer_pubkeys: &'a [&'a Pubkey],
    pub amount: u64,
    pub decimals: u8,
}

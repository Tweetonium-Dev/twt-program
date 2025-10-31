use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
};
use spl_token::instruction::{burn_checked, transfer, transfer_checked};

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

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
        let owner_bytes = account.owner.to_bytes();
        let legacy_id = TOKEN_PROGRAM_ID.to_bytes();
        let token2022_id = TOKEN_2022_PROGRAM_ID.to_bytes();

        if owner_bytes == legacy_id {
            Ok(Self::Token)
        } else if owner_bytes == token2022_id {
            Ok(Self::Token2022)
        } else {
            Err(ProgramError::InvalidAccountOwner)
        }
    }

    pub fn get_decimal<'info>(
        mint: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<u8, ProgramError> {
        let data = mint.try_borrow_data()?;
        let decimals_offset = match Self::detect_token_program(token_program)? {
            Self::Token => 36,
            Self::Token2022 => 8 + 36, // Extension header (8 bytes) + Mint::decimals
        };

        if data.len() < decimals_offset + 1 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data[decimals_offset])
    }

    pub fn transfer(args: TransferArgs) -> ProgramResult {
        Self::transfer_signed(args, &[])
    }

    pub fn transfer_signed(args: TransferArgs, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
                        args.mint.clone(),
                        args.token_program.clone(),
                    ],
                )?;
            }
            Self::Token2022 => {
                let ix = transfer_checked(
                    &TOKEN_2022_PROGRAM_ID,
                    args.source.key,
                    args.mint.key,
                    args.destination.key,
                    args.authority.key,
                    args.signer_pubkeys,
                    args.amount,
                    args.decimals,
                )?;

                invoke_signed(
                    &ix,
                    &[
                        args.source.clone(),
                        args.destination.clone(),
                        args.authority.clone(),
                        args.mint.clone(),
                        args.token_program.clone(),
                    ],
                    signers_seeds,
                )?;
            }
        };

        Ok(())
    }

    pub fn burn_nft<'info>(
        token_program: &AccountInfo<'info>,
        token_account: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        decimals: u8,
        multisig_signers: &[&Pubkey],
    ) -> ProgramResult {
        Self::burn_nft_signed(
            token_program,
            token_account,
            mint,
            authority,
            multisig_signers,
            decimals,
            &[],
        )
    }

    pub fn burn_nft_signed<'info>(
        token_program: &AccountInfo<'info>,
        token_account: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        multisig_signers: &[&Pubkey],
        decimals: u8,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        match Self::detect_token_program(token_program)? {
            Self::Token => {
                let ix = burn_checked(
                    &TOKEN_PROGRAM_ID,
                    token_account.key,
                    mint.key,
                    authority.key,
                    multisig_signers,
                    1,
                    decimals,
                )?;

                invoke(
                    &ix,
                    &[
                        token_account.clone(),
                        mint.clone(),
                        authority.clone(),
                        token_program.clone(),
                    ],
                )?;
            }
            Self::Token2022 => {
                let ix = burn_checked(
                    &TOKEN_2022_PROGRAM_ID,
                    token_account.key,
                    mint.key,
                    authority.key,
                    multisig_signers,
                    1,
                    decimals,
                )?;

                invoke_signed(
                    &ix,
                    &[
                        token_account.clone(),
                        mint.clone(),
                        authority.clone(),
                        token_program.clone(),
                    ],
                    signers_seeds,
                )?;
            }
        };

        Ok(())
    }
}

#[derive(Clone)]
pub struct TransferArgs<'a, 'info> {
    pub source: &'a AccountInfo<'info>,
    pub destination: &'a AccountInfo<'info>,
    pub authority: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub signer_pubkeys: &'a [&'a Pubkey],
    pub amount: u64,
    pub decimals: u8,
}

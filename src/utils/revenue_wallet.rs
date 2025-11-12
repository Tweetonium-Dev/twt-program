use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::utils::{
    AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenProgram,
    InitAssociatedTokenProgramAccounts, TokenProgram, TokenTransferAccounts, TokenTransferArgs,
};

pub struct RevenueWallet;

impl RevenueWallet {
    pub fn transfer<'a, 'info>(
        accounts: RevenueWalletAccounts<'a, 'info>,
        args: RevenueWalletArgs,
    ) -> ProgramResult {
        AssociatedTokenProgram::init_if_needed(InitAssociatedTokenProgramAccounts {
            payer: accounts.payer,
            wallet: accounts.wallet,
            mint: accounts.mint,
            token_program: accounts.token_program,
            associated_token_program: accounts.associated_token_program,
            system_program: accounts.system_program,
            ata: accounts.destination_ata,
        })?;

        AssociatedTokenAccount::check(
            accounts.destination_ata,
            accounts.wallet.key,
            accounts.mint.key,
            accounts.token_program.key,
        )?;

        TokenProgram::transfer(
            TokenTransferAccounts {
                source: accounts.payer_ata,
                destination: accounts.destination_ata,
                authority: accounts.payer,
                mint: accounts.mint,
                token_program: accounts.token_program,
            },
            TokenTransferArgs {
                signer_pubkeys: &[],
                amount: args.amount,
                decimals: args.decimals,
            },
        )
    }
}

pub struct RevenueWalletAccounts<'a, 'info> {
    pub payer_ata: &'a AccountInfo<'info>,
    pub destination_ata: &'a AccountInfo<'info>,
    pub payer: &'a AccountInfo<'info>,
    pub wallet: &'a AccountInfo<'info>,
    pub mint: &'a AccountInfo<'info>,
    pub token_program: &'a AccountInfo<'info>,
    pub associated_token_program: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct RevenueWalletArgs {
    pub amount: u64,
    pub decimals: u8,
}

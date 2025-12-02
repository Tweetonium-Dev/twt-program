use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::VaultV1,
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenProgram, InitAssociatedTokenProgramAccounts, MintAccount, Pda, ProcessInstruction, SignerAccount, SystemProgram, TokenProgram, TokenTransferAccounts, TokenTransferArgs, WritableAccount
    },
};

#[derive(Debug)]
pub struct TransferToVaultV1Accounts<'a, 'info> {
    /// User paying the mint price in 'token_mint' and solana.
    /// Must be signer and owner of `payer_ata`.
    pub payer: &'a AccountInfo<'info>,

    /// Payer's ATA for 'token_mint' — source of payment.
    /// Must be writable, owned by `token_program`.
    pub payer_ata: &'a AccountInfo<'info>,

    /// PDA: `["vault_v1", nft_asset, nft_collection, token_mint, program_id]` — stores `Vault` state.
    /// Must be writable if updating vault balance.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Associated Token Account (ATA) of the vault PDA.
    /// Holds 'new_token_mint' received from users.
    /// Must be writable, owned by `token_program`.
    pub new_vault_ata: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before config creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// Project token mint — the token already escrowed in the vault (e.g. TWT).
    /// Must match `config_pda.data.mint`, owned by `token_program`.
    pub project_token_mint: &'a AccountInfo<'info>,

    /// New token mint — the new token being escrowed (e.g. TWT).
    /// Must owned by `token_program`.
    pub new_token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_mint.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// Associated Token Program (ATA).
    /// Must be the official SPL Associated Token Account program.
    pub associated_token_program: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for TransferToVaultV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [payer, payer_ata, vault_pda, new_vault_ata, nft_collection, nft_asset, project_token_mint, new_token_mint, token_program, associated_token_program, system_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;

        WritableAccount::check(payer_ata)?;
        WritableAccount::check(new_vault_ata)?;

        MintAccount::check(project_token_mint)?;
        MintAccount::check(new_token_mint)?;
        SystemProgram::check(system_program)?;

        AssociatedTokenAccount::check(payer_ata, payer.key, new_token_mint.key, token_program.key)?;

        Ok(Self {
            payer,
            payer_ata,
            vault_pda,
            new_vault_ata,
            nft_collection,
            nft_asset,
            project_token_mint,
            new_token_mint,
            token_program,
            associated_token_program,
            system_program,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TransferToVaultV1InstructionData {
    pub amount: u64,
}

#[derive(Debug)]
pub struct TransferToVaultV1<'a, 'info> {
    pub accounts: TransferToVaultV1Accounts<'a, 'info>,
    pub instruction_data: TransferToVaultV1InstructionData,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        TransferToVaultV1InstructionData,
        &'a Pubkey,
    )> for TransferToVaultV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            TransferToVaultV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = TransferToVaultV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.vault_pda,
            &[
                VaultV1::SEED,
                accounts.nft_asset.key.as_ref(),
                accounts.nft_collection.key.as_ref(),
                accounts.project_token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a, 'info> TransferToVaultV1<'a, 'info> {
    fn init_vault(&self) -> ProgramResult {
        AssociatedTokenProgram::init_if_needed(InitAssociatedTokenProgramAccounts {
            payer: self.accounts.payer,
            wallet: self.accounts.vault_pda,
            mint: self.accounts.new_token_mint,
            token_program: self.accounts.token_program,
            associated_token_program: self.accounts.associated_token_program,
            system_program: self.accounts.system_program,
            ata: self.accounts.new_vault_ata,
        })
    }

    fn transfer_token(&self) -> ProgramResult {
        let decimals = TokenProgram::get_decimal(self.accounts.new_token_mint)?;

        TokenProgram::transfer(
            TokenTransferAccounts {
                source: self.accounts.payer_ata,
                destination: self.accounts.new_vault_ata,
                authority: self.accounts.payer,
                mint: self.accounts.new_token_mint,
                token_program: self.accounts.token_program,
            },
            TokenTransferArgs {
                amount: self.instruction_data.amount,
                decimals,
            },
        )
    }
}

impl<'a, 'info> ProcessInstruction for TransferToVaultV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        if self.instruction_data.amount == 0 {
            return Ok(());
        }

        self.init_vault()?;
        self.transfer_token()
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{ProjectV1, InitVaultAccounts, InitVaultArgs, NftAuthorityV1, VaultV1},
    utils::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenProgram,
        ProjectAccount, CreateMplCoreAssetAccounts, CreateMplCoreAssetArgs,
        InitAssociatedTokenProgramAccounts, InitPdaAccounts, InitPdaArgs, MintAccount,
        MplCoreProgram, Pda, ProcessInstruction, SignerAccount, SystemProgram, TokenProgram,
        TokenTransferAccounts, TokenTransferArgs, UninitializedAccount, WritableAccount,
    },
};

#[derive(Debug)]
pub struct MintAdminV1Accounts<'a, 'info> {
    /// Authority as payer (e.g. admin wallet).
    /// Must be a signer.
    pub admin: &'a AccountInfo<'info>,

    /// Admin's ATA for 'token_mint' — source of payment.
    /// Must be writable, owned by `token_program`.
    pub admin_ata: &'a AccountInfo<'info>,

    /// PDA: `["project_v1", nft_collection, token_mint, program_id]` — stores global project config.
    /// Must be readable, owned by program.
    pub project_pda: &'a AccountInfo<'info>,

    /// PDA: `["vault_v1", nft_asset, nft_collection, token_mint, program_id]` — stores `Vault` state.
    /// Must be writable if updating vault balance.
    pub vault_pda: &'a AccountInfo<'info>,

    /// Associated Token Account (ATA) of the vault PDA.
    /// Holds 'token_mint' received from users.
    /// Must be writable, owned by `token_program`.
    pub vault_ata: &'a AccountInfo<'info>,

    /// PDA: `["nft_authority_v1", program_id]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Must be initialized before project creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the NFT being minted.
    /// Must be uninitialized, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// Token mint — the token being escrowed (e.g. ZDLT).
    /// Must match `project_pda.data.mint`, owned by `token_program`.
    pub token_mint: &'a AccountInfo<'info>,

    /// SPL Token Program (legacy or Token-2022).
    /// Must match `token_mint.owner`.
    pub token_program: &'a AccountInfo<'info>,

    /// Associated Token Program (ATA).
    /// Used to derive and create ATAs (`vault_ata`, `payer_ata`) deterministically.
    /// Must be the official SPL Associated Token Account program.
    pub associated_token_program: &'a AccountInfo<'info>,

    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// System program — for account allocation.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for MintAdminV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [admin, admin_ata, project_pda, vault_pda, vault_ata, nft_authority, nft_collection, nft_asset, token_mint, token_program, associated_token_program, protocol_wallet, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(admin)?;
        SignerAccount::check(nft_asset)?;

        WritableAccount::check(admin_ata)?;
        WritableAccount::check(project_pda)?;
        WritableAccount::check(vault_pda)?;
        WritableAccount::check(vault_ata)?;
        WritableAccount::check(nft_collection)?;
        WritableAccount::check(nft_asset)?;
        WritableAccount::check(protocol_wallet)?;

        UninitializedAccount::check(nft_asset)?;

        ProjectAccount::check(project_pda)?;
        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        AssociatedTokenAccount::check(admin_ata, admin.key, token_mint.key, token_program.key)?;

        Ok(Self {
            admin,
            admin_ata,
            project_pda,
            vault_pda,
            vault_ata,
            nft_authority,
            nft_collection,
            nft_asset,
            token_mint,
            token_program,
            associated_token_program,
            protocol_wallet,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MintAdminV1InstructionData {
    pub nft_name: String,
    pub nft_uri: String,
}

#[derive(Debug)]
pub struct MintAdminV1<'a, 'info> {
    pub accounts: MintAdminV1Accounts<'a, 'info>,
    pub instruction_data: MintAdminV1InstructionData,
    pub program_id: &'a Pubkey,
    pub nft_authority_bump: u8,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        MintAdminV1InstructionData,
        &'a Pubkey,
    )> for MintAdminV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            MintAdminV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = MintAdminV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.project_pda,
            &[
                ProjectV1::SEED,
                accounts.nft_collection.key.as_ref(),
                accounts.token_mint.key.as_ref(),
            ],
            program_id,
        )?;

        let (_, nft_authority_bump) =
            Pda::validate(accounts.nft_authority, &[NftAuthorityV1::SEED], program_id)?;

        Ok(Self {
            accounts,
            instruction_data,
            program_id,
            nft_authority_bump,
        })
    }
}

impl<'a, 'info> MintAdminV1<'a, 'info> {
    fn check_mint_eligibility(&self, project: &ProjectV1) -> ProgramResult {
        let max_supply = project.max_supply;
        let released = project.released;
        let admin_supply = max_supply - released;
        let admin_minted = project.admin_minted;
        let user_minted = project.user_minted;
        let minted = admin_minted + user_minted;

        if !project.nft_stock_available() {
            msg!(
                "All NFTs are minted. Allowed supply: {}. Minted: {}",
                max_supply,
                minted,
            );
            return Err(ProgramError::Custom(0));
        }

        if !project.admin_mint_available() {
            msg!(
                "All admin NFTs already minted. Allowed supply: {}. Minted: {}",
                admin_supply,
                admin_minted
            );
            return Err(ProgramError::Custom(1));
        }

        Ok(())
    }

    fn store_to_vault(&self, project: &ProjectV1) -> ProgramResult {
        if !project.need_vault() {
            return Ok(());
        }

        let seeds: &[&[u8]] = &[
            VaultV1::SEED,
            self.accounts.nft_asset.key.as_ref(),
            self.accounts.nft_collection.key.as_ref(),
            self.accounts.token_mint.key.as_ref(),
        ];

        VaultV1::init_if_needed(
            InitVaultAccounts {
                pda: self.accounts.vault_pda,
            },
            InitVaultArgs {
                nft: *self.accounts.nft_asset.key,
                amount: project.escrow_amount,
                is_unlocked: false,
            },
            InitPdaAccounts {
                payer: self.accounts.admin,
                pda: self.accounts.vault_pda,
                system_program: self.accounts.system_program,
            },
            InitPdaArgs {
                seeds,
                space: VaultV1::LEN,
                program_id: self.program_id,
            },
        )?;

        AssociatedTokenProgram::init_if_needed(InitAssociatedTokenProgramAccounts {
            payer: self.accounts.admin,
            wallet: self.accounts.vault_pda,
            mint: self.accounts.token_mint,
            token_program: self.accounts.token_program,
            associated_token_program: self.accounts.associated_token_program,
            system_program: self.accounts.system_program,
            ata: self.accounts.vault_ata,
        })?;

        TokenProgram::transfer(
            TokenTransferAccounts {
                source: self.accounts.admin_ata,
                destination: self.accounts.vault_ata,
                authority: self.accounts.admin,
                mint: self.accounts.token_mint,
                token_program: self.accounts.token_program,
            },
            TokenTransferArgs {
                amount: project.escrow_amount,
                decimals: project.mint_decimals,
            },
        )
    }

    fn pay_protocol_fee(&self, project: &ProjectV1) -> ProgramResult {
        if project.is_free_mint_nft_fee() {
            return Ok(());
        }

        SystemProgram::transfer(
            self.accounts.admin,
            self.accounts.protocol_wallet,
            self.accounts.system_program,
            project.mint_nft_fee_lamports,
        )
    }

    fn mint_nft(self, project: &mut ProjectV1) -> ProgramResult {
        MplCoreProgram::create(
            CreateMplCoreAssetAccounts {
                payer: self.accounts.admin,
                asset: self.accounts.nft_asset,
                collection: self.accounts.nft_collection,
                authority: Some(self.accounts.nft_authority),
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            CreateMplCoreAssetArgs {
                name: self.instruction_data.nft_name,
                uri: self.instruction_data.nft_uri,
            },
            &[&[NftAuthorityV1::SEED, &[self.nft_authority_bump]]],
        )?;

        project.increment_admin_minted()?;

        Ok(())
    }
}

impl<'a, 'info> ProcessInstruction for MintAdminV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let mut project_data = self.accounts.project_pda.try_borrow_mut_data()?;
        let project = ProjectV1::load_mut(project_data.as_mut())?;

        self.check_mint_eligibility(project)?;
        self.store_to_vault(project)?;
        self.pay_protocol_fee(project)?;
        self.mint_nft(project)
    }
}

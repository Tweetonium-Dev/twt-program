use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{NftAuthorityV1, ProjectV1},
    utils::{
        AccountCheck, MintAccount, MplCoreProgram, Pda, ProcessInstruction, ProjectAccount,
        SignerAccount, SystemProgram, UpdateMplCoreAssetAccounts, UpdateMplCoreAssetArgs,
        WritableAccount,
    },
};

#[derive(Debug)]
pub struct UpdateNftV1Accounts<'a, 'info> {
    /// Authority allowed to update the NFT (e.g. update authority).
    /// Must be signer if required by MPL Core.
    pub payer: &'a AccountInfo<'info>,

    /// PDA: `["project_v1", nft_collection, token_mint, program_id]` — stores global project config.
    /// Must be readable, owned by program.
    pub project_pda: &'a AccountInfo<'info>,

    /// Token mint (fungible token used for minting/refunding e.g. ZDLT).
    /// Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022.
    pub token_mint: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "nft_authority"]`
    /// Controls: update/burn all NFTs.
    /// Only program can sign
    pub nft_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this project.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub nft_collection: &'a AccountInfo<'info>,

    /// NFT asset (MPL Core) — the asset being updated.
    /// Must be mutable, owned by `mpl_core`.
    pub nft_asset: &'a AccountInfo<'info>,

    /// Protocol wallet — receives the configurable SOL protocol fee.
    /// Must writable, not zero address, owned by system_program.
    pub protocol_wallet: &'a AccountInfo<'info>,

    /// System program — for potential realloc.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — performs the update.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for UpdateNftV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [payer, project_pda, token_mint, nft_authority, nft_collection, nft_asset, protocol_wallet, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(payer)?;

        WritableAccount::check(project_pda)?;
        WritableAccount::check(nft_asset)?;
        WritableAccount::check(protocol_wallet)?;

        ProjectAccount::check(project_pda)?;
        MintAccount::check(token_mint)?;
        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            payer,
            project_pda,
            token_mint,
            nft_authority,
            nft_collection,
            nft_asset,
            protocol_wallet,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct UpdateNftV1InstructionData {
    pub nft_name: String,
    pub nft_uri: String,
}

#[derive(Debug)]
pub struct UpdateNftV1<'a, 'info> {
    pub accounts: UpdateNftV1Accounts<'a, 'info>,
    pub instruction_data: UpdateNftV1InstructionData,
    pub nft_authority_bump: u8,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        UpdateNftV1InstructionData,
        &'a Pubkey,
    )> for UpdateNftV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            UpdateNftV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = UpdateNftV1Accounts::try_from(accounts)?;

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
            nft_authority_bump,
        })
    }
}

impl<'a, 'info> UpdateNftV1<'a, 'info> {
    fn check_ownership(&self) -> ProgramResult {
        let asset_owner = MplCoreProgram::get_asset_owner(self.accounts.nft_asset)?;

        if asset_owner != *self.accounts.payer.key {
            msg!("Signer is not the NFT owner");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    fn pay_protocol_fee(&self, project: &ProjectV1) -> ProgramResult {
        if project.is_free_update_nft_fee() {
            return Ok(());
        }

        SystemProgram::transfer(
            self.accounts.payer,
            self.accounts.protocol_wallet,
            self.accounts.system_program,
            project.update_nft_fee_lamports,
        )
    }

    fn update_nft(self) -> ProgramResult {
        MplCoreProgram::update(
            UpdateMplCoreAssetAccounts {
                asset: self.accounts.nft_asset,
                collection: self.accounts.nft_collection,
                payer: self.accounts.payer,
                update_authority: self.accounts.nft_authority,
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            UpdateMplCoreAssetArgs {
                name: self.instruction_data.nft_name,
                uri: self.instruction_data.nft_uri,
            },
            &[&[NftAuthorityV1::SEED, &[self.nft_authority_bump]]],
        )
    }
}

impl<'a, 'info> ProcessInstruction for UpdateNftV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        let project_data = self.accounts.project_pda.data.borrow_mut();
        let project = ProjectV1::load(&project_data)?;

        self.check_ownership()?;
        self.pay_protocol_fee(project)?;
        self.update_nft()
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{TraitAuthorityV1, TraitItemV1, UpdateTraitItemArgs},
    utils::{
        AccountCheck, MplCoreProgram, Pda, ProcessInstruction, SignerAccount, SystemProgram,
        UpdateMplCoreCollectionAccounts, UpdateMplCoreCollectionArgs, WritableAccount,
    },
};

#[derive(Debug)]
pub struct UpdateTraitV1Accounts<'a, 'info> {
    /// Authority that will control trait (e.g. protocol wallet).
    /// Must be a signer.
    pub authority: &'a AccountInfo<'info>,

    /// PDA: `[program_id, trait_collection, "config"]` — stores `TraitItem` struct.
    /// Must be uninitialized, writable, owned by this program.
    pub trait_pda: &'a AccountInfo<'info>,

    /// PDA: `[program_id, "trait_authority"]`
    /// Controls: update/burn all trait NFTs.
    /// Only program can sign.
    pub trait_authority: &'a AccountInfo<'info>,

    /// MPL Core Collection account that groups NFTs under this trait.
    /// Must be initialized before trait creation via `CreateV1CpiBuilder`.
    /// Determines the project scope for mint rules, royalties, and limits.
    pub trait_collection: &'a AccountInfo<'info>,

    /// System program — required for PDA creation and rent.
    pub system_program: &'a AccountInfo<'info>,

    /// Metaplex Core program — for NFT minting.
    /// Must be the official MPL Core program.
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for UpdateTraitV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, trait_pda, trait_authority, trait_collection, system_program, mpl_core] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;

        WritableAccount::check(trait_pda)?;
        WritableAccount::check(trait_collection)?;

        SystemProgram::check(system_program)?;
        MplCoreProgram::check(mpl_core)?;

        Ok(Self {
            authority,
            trait_pda,
            trait_authority,
            trait_collection,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct UpdateTraitV1InstructionData {
    pub max_supply: u64,
    pub mint_fee_lamports: u64,
    pub trait_name: String,
    pub trait_uri: String,
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; 5],
    pub royalty_shares_bps: [u16; 5],
}

#[derive(Debug)]
pub struct UpdateTraitV1<'a, 'info> {
    pub accounts: UpdateTraitV1Accounts<'a, 'info>,
    pub instruction_data: UpdateTraitV1InstructionData,
    pub trait_authority_bump: u8,
}

impl<'a, 'info> UpdateTraitV1<'a, 'info> {
    fn check_trait_royalties(&self) -> ProgramResult {
        TraitItemV1::check_trait_royalties(
            self.instruction_data.num_royalty_recipients,
            self.instruction_data.royalty_recipients,
            self.instruction_data.royalty_shares_bps,
        )
    }

    fn update_trait(&self) -> ProgramResult {
        let mut trait_data = self.accounts.trait_pda.try_borrow_mut_data()?;
        let trait_item = TraitItemV1::load_mut(trait_data.as_mut())?;

        if trait_item.authority != *self.accounts.authority.key {
            msg!("Unauthorized authority for trait update");
            return Err(ProgramError::InvalidAccountData);
        }

        trait_item.update(UpdateTraitItemArgs {
            max_supply: self.instruction_data.max_supply,
            mint_fee_lamports: self.instruction_data.mint_fee_lamports,
        });

        Ok(())
    }

    fn update_collection(self) -> ProgramResult {
        MplCoreProgram::update_collection(
            UpdateMplCoreCollectionAccounts {
                payer: self.accounts.authority,
                collection: self.accounts.trait_collection,
                update_authority: self.accounts.trait_authority,
                mpl_core: self.accounts.mpl_core,
                system_program: self.accounts.system_program,
            },
            UpdateMplCoreCollectionArgs {
                num_royalty_recipients: self.instruction_data.num_royalty_recipients,
                royalty_recipients: self.instruction_data.royalty_recipients,
                royalty_shares_bps: self.instruction_data.royalty_shares_bps,
                name: self.instruction_data.trait_name,
                uri: self.instruction_data.trait_uri,
            },
            &[&[TraitAuthorityV1::SEED, &[self.trait_authority_bump]]],
        )
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        UpdateTraitV1InstructionData,
        &'a Pubkey,
    )> for UpdateTraitV1<'a, 'info>
{
    type Error = ProgramError;
    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            UpdateTraitV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = UpdateTraitV1Accounts::try_from(accounts)?;

        Pda::validate(
            accounts.trait_pda,
            &[TraitItemV1::SEED, accounts.trait_collection.key.as_ref()],
            program_id,
        )?;

        let (_, trait_authority_bump) = Pda::validate(
            accounts.trait_authority,
            &[TraitAuthorityV1::SEED],
            program_id,
        )?;

        Ok(Self {
            accounts,
            instruction_data,
            trait_authority_bump,
        })
    }
}

impl<'a, 'info> ProcessInstruction for UpdateTraitV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.check_trait_royalties()?;
        self.update_trait()?;
        self.update_collection()
    }
}

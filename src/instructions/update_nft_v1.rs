use borsh::{BorshDeserialize, BorshSerialize};
use mpl_core::{instructions::UpdateV1CpiBuilder, Asset};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::utils::{
    AccountCheck, MplCoreAccount, ProcessInstruction, SignerAccount, SystemAccount, WritableAccount
};

#[derive(Debug)]
pub struct UpdateNftV1Accounts<'a, 'info> {
    pub authority: &'a AccountInfo<'info>,
    pub nft_asset: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
    pub mpl_core: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for UpdateNftV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [authority, nft_asset, system_program, mpl_core] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(authority)?;
        WritableAccount::check(nft_asset)?;
        SystemAccount::check(system_program)?;
        MplCoreAccount::check(mpl_core)?;

        Ok(Self {
            authority,
            nft_asset,
            system_program,
            mpl_core,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct UpdateNftV1InstructionData {
    pub name: String,
    pub uri: String,
}

#[derive(Debug)]
pub struct UpdateNftV1<'a, 'info> {
    pub accounts: UpdateNftV1Accounts<'a, 'info>,
    pub instruction_data: UpdateNftV1InstructionData,
}

impl<'a, 'info> UpdateNftV1<'a, 'info> {
    fn check_ownership(&self) -> ProgramResult {
        let authority = self.accounts.authority;
        let nft_asset = self.accounts.nft_asset;

        let asset_data = &nft_asset.data.borrow();
        let asset = Asset::deserialize(&asset_data[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if asset.base.owner != *authority.key {
            msg!("Signer is not the NFT owner");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    fn update_nft(&self) -> ProgramResult {
        let authority = self.accounts.authority;
        let nft_asset = self.accounts.nft_asset;
        let system_program = self.accounts.system_program;
        let mpl_core = self.accounts.mpl_core;

        UpdateV1CpiBuilder::new(mpl_core)
            .asset(nft_asset)
            .authority(Some(authority))
            .system_program(system_program)
            .new_name(self.instruction_data.name.to_string())
            .new_uri(self.instruction_data.uri.to_string())
            .invoke()?;

        Ok(())
    }
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        UpdateNftV1InstructionData,
    )> for UpdateNftV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data): (
            &'a [AccountInfo<'info>],
            UpdateNftV1InstructionData,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = UpdateNftV1Accounts::try_from(accounts)?;
        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a, 'info> ProcessInstruction for UpdateNftV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.check_ownership()?;
        self.update_nft()?;

        msg!(
            "UpdateNft: updated NFT {} with name='{}', uri='{}'",
            self.accounts.nft_asset.key,
            self.instruction_data.name,
            self.instruction_data.uri
        );

        Ok(())
    }
}

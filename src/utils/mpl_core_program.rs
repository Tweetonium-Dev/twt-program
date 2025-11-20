use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{
        BurnV1CpiBuilder, CreateCollectionV2CpiBuilder, CreateV2CpiBuilder,
        UpdateCollectionPluginV1CpiBuilder, UpdateCollectionV1CpiBuilder, UpdateV1CpiBuilder,
    },
    types::{
        Creator, PermanentBurnDelegate, Plugin, PluginAuthority, PluginAuthorityPair, Royalties,
        RuleSet,
    },
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{states::MAX_ROYALTY_RECIPIENTS, utils::AccountCheck};

pub struct MplCoreProgram;

impl MplCoreProgram {
    pub fn get_asset_owner<'info>(account: &AccountInfo<'info>) -> Result<Pubkey, ProgramError> {
        let data = account.try_borrow_data()?;
        let base = BaseAssetV1::from_bytes(&data).map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(base.owner)
    }

    pub fn get_royalties(
        num_royalty_recipients: u8,
        royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
        royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    ) -> Option<Royalties> {
        if num_royalty_recipients == 0 {
            return None;
        }

        let total_bps = royalty_shares_bps
            .iter()
            .take(num_royalty_recipients as usize)
            .sum::<u16>();

        if total_bps == 0 {
            return None;
        }

        let creators: Vec<Creator> = royalty_recipients
            .iter()
            .zip(royalty_shares_bps.iter())
            .take(num_royalty_recipients as usize)
            .filter(|(pk, bps)| **bps > 0 && **pk != Pubkey::default())
            .map(|(pk, bps)| Creator {
                address: *pk,
                percentage: if total_bps == 0 {
                    0
                } else {
                    let bps = (*bps as u64) * 100;
                    let total_bps = total_bps as u64;
                    (bps / total_bps) as u8
                },
            })
            .collect();

        if creators.is_empty() {
            return None;
        }

        Some(Royalties {
            basis_points: total_bps,
            creators,
            rule_set: RuleSet::None,
        })
    }

    pub fn init_collection<'a, 'info>(
        accounts: InitMplCoreCollectionAccounts<'a, 'info>,
        args: InitMplCoreCollectionArgs,
    ) -> ProgramResult {
        let mut cpi = CreateCollectionV2CpiBuilder::new(accounts.mpl_core);

        cpi.collection(accounts.collection)
            .payer(accounts.payer)
            .update_authority(accounts.update_authority)
            .system_program(accounts.system_program)
            .name(args.name)
            .uri(args.uri);

        let mut plugins: Vec<PluginAuthorityPair> = vec![PluginAuthorityPair {
            plugin: Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            authority: Some(PluginAuthority::UpdateAuthority),
        }];

        if let Some(royalties) = Self::get_royalties(
            args.num_royalty_recipients,
            args.royalty_recipients,
            args.royalty_shares_bps,
        ) {
            plugins.push(PluginAuthorityPair {
                plugin: Plugin::Royalties(royalties),
                authority: Some(PluginAuthority::UpdateAuthority),
            });
        }

        cpi.plugins(plugins).invoke()
    }

    pub fn update_collection<'a, 'info>(
        accounts: UpdateMplCoreCollectionAccounts<'a, 'info>,
        args: UpdateMplCoreCollectionArgs,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        UpdateCollectionV1CpiBuilder::new(accounts.mpl_core)
            .collection(accounts.collection)
            .payer(accounts.payer)
            .authority(Some(accounts.update_authority))
            .system_program(accounts.system_program)
            .new_name(args.name)
            .new_uri(args.uri)
            .invoke_signed(signers_seeds)?;

        if let Some(royalties) = Self::get_royalties(
            args.num_royalty_recipients,
            args.royalty_recipients,
            args.royalty_shares_bps,
        ) {
            UpdateCollectionPluginV1CpiBuilder::new(accounts.mpl_core)
                .collection(accounts.collection)
                .payer(accounts.payer)
                .authority(Some(accounts.update_authority))
                .system_program(accounts.system_program)
                .plugin(Plugin::Royalties(royalties))
                .invoke_signed(signers_seeds)?
        }

        Ok(())
    }

    pub fn create<'a, 'info>(
        accounts: CreateMplCoreAssetAccounts<'a, 'info>,
        args: CreateMplCoreAssetArgs,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        CreateV2CpiBuilder::new(accounts.mpl_core)
            .asset(accounts.asset)
            .collection(Some(accounts.collection))
            .payer(accounts.payer)
            .authority(accounts.authority)
            .owner(Some(accounts.payer))
            .system_program(accounts.system_program)
            .name(args.name)
            .uri(args.uri)
            .invoke_signed(signer_seeds)
    }

    pub fn update<'a, 'info>(
        accounts: UpdateMplCoreAssetAccounts<'a, 'info>,
        args: UpdateMplCoreAssetArgs,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        UpdateV1CpiBuilder::new(accounts.mpl_core)
            .asset(accounts.asset)
            .collection(Some(accounts.collection))
            .payer(accounts.payer)
            .authority(Some(accounts.update_authority))
            .system_program(accounts.system_program)
            .new_name(args.name)
            .new_uri(args.uri)
            .invoke_signed(signers_seeds)
    }

    pub fn burn<'a, 'info>(
        accounts: BurnMplCoreAssetAccounts<'a, 'info>,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        BurnV1CpiBuilder::new(accounts.mpl_core)
            .collection(Some(accounts.collection))
            .asset(accounts.asset)
            .payer(accounts.payer)
            .authority(Some(accounts.update_authority))
            .system_program(Some(accounts.system_program))
            .invoke_signed(signers_seeds)
    }
}

impl AccountCheck for MplCoreProgram {
    fn check<'info>(account: &AccountInfo<'info>) -> ProgramResult {
        if *account.key != mpl_core::ID {
            msg!("Mpl core invalid");
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }
}

pub struct InitMplCoreCollectionAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub collection: &'a AccountInfo<'info>,
    pub update_authority: Option<&'a AccountInfo<'info>>,
    pub mpl_core: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct InitMplCoreCollectionArgs {
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
    pub royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    pub name: String,
    pub uri: String,
}

pub struct UpdateMplCoreCollectionAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub collection: &'a AccountInfo<'info>,
    pub update_authority: &'a AccountInfo<'info>,
    pub mpl_core: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct UpdateMplCoreCollectionArgs {
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
    pub royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    pub name: String,
    pub uri: String,
}

pub struct CreateMplCoreAssetAccounts<'a, 'info> {
    pub payer: &'a AccountInfo<'info>,
    pub asset: &'a AccountInfo<'info>,
    pub collection: &'a AccountInfo<'info>,
    pub authority: Option<&'a AccountInfo<'info>>,
    pub mpl_core: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct CreateMplCoreAssetArgs {
    pub name: String,
    pub uri: String,
}

pub struct UpdateMplCoreAssetAccounts<'a, 'info> {
    pub asset: &'a AccountInfo<'info>,
    pub collection: &'a AccountInfo<'info>,
    pub payer: &'a AccountInfo<'info>,
    pub update_authority: &'a AccountInfo<'info>,
    pub mpl_core: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

pub struct UpdateMplCoreAssetArgs {
    pub name: String,
    pub uri: String,
}

pub struct BurnMplCoreAssetAccounts<'a, 'info> {
    pub asset: &'a AccountInfo<'info>,
    pub collection: &'a AccountInfo<'info>,
    pub payer: &'a AccountInfo<'info>,
    pub update_authority: &'a AccountInfo<'info>,
    pub mpl_core: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        mock::{default_pubkeys, mock_account, mock_u16s},
        mock_base_asset,
    };

    // --- Test Helpers ---

    fn mock_account_info(key: Pubkey, data: Vec<u8>) -> AccountInfo<'static> {
        crate::utils::mock::mock_account_with_data(key, false, true, 0, data, Pubkey::new_unique())
    }

    fn mock_mpl_asset(owner: Pubkey, name: &str, uri: &str) -> AccountInfo<'static> {
        crate::utils::mock::mock_account_with_data(
            Pubkey::new_unique(),
            false,
            true,
            1,
            mock_base_asset(owner, name, uri),
            mpl_core::ID,
        )
    }

    // --- Test Cases ---

    #[test]
    fn test_get_asset_success() {
        let owner = Pubkey::new_unique();
        let name = "Test NFT";
        let uri = "https://example.com";
        let account = mock_mpl_asset(owner, name, uri);

        let asset_owner = MplCoreProgram::get_asset_owner(&account).expect("Failed to get asset");
        assert_eq!(asset_owner, owner);
    }

    #[test]
    fn test_get_asset_invalid_data() {
        let key = Pubkey::new_unique();
        let data = vec![1, 2, 3, 4];
        let account = mock_account_info(key, data);

        let result = MplCoreProgram::get_asset_owner(&account);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_get_royalties() {
        let mut recipients = default_pubkeys::<MAX_ROYALTY_RECIPIENTS>();
        recipients[0] = Pubkey::new_unique();
        recipients[1] = Pubkey::new_unique();

        let mut bps = mock_u16s::<MAX_ROYALTY_RECIPIENTS>(0);
        bps[0] = 1000;
        bps[1] = 500;

        let result = MplCoreProgram::get_royalties(2, recipients, bps);
        assert!(result.is_some());

        let royalties = result.unwrap();
        assert_eq!(royalties.creators.len(), 2);
        assert_eq!(royalties.basis_points, 1500);
    }

    #[test]
    fn test_check_mpl_core_program() {
        let acc = mock_account(mpl_core::ID, false, false, 1, 0, Pubkey::default());
        assert!(MplCoreProgram::check(&acc).is_ok());

        let acc = mock_account(Pubkey::new_unique(), false, false, 1, 0, Pubkey::default());
        assert_eq!(
            MplCoreProgram::check(&acc).unwrap_err(),
            ProgramError::IncorrectProgramId
        );
    }
}

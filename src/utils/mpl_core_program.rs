use mpl_core::{
    instructions::{
        CreateCollectionV2CpiBuilder, CreateV2CpiBuilder, UpdateCollectionPluginV1CpiBuilder,
        UpdateCollectionV1CpiBuilder,
    },
    types::{Creator, Plugin, PluginAuthority, PluginAuthorityPair, Royalties, RuleSet},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{states::MAX_ROYALTY_RECIPIENTS, utils::AccountCheck};

pub struct MplCoreProgram;

impl MplCoreProgram {
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

    pub fn init_collection<'info>(
        collection: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        mpl_core: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        args: InitMplCoreCollectionArgs,
    ) -> ProgramResult {
        let mut cpi = CreateCollectionV2CpiBuilder::new(mpl_core);

        cpi.collection(collection)
            .payer(authority)
            .update_authority(Some(authority))
            .system_program(system_program)
            .name(args.name)
            .uri(args.uri);

        if let Some(royalties) = Self::get_royalties(
            args.num_royalty_recipients,
            args.royalty_recipients,
            args.royalty_shares_bps,
        ) {
            let royalties_plugin = PluginAuthorityPair {
                plugin: Plugin::Royalties(royalties),
                authority: Some(PluginAuthority::UpdateAuthority),
            };
            cpi.plugins(vec![royalties_plugin]);
        }

        cpi.invoke()
    }

    pub fn update_collection<'info>(
        collection: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        mpl_core: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        args: UpdateMplCoreCollectionArgs,
    ) -> ProgramResult {
        UpdateCollectionV1CpiBuilder::new(mpl_core)
            .collection(collection)
            .payer(authority)
            .authority(Some(authority))
            .system_program(system_program)
            .new_name(args.name)
            .new_uri(args.uri)
            .invoke()?;

        if let Some(royalties) = Self::get_royalties(
            args.num_royalty_recipients,
            args.royalty_recipients,
            args.royalty_shares_bps,
        ) {
            UpdateCollectionPluginV1CpiBuilder::new(mpl_core)
                .collection(collection)
                .payer(authority)
                .authority(Some(authority))
                .system_program(system_program)
                .plugin(Plugin::Royalties(royalties))
                .invoke()?;
        }

        Ok(())
    }

    pub fn create<'info>(
        asset: &AccountInfo<'info>,
        collection: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        update_authority: Option<&AccountInfo<'info>>,
        mpl_core: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        args: InitMplCoreAssetArgs,
    ) -> ProgramResult {
        CreateV2CpiBuilder::new(mpl_core)
            .asset(asset)
            .collection(Some(collection))
            .payer(authority)
            .authority(Some(authority))
            .owner(Some(authority))
            .update_authority(update_authority)
            .system_program(system_program)
            .name(args.name)
            .uri(args.uri)
            .invoke()
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

pub struct InitMplCoreCollectionArgs {
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
    pub royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    pub name: String,
    pub uri: String,
}

pub struct UpdateMplCoreCollectionArgs {
    pub num_royalty_recipients: u8,
    pub royalty_recipients: [Pubkey; MAX_ROYALTY_RECIPIENTS],
    pub royalty_shares_bps: [u16; MAX_ROYALTY_RECIPIENTS],
    pub name: String,
    pub uri: String,
}

pub struct InitMplCoreAssetArgs {
    pub name: String,
    pub uri: String,
}

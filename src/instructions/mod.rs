mod burn_and_refund_v1;
mod force_unlock_vesting_v1;
mod init_config_v1;
mod init_trait_v1;
mod mint_admin_v1;
mod mint_trait_v1;
mod mint_user_v1;
mod mint_vip_v1;
mod update_nft_v1;

pub use burn_and_refund_v1::*;
pub use force_unlock_vesting_v1::*;
pub use init_config_v1::*;
pub use init_trait_v1::*;
pub use mint_admin_v1::*;
pub use mint_trait_v1::*;
pub use mint_user_v1::*;
pub use mint_vip_v1::*;
pub use update_nft_v1::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum TweetoniumInstruction {
    InitConfigV1(InitConfigV1InstructionData),
    MintAdminV1(MintAdminV1InstructionData),
    MintUserV1(MintUserV1InstructionData),
    MintVipV1(MintVipV1InstructionData),
    InitTraitV1(InitTraitV1InstructionData),
    MintTraitV1(MinTraitV1InstructionData),
    UpdateNftV1(UpdateNftV1InstructionData),
    BurnAndRefundV1,
    ForceUnlockVestingV1,
}

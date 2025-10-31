mod burn_and_refund_v1;
mod init_config_v1;
mod mint_and_vault_v1;
mod update_nft_v1;

pub use burn_and_refund_v1::*;
pub use init_config_v1::*;
pub use mint_and_vault_v1::*;
pub use update_nft_v1::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum TweetoniumInstruction {
    InitConfigV1(InitConfigV1InstructionData),
    MintAndVaultV1(MintAndVaultV1InstructionData),
    UpdateNftV1(UpdateNftV1InstructionData),
    BurnAndRefundV1,
}

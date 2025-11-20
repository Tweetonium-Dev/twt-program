mod burn_and_refund_v1;
mod force_unlock_vesting_v1;
mod init_config_v1;
mod init_trait_v1;
mod mint_admin_v1;
mod mint_trait_v1;
mod mint_user_v1;
mod mint_vip_v1;
mod update_config_v1;
mod update_nft_v1;
mod update_trait_v1;

pub use burn_and_refund_v1::*;
pub use force_unlock_vesting_v1::*;
pub use init_config_v1::*;
pub use init_trait_v1::*;
pub use mint_admin_v1::*;
pub use mint_trait_v1::*;
pub use mint_user_v1::*;
pub use mint_vip_v1::*;
pub use update_config_v1::*;
pub use update_nft_v1::*;
pub use update_trait_v1::*;

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum TweetoniumInstruction {
    #[account(
        0,
        signer,
        name = "admin",
        desc = "Authority that will control config updates (e.g. admin wallet)."
    )]
    #[account(
        1,
        name = "nft_authority",
        desc = "PDA that have authority control of nft mint, updates, and burn."
    )]
    #[account(
        2,
        signer,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        3,
        writable,
        name = "config_pda",
        desc = "Uninitialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        4,
        name = "token_mint",
        desc = "Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022."
    )]
    #[account(
        5,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        6,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    InitConfigV1(InitConfigV1InstructionData),

    #[account(
        0,
        signer,
        name = "admin",
        desc = "Authority that will control config updates (e.g. admin wallet)."
    )]
    #[account(
        1,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        2,
        name = "nft_authority",
        desc = "PDA that have authority control of nft mint, updates, and burn."
    )]
    #[account(
        3,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        4,
        name = "token_mint",
        desc = "Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022."
    )]
    #[account(
        5,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        6,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    UpdateConfigV1(UpdateConfigV1InstructionData),

    #[account(
        0,
        signer,
        name = "admin",
        desc = "Authority as payer (admin wallet). Must sign."
    )]
    #[account(
        1,
        writable,
        name = "admin_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        2,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        3,
        writable,
        name = "vault_pda",
        desc = "Uninitialized vault pda with seeds [\"vault_v1\", nft_asset, nft_collection, token_mint, program_id]"
    )]
    #[account(
        4,
        writable,
        name = "vault_ata",
        desc = "Vault PDA's associated token account — holds escrowed 'token_mint' funds."
    )]
    #[account(5, name = "nft_authority", desc = "Controls: update all NFTs.")]
    #[account(
        6,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        7,
        signer,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        8,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        9,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        10,
        name = "associated_token_program",
        desc = "Associated Token Program — for ATA derivation and creation."
    )]
    #[account(
        11,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        12,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        13,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    MintAdminV1(MintAdminV1InstructionData),

    #[account(
        0,
        signer,
        name = "payer",
        desc = "User paying the mint price in 'token_mint' and solana."
    )]
    #[account(
        1,
        writable,
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        2,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        3,
        writable,
        name = "vault_pda",
        desc = "Uninitialized vault pda with seeds [\"vault_v1\", nft_asset, nft_collection, token_mint, program_id]"
    )]
    #[account(
        4,
        writable,
        name = "vault_ata",
        desc = "Associated Token Account (ATA) of the vault PDA."
    )]
    #[account(
        5,
        writable,
        name = "user_minted_pda",
        desc = "Uninitialize user mint pda with seeds [\"user_minted_v1\", nft_collection, token_mint, payer, program_id]"
    )]
    #[account(6, name = "nft_authority", desc = "Controls: update all NFTs.")]
    #[account(
        7,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        8,
        signer,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        9,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        10,
        writable,
        name = "revenue_wallet_0",
        desc = "Revenue wallet #0 — corresponds to config.revenue_wallet(0)."
    )]
    #[account(
        11,
        writable,
        name = "revenue_wallet_ata_0",
        desc = "ATA for revenue wallet #0 — receives share from mint price."
    )]
    #[account(
        12,
        writable,
        name = "revenue_wallet_1",
        desc = "Revenue wallet #1 — corresponds to config.revenue_wallet(1)."
    )]
    #[account(
        13,
        writable,
        name = "revenue_wallet_ata_1",
        desc = "ATA for revenue wallet #1 — receives share from mint price."
    )]
    #[account(
        14,
        writable,
        name = "revenue_wallet_2",
        desc = "Revenue wallet #2 — corresponds to config.revenue_wallet(2)."
    )]
    #[account(
        15,
        writable,
        name = "revenue_wallet_ata_2",
        desc = "ATA for revenue wallet #2 — receives share from mint price."
    )]
    #[account(
        16,
        writable,
        name = "revenue_wallet_3",
        desc = "Revenue wallet #3 — corresponds to config.revenue_wallet(3)."
    )]
    #[account(
        17,
        writable,
        name = "revenue_wallet_ata_3",
        desc = "ATA for revenue wallet #3 — receives share from mint price."
    )]
    #[account(
        18,
        writable,
        name = "revenue_wallet_4",
        desc = "Revenue wallet #4 — corresponds to config.revenue_wallet(4)."
    )]
    #[account(
        19,
        writable,
        name = "revenue_wallet_ata_4",
        desc = "ATA for revenue wallet #4 — receives share from mint price."
    )]
    #[account(
        20,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        21,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        22,
        name = "associated_token_program",
        desc = "Associated Token Program"
    )]
    #[account(
        23,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        24,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    MintUserV1(MintUserV1InstructionData),

    #[account(
        0,
        signer,
        name = "payer",
        desc = "User paying the mint price in 'token_mint' and solana."
    )]
    #[account(
        1,
        writable,
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        2,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        3,
        writable,
        name = "vault_pda",
        desc = "Uninitialized vault pda with seeds [\"vault_v1\", nft_asset, nft_collection, token_mint, program_id]"
    )]
    #[account(
        4,
        writable,
        name = "vault_ata",
        desc = "Associated Token Account (ATA) of the vault PDA."
    )]
    #[account(
        5,
        writable,
        name = "user_minted_pda",
        desc = "Uninitialize user mint pda with seeds [\"user_minted_v1\", nft_collection, token_mint, payer, program_id]"
    )]
    #[account(6, name = "nft_authority", desc = "Controls: update all NFTs.")]
    #[account(
        7,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        8,
        signer,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        9,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        10,
        writable,
        name = "revenue_wallet_0",
        desc = "Revenue wallet #0 — corresponds to config.revenue_wallet(0)."
    )]
    #[account(
        11,
        writable,
        name = "revenue_wallet_ata_0",
        desc = "ATA for revenue wallet #0 — receives share from mint price."
    )]
    #[account(
        12,
        writable,
        name = "revenue_wallet_1",
        desc = "Revenue wallet #1 — corresponds to config.revenue_wallet(1)."
    )]
    #[account(
        13,
        writable,
        name = "revenue_wallet_ata_1",
        desc = "ATA for revenue wallet #1 — receives share from mint price."
    )]
    #[account(
        14,
        writable,
        name = "revenue_wallet_2",
        desc = "Revenue wallet #2 — corresponds to config.revenue_wallet(2)."
    )]
    #[account(
        15,
        writable,
        name = "revenue_wallet_ata_2",
        desc = "ATA for revenue wallet #2 — receives share from mint price."
    )]
    #[account(
        16,
        writable,
        name = "revenue_wallet_3",
        desc = "Revenue wallet #3 — corresponds to config.revenue_wallet(3)."
    )]
    #[account(
        17,
        writable,
        name = "revenue_wallet_ata_3",
        desc = "ATA for revenue wallet #3 — receives share from mint price."
    )]
    #[account(
        18,
        writable,
        name = "revenue_wallet_4",
        desc = "Revenue wallet #4 — corresponds to config.revenue_wallet(4)."
    )]
    #[account(
        19,
        writable,
        name = "revenue_wallet_ata_4",
        desc = "ATA for revenue wallet #4 — receives share from mint price."
    )]
    #[account(
        20,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        21,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        22,
        name = "associated_token_program",
        desc = "Associated Token Program"
    )]
    #[account(
        23,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        24,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    MintVipV1(MintVipV1InstructionData),

    #[account(
        0,
        signer,
        name = "authority",
        desc = "Authority that will control trait updates (e.g. protocol wallet)."
    )]
    #[account(
        1,
        writable,
        name = "trait_pda",
        desc = "Uninitialize config pda with seeds [\"trait_item_v1\", trait_collection, program_id]"
    )]
    #[account(
        2,
        name = "trait_authority",
        desc = "PDA that have authority control of trait nft mint, updates, and burn."
    )]
    #[account(
        3,
        signer,
        writable,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        4,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        5,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    InitTraitV1(InitTraitV1InstructionData),

    #[account(
        0,
        signer,
        name = "authority",
        desc = "Authority that will control trait updates (e.g. protocol wallet)."
    )]
    #[account(
        1,
        writable,
        name = "trait_pda",
        desc = "Initialized config pda with seeds [\"trait_item_v1\", trait_collection, program_id]"
    )]
    #[account(
        2,
        name = "trait_authority",
        desc = "PDA that have authority control of trait nft mint, updates, and burn."
    )]
    #[account(
        3,
        writable,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        4,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        5,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    UpdateTraitV1(UpdateTraitV1InstructionData),

    #[account(
        0,
        signer,
        name = "payer",
        desc = "User paying the mint price in 'token_mint' and solana."
    )]
    #[account(
        1,
        writable,
        name = "trait_pda",
        desc = "Initialized config pda with seeds [\"trait_item_v1\", trait_collection, program_id]"
    )]
    #[account(
        2,
        name = "trait_authority",
        desc = "PDA that have authority control of trait nft mint, updates, and burn."
    )]
    #[account(
        3,
        writable,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        4,
        signer,
        writable,
        name = "trait_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        5,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        6,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        7,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    MintTraitV1(MintTraitV1InstructionData),

    #[account(
        0,
        signer,
        name = "payer",
        desc = "User paying the mint price in 'token_mint' and solana."
    )]
    #[account(
        1,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        2,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        3,
        name = "nft_authority",
        desc = "Authority to controls update for all NFTs."
    )]
    #[account(
        4,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        5,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        6,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        7,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        8,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    UpdateNftV1(UpdateNftV1InstructionData),

    #[account(
        0,
        signer,
        name = "payer",
        desc = "User paying the mint price in 'token_mint' and solana."
    )]
    #[account(
        1,
        writable,
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        2,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        3,
        writable,
        name = "vault_pda",
        desc = "Initialized vault pda with seeds [\"vault_v1\", nft_asset, nft_collection, token_mint, program_id]"
    )]
    #[account(
        4,
        writable,
        name = "vault_ata",
        desc = "Associated Token Account (ATA) of the vault PDA."
    )]
    #[account(5, name = "nft_authority", desc = "Controls: update all NFTs.")]
    #[account(
        6,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        7,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        8,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        9,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        10,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        11,
        name = "mpl_core",
        desc = "Metaplex Core program — must be the official MPL Core program."
    )]
    BurnAndRefundV1,

    #[account(
        0,
        signer,
        name = "admin",
        desc = "Authority that will control force unlock vesting (e.g. admin wallet)."
    )]
    #[account(
        1,
        writable,
        name = "config_pda",
        desc = "Initialized config pda with seeds [\"config_v1\", nft_collection, token_mint, program_id]"
    )]
    #[account(
        2,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        3,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    ForceUnlockVestingV1,
}

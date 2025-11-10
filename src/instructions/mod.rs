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
        signer,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        2,
        writable,
        name = "config_pda",
        desc = "Uninitialize config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        3,
        name = "token_mint",
        desc = "Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022."
    )]
    #[account(
        4,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
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
        desc = "The config authority — must sign and match `config.admin`."
    )]
    #[account(
        1,
        writable,
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project. Must be initialized before config creation."
    )]
    #[account(
        2,
        writable,
        name = "config_pda",
        desc = "Uninitialize config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        3,
        name = "token_mint",
        desc = "Must be valid mint (82 or 90+ bytes), owned by SPL Token or Token-2022."
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
        name = "config_pda",
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        2,
        writable,
        name = "vault_pda",
        desc = "Uninitialize vault pda with seeds [program_id, admin, token_mint, nft_collection, \"vault\"]"
    )]
    #[account(
        3,
        writable,
        name = "vault_ata",
        desc = "Vault PDA's associated token account — holds escrowed 'token_mint' funds."
    )]
    #[account(
        4,
        writable,
        name = "admin_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
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
        name = "config_pda",
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        2,
        writable,
        name = "vault_pda",
        desc = "Uninitialize vault pda with seeds [program_id, payer, token_mint, nft_collection, \"vault\"]"
    )]
    #[account(
        3,
        writable,
        name = "vault_ata",
        desc = "Associated Token Account (ATA) of the vault PDA."
    )]
    #[account(
        4,
        writable,
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        5,
        writable,
        name = "user_mint_pda",
        desc = "Uninitialize user mint pda with seeds [program_id, payer, token_mint, nft_collection, \"user_mint\"]"
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
        name = "revenue_wallet_ata_0",
        desc = "ATA for revenue wallet #0 — receives share from mint price."
    )]
    #[account(
        11,
        writable,
        name = "revenue_wallet_ata_1",
        desc = "ATA for revenue wallet #1 — receives share from mint price."
    )]
    #[account(
        12,
        writable,
        name = "revenue_wallet_ata_2",
        desc = "ATA for revenue wallet #2 — receives share from mint price."
    )]
    #[account(
        13,
        writable,
        name = "revenue_wallet_ata_3",
        desc = "ATA for revenue wallet #3 — receives share from mint price."
    )]
    #[account(
        14,
        writable,
        name = "revenue_wallet_ata_4",
        desc = "ATA for revenue wallet #4 — receives share from mint price."
    )]
    #[account(
        15,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        16,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        17,
        name = "associated_token_program",
        desc = "Associated Token Program"
    )]
    #[account(
        18,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        19,
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
        name = "config_pda",
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        2,
        writable,
        name = "vault_pda",
        desc = "Uninitialize vault pda with seeds [program_id, payer, token_mint, nft_collection, \"vault\"]"
    )]
    #[account(
        3,
        writable,
        name = "vault_ata",
        desc = "Associated Token Account (ATA) of the vault PDA."
    )]
    #[account(
        4,
        writable,
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        5,
        writable,
        name = "user_mint_pda",
        desc = "Uninitialize user mint pda with seeds [program_id, payer, token_mint, nft_collection, \"user_mint\"]"
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
        name = "revenue_wallet_ata_0",
        desc = "ATA for revenue wallet #0 — receives share from mint price."
    )]
    #[account(
        11,
        writable,
        name = "revenue_wallet_ata_1",
        desc = "ATA for revenue wallet #1 — receives share from mint price."
    )]
    #[account(
        12,
        writable,
        name = "revenue_wallet_ata_2",
        desc = "ATA for revenue wallet #2 — receives share from mint price."
    )]
    #[account(
        13,
        writable,
        name = "revenue_wallet_ata_3",
        desc = "ATA for revenue wallet #3 — receives share from mint price."
    )]
    #[account(
        14,
        writable,
        name = "revenue_wallet_ata_4",
        desc = "ATA for revenue wallet #4 — receives share from mint price."
    )]
    #[account(
        15,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
    )]
    #[account(
        16,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        17,
        name = "associated_token_program",
        desc = "Associated Token Program"
    )]
    #[account(
        18,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        19,
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
        desc = "Uninitialize config pda with seeds [program_id, trait_collection, \"trait_item\"]"
    )]
    #[account(
        2,
        signer,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        3,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        4,
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
        desc = "Uninitialize config pda with seeds [program_id, trait_collection, \"trait_item\"]"
    )]
    #[account(
        2,
        writable,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        3,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        4,
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
        desc = "Uninitialize config pda with seeds [program_id, trait_collection, \"trait_item\"]"
    )]
    #[account(
        2,
        writable,
        name = "trait_collection",
        desc = "MPL Core Collection account that groups trait NFTs."
    )]
    #[account(
        3,
        signer,
        name = "trait_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        4,
        writable,
        name = "protocol_wallet",
        desc = "Protocol wallet — receives the configurable SOL protocol fee."
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
    MintTraitV1(MinTraitV1InstructionData),

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
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        2,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        3,
        signer,
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
        signer,
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
        name = "nft_collection",
        desc = "MPL Core Collection account that groups NFTs under this project."
    )]
    #[account(
        2,
        writable,
        name = "nft_asset",
        desc = "Uninitialize NFT asset (MPL Core) — the NFT being minted."
    )]
    #[account(
        3,
        writable,
        name = "vault_pda",
        desc = "Uninitialize vault pda with seeds [program_id, payer, token_mint, nft_collection, \"vault\"]"
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
        name = "payer_ata",
        desc = "Admin's ATA for 'token_mint' — source of payment."
    )]
    #[account(
        6,
        name = "config_pda",
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
    )]
    #[account(
        7,
        name = "token_mint",
        desc = "Token mint — the token being escrowed (e.g. ZDLT)"
    )]
    #[account(
        8,
        name = "token_program",
        desc = "SPL Token Program (legacy) or Token-2022 Program."
    )]
    #[account(
        9,
        name = "system_program",
        desc = "System Program — required for PDA creation and rent."
    )]
    #[account(
        10,
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
        desc = "Initialized config pda with seeds [program_id, token_mint, nft_collection, \"config\"]"
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

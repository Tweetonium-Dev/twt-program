<p align="center">
  <img src="./assets/icon.png" width="1500" alt="Tweetonium" />
</p>

# Solana NFT Mint, Escrow, Vesting & Refund Protocol

[![Rust](https://img.shields.io/badge/Rust-F74C00?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Solana](https://img.shields.io/badge/Solana-14F195?logo=solana&logoColor=white)](https://solana.com)
[![MPL Core](https://img.shields.io/badge/Metaplex-MPL%20Core-6C5CE7?logo=metaplex)](https://developers.metaplex.com/core)

**Tweetonium** is an upgradeable Solana program that enables secure NFT minting with token-based payments, configurable vesting, per-wallet mint guards, and burn-to-refund mechanics — built using MPL Core, Shank, and pure Rust (no Anchor).

---

## Features

| Feature                           | Description                                                                                                                   |
| --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Mint NFTs with token payments** | Users pay with optional SOL fee and SPL token (e.g., TWT), which is split and escrowed safely.                                |
| **Vesting & Escrow**              | A portion of the mint price is locked in a per-nft vault, redeemable only after vesting conditions.                           |
| **Vesting Mode**                  | Vesting model of escrowed token (none, permanent, and timestamp).                                                             |
| **Burn & Refund**                 | Burn NFT → Immediately reclaim the escrowed tokens after vesting unlocks.                                                     |
| **Force Unlock**                  | Admin can override vesting and unlock escrow early if necessary (only for timestamp vesting mode).                            |
| **Update NFT**                    | NFT owner can update metadata (name / URI) with optional SOL fee via program authority.                                       |
| **Royalty**                       | Force royalty on resale market (max 5 recipients).                                                                            |
| **Revenue Wallet**                | Optionally user pay to revenue wallet (max 5 wallets).                                                                        |
| **Mint Guard**                    | Configurable max mint per wallet. Separate max mint for admin, vip, and regular user. Mint guard doesn't apply to admin user. |
| **VIP User**                      | VIP user has separate instruction and max mint nft per project.                                                               |
| **NFT Supply**                    | Constraint NFT max supply. Max supply is sum of user and admin supply. VIP and regular user use same released supply          |

## Program ID

```text
TWTfEU1tgnaErUq4BetvcskjrkV1Hz5K3pgS4ezzytt
```

## Instructions

### 1. InitConfigV1

Initialize global config.

- Sets admin.
- Sets vault/revenue rules.
- Sets mint metadata.
- Sets vesting mode and unlock timestamp.
- Cannot be called again unless program is upgraded.

### 2. UpdateConfigV1

Admin-only update to modify:

- Mint price.
- Revenue shares.
- Released supply.
- Vesting unlock timestamp.
- Max per-user mint limits.

Struct layout cannot change after deployment — only values.

### 3. Admin Mint — MintAdminV1

Used for team allocations, DAO mints, and reserved supply.

- Does **not** consume public `released` supply.
- Increments `admin_minted`.
- Enforces: `admin_minted ≤ max_supply - released`.
- A per-nft vault (if not existing).
- Transfers escrow_amount → vault.
- Optional mint fee (SOL).
- Creates an MPL Core NFT Asset.

### 4. VIP User Mint — MintVipV1

Same flow as user mint but:

- Higher limits: `max_mint_per_vip_user`.
- Still consumes from `released` supply.
- A per-nft vault (if not existing).
- Transfers escrow_amount → vault.
- Transfers revenue_shares → revenue wallets.
- Increments `user_minted`.
- Optional mint fee (SOL).
- Creates an MPL Core NFT Asset.

### 5. Public User Mint — MintUserV1

Standard mint for public users.

- Enforces per-wallet limit: `max_mint_per_user`.
- Enforces supply: `user_minted < released`.
- A per-nft vault (if not existing).
- Transfers escrow_amount → vault.
- Transfers revenue_shares → revenue wallets.
- Increments `user_minted`.
- Optional mint fee (SOL).
- Creates an MPL Core NFT Asset.

### 6. Burn & Refund — BurnAndRefundV1

User burns NFT and:

- Validates vesting unlock rules.
- Transfers escrow_amount back from vault to nft owner.
- Closes NFT asset.
- Closes minted_user_pda if applicable.
- Emits refund event.
- Refund behavior depends on `VestingMode`.

### 7. Force Unlock Vesting — ForceUnlockVestingV1

Admin-only override:

- Allows refund regardless of vesting schedule.
- Used for emergency unlock.
- Only unlocks one project (NFT collection + token mint).

### 8. Update NFT — UpdateNftV1

NFT owner can update:

- Name
- URI
- (Optional) SOL fee that goes to the protocol treasury

Uses NFT authority PDA to sign session authority mutation.

### 9. Traits Architecture (V1)

The codebase uses modular trait-based architecture:

| Instruction            | Role                                                           |
| ---------------------- | -------------------------------------------------------------- |
| **init_trait_v1.rs**   | Shared logic for trait config initialization                   |
| **update_trait_v1.rs** | Logic for trait config updates                                 |
| **mint_trait_v1.rs**   | Shared logic for all mint trait flows that follow trait config |

Traits allow consistent business logic across multiple instruction files.

## Account Structure

### PDAs

| PDA Seed                                                | Purpose                            |
| ------------------------------------------------------- | ---------------------------------- |
| `["config_v1", nft_collection, token_mint]`             | Global config & mint rules         |
| `["vault_v1", nft_asset, nft_collection, token_mint]`   | Per-nft token escrow               |
| `["user_minted_v1", nft_collection, token_mint, payer]` | Mint guard per wallet              |
| `["nft_authority_v1"]`                                  | MPL Core update / burn authority   |
| `["trait_authority_v1"]`                                | Trait update / burn authority      |
| `["trait_item_v1", trait_collection]`                   | Trait configuration and mint rules |

## Setup & Development

### Prerequisites

- [Rust](https://rustup.rs)
- [Solana CLI](https://solana.com/id/docs/intro/installation)
- [shank](https://github.com/metaplex-foundation/shank)

### Build

```sh
make build
```

### Deploy

`AUTH`: Wallet that will pay and own tweetonium program

```sh
make deploy AUTH=~/your/deployment/wallet/path
```

### Release

`AUTH`: Wallet that will pay and own tweetonium program

```sh
make release AUTH=~/your/deployment/wallet/path
```

### Generate IDL

```sh
make idl
```

> **IDL output**: ./idl/tweetonium.json

### Send IDL

`DEST`: Client code base path

```sh
make send DEST=~/your/dapp/code/base/path
```

## Makefile Commands

```sh
make build                          # Debug build
make deploy AUTH=...                # Deploy with upgrade authority
make release AUTH=...               # Build and deploy with upgrade authority
make change-authority NEW_AUTH=...  # Transfer upgrade authority
make verify                         # Show program info
make idl                            # Generate IDL
make send                           # Send IDL to client codebase
```

## Security & Audits

- **PDA validation** on all program-derived addresses.
- **Fixed account layout** for all program-derived addresses.
- **No reentrancy**: All state checks before external calls.
- **Mint supply caps** enforced.
- **Metaplex-compliant** NFT creation and update.
- **Strict vesting unlock** for vesting mode with timestamp.
- **Admin privileged** flows.
- **Signer checks** on all mutable actions.

## Use Cases

- Refundable mints
- Token-gated mints
- Vesting-based collectibles
- DAO revenue sharing
- NFT drops with guaranteed refund window
- VIP or tiered mint systems
- Multi-wallet revenue distribution
- Minted NFT with royalties

## License

MIT © 2025

Built for Solana. Powered by Rust. Secured by PDAs.

# Tweetonium – Solana Upgradable NFT Mint and Burn Program

[![Solana](https://img.shields.io/badge/Solana-14F195?logo=solana&logoColor=white)](https://solana.com)
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MPL Core](https://img.shields.io/badge/Metaplex-MPL%20Core-6C5CE7?logo=metaplex)](https://developers.metaplex.com/core)

**Tweetonium** is a secure, auditable Solana program that enables:

- **Minting NFTs** with token payment (e.g., $ZDLT)
- **Escrow & vesting** of payment tokens
- **Burn-to-refund** after vesting
- **Admin force-unlock**
- **Post-mint NFT metadata updates**

Built using **MPL Core** and **Anchor-free** Rust with **Shank IDL generation**.

## Features

| Feature                 | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| **Mint & Escrow**       | Pay in fungible tokens → Mint MPL Core NFT → Tokens locked in per-user vault |
| **Vesting Lock**        | Refund only after `vesting_end_ts`                                           |
| **Burn & Refund**       | Burn NFT → Instantly receive escrowed tokens                                 |
| **Force Unlock**        | Admin can unlock vesting early                                               |
| **Update NFT**          | NFT owner can update name/URI (with optional SOL fee)                        |
| **Per-user Mint Guard** | One mint per wallet                                                          |
| **Upgradeable**         | Full upgrade authority control                                               |

## Program ID

```text
8WNZn3jMFzHbPJ3mfahUD894hmTJvgn2T3VJncVSe8kA
```

## Instructions

| Instruction              | Description                      |
| ------------------------ | -------------------------------- |
| `InitConfigV1Initialize` | global config (admin only)       |
| `MintAndVaultV1`         | Pay → Mint NFT → Lock tokens     |
| `UpdateNftV1`            | Update NFT metadata (owner only) |
| `BurnAndRefundV1`        | Burn NFT → Get refund            |
| `ForceUnlockVestingV1`   | Admin: unlock vesting early      |

## Account Structure

### PDAs

| PDA Seed                  | Purpose                          |
| ------------------------- | -------------------------------- |
| `["config"]`              | Global config & mint rules       |
| `["vault", user_pubkey]`  | Per-user token escrow            |
| `["minted", user_pubkey]` | Mint guard                       |
| `["vault_authority"]`     | Signs token transfers from vault |
| `["nft_authority"]`       | MPL Core update/burn authority   |

## Setup & Development

### Prerequisites

- [Rust](https://rustup.rs)
- [Solana CLI](https://solana.com/id/docs/intro/installation)
- `cargo-build-sbf`
- [shank](https://github.com/metaplex-foundation/shank)

### Build

```bash
make build
```

### Deploy

`AUTH`: Wallet that will pay and own tweetonium program

```bash
make deploy AUTH=~/.config/solana/id.json
```

### Generate IDL

```bash
make idl
```

> **IDL output**: ./idl/tweetonium.json

## Makefile Commands

```bash
make build                          # Debug build
make build-release                  # Optimized build
make deploy AUTH=...                # Deploy with upgrade authority
make change-authority NEW_AUTH=...  # Transfer upgrade authority
make verify                         # Show program info
make idl                            # Generate IDL
```

## Security & Audits

- **No reentrancy**: All state checks before external calls
- **PDA validation** on all program-derived addresses
- **Signer checks** on all mutable actions
- **Mint supply caps** enforced
- **Double-mint protection** via minted_user_pda

> Ready for audit.

## Use Cases

- **Token-gated NFT drops**
- **Refundable collectibles**
- **Time-locked airdrops**
- **Burn-to-earn mechanics**

## Contributing

1. Fork the repo
2. Create a feature branch
3. Write tests (TBD)
4. Submit PR

## License

MIT © 2025

Built for Solana. Powered by Rust. Secured by PDAs.

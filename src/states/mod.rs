mod authorities;
mod config;
mod user_minted;
mod vault;
mod vesting;

pub use authorities::*;
pub use config::*;
pub use user_minted::*;
pub use vault::*;
pub use vesting::*;

pub const MAX_REVENUE_WALLETS: usize = 5;
pub const MAX_ROYALTY_RECIPIENTS: usize = 5;
pub const MAX_BASIS_POINTS: u16 = 10_000;

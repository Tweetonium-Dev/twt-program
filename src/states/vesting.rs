use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankType;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankType)]
pub enum VestingMode {
    /// No vesting lock is applied. Tokens are immediately withdrawable after mint.
    None = 0,

    /// Tokens are permanently locked and can never be withdrawn.
    Permanent = 1,

    /// Tokens unlock automatically once the on-chain timestamp exceeds `vesting_unlock_time`.
    TimeStamp = 2,
}

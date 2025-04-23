use anchor_lang::prelude::*;

use crate::{
    constants::constants, errors::ErrorCode, state::GlobalConfig
};

#[account]
#[derive(Debug)]
pub struct UserStakedAccount {
    pub amount_staked: u64,
    pub identifier: u64,
    pub last_claimed_timestamp: i64,
    pub staked_timestamp: i64,
    pub user: Pubkey,
}

impl UserStakedAccount {
    pub const LEN: usize = 8 + std::mem::size_of::<UserStakedAccount>();

    pub fn calculate_rewards(
        &self,
        global_config: &Account<GlobalConfig>,
        now: i64,
    ) -> Result<u64> {
        let mut rewards: u128 = 0;
        let stake_time = self.staked_timestamp;
        let mut last_claimed = self.last_claimed_timestamp;
        let principal = self.amount_staked as u128;
        let apy_history = global_config.apy_history.clone();
        let previous_apy = apy_history.first().unwrap().apy_bps as u128;
        let current_apy_start = apy_history.first().unwrap().timestamp;
        let current_apy = global_config.current_apy_bps as u128;
        let stake_hours = stake_time % constants::CLAIM_PERIOD_SECONDS;

        let last_claimed_day = last_claimed - (last_claimed % constants::CLAIM_PERIOD_SECONDS);
        let current_apy_start_day = current_apy_start - (current_apy_start % constants::CLAIM_PERIOD_SECONDS);

        if last_claimed_day < current_apy_start_day {
            let interval = (current_apy_start_day - last_claimed_day) / constants::CLAIM_PERIOD_SECONDS;

            let rewards_by_day = principal
                .checked_mul(previous_apy)
                .ok_or(ErrorCode::Overflow)?
                .checked_mul(constants::CLAIM_PERIOD_SECONDS as u128)
                .ok_or(ErrorCode::Overflow)?
                .checked_div(365 * 24 * 60 * 60 * 10_000) // Adjust based on staking period
                .ok_or(ErrorCode::Overflow)?;

            rewards += rewards_by_day * interval as u128;

            last_claimed = current_apy_start_day + stake_hours;
        }

        let interval = (now - last_claimed) / constants::CLAIM_PERIOD_SECONDS;

        let rewards_by_day = principal
            .checked_mul(current_apy)
            .ok_or(ErrorCode::Overflow)?
            .checked_mul(constants::CLAIM_PERIOD_SECONDS as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(365 * 24 * 60 * 60 * 10_000) // Adjust based on staking period
            .ok_or(ErrorCode::Overflow)?;

        rewards += rewards_by_day * interval as u128;

        msg!("Total rewards: {:?}", rewards);

        Ok(u64::try_from(rewards).map_err(|_| ErrorCode::Overflow)?)
    }
}

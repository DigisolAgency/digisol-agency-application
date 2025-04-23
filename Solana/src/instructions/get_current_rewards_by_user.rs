use crate::{
    constants::constants,
    errors::ErrorCode,
    state::{GlobalConfig, UserStakedAccount},
};

use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetCurrentRewardsByUser<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account()]
    pub user_staked_account: Account<'info, UserStakedAccount>,
}

pub fn calculate_current_rewards(
    ctx: Context<GetCurrentRewardsByUser>,
) -> Result<u128> {
    let now = Clock::get()?.unix_timestamp;

    let global_config = &ctx.accounts.global_config;
    let user_staked = &ctx.accounts.user_staked_account;

    let stake_time = user_staked.staked_timestamp;

    let today = now - (now % constants::CLAIM_PERIOD_SECONDS);
    let staked_day = stake_time - (stake_time % constants::CLAIM_PERIOD_SECONDS);
    let time_in_hours = stake_time - staked_day;
    let yesterday = today - constants::CLAIM_PERIOD_SECONDS;

    let staking_period = if staked_day == today {
        now - stake_time
    } else {
        (now - (yesterday + time_in_hours)) % constants::CLAIM_PERIOD_SECONDS
    };

    let apy_decimal = global_config.current_apy_bps as u128;
    let principal = user_staked.amount_staked as u128;

    let current_rewards = principal
        .checked_mul(apy_decimal)
        .ok_or(ErrorCode::Overflow)?
        .checked_mul(staking_period as u128)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(365 * 24 * 60 * 60 * 10_000) // Adjust based on staking period
        .ok_or(ErrorCode::Overflow)?;

    msg!("Current rewards: {:?}", current_rewards);

    Ok(current_rewards)
}

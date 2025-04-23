use crate::{constants, state::{GlobalConfig, UserStakedAccount}};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetRewardsByUser<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account()]
    pub user_staked_account: Account<'info, UserStakedAccount>,
}

pub fn calculate_rewards(
    ctx: Context<GetRewardsByUser>,
) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;
    let global_config = &ctx.accounts.global_config;
    let user_staked = &ctx.accounts.user_staked_account;
    let last_claim_time = user_staked.last_claimed_timestamp;

    let rewards = if last_claim_time + constants::CLAIM_PERIOD_SECONDS > now {
        0
    } else {
        user_staked.calculate_rewards(global_config, now)?
    };

    Ok(rewards)
}

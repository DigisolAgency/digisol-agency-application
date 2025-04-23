use crate::state::{GlobalConfig, StakingAccount};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetReferralRewards<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account()]
    pub staking_account: Account<'info, StakingAccount>,
}

pub fn calculate_referral_rewards(
    ctx: Context<GetReferralRewards>,
) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;
    let global_config = &ctx.accounts.global_config;
    let staking_account = &ctx.accounts.staking_account;

    let rewards = staking_account.calculate_referral_rewards(global_config, now)?;
    Ok(rewards)
}

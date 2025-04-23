use crate::state::{GlobalConfig, StakingAccount};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetAllRewardsByUser<'info> {
    #[account(
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account()]
    pub staking_account: Account<'info, StakingAccount>,

    /*
    REMAINING_ACCOUNTS: user_staked_accounts (limit 25)
     */
}

pub fn calculate_rewards(
    ctx: Context<GetAllRewardsByUser>,
) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;
    let global_config = &ctx.accounts.global_config;
    let staking_account = &ctx.accounts.staking_account;
    let remaining_accounts = ctx.remaining_accounts;

    let rewards = staking_account.calculate_all_rewards(
        remaining_accounts,
        global_config,
        now
    )?;

    Ok(rewards)
}

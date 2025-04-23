use anchor_lang::prelude::*;

use crate::state::{GlobalConfig, StakingAccount};

#[derive(Accounts)]
pub struct SetReferralsInfo<'info> {
    #[account(
        has_one = admin,
        seeds = [b"config"],
        bump = global_config.config_bumps
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,

    pub admin: Signer<'info>,
}

pub fn set_referrals_info(
    ctx: Context<SetReferralsInfo>,
    new_total_referral_staked: u64,
    new_daily_referral_rewards: u64,
) -> Result<()> {
    let staking_account = &mut ctx.accounts.staking_account;

    staking_account.total_referral_staked = new_total_referral_staked;
    staking_account.daily_referral_rewards = new_daily_referral_rewards;

    msg!(
        "Referral info updated: total staked {}, daily rewards {}",
        new_total_referral_staked,
        new_daily_referral_rewards
    );

    Ok(())
}
